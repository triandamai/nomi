pub mod rag_model;

use std::sync::Arc;
use gemini_rust::{Blob, Content, Message, Part, Role, UsageMetadata};
use crate::rag::rag_model::EmbeddingResponse;
use reqwest::Client as ReqwestClient;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::PgQueryResult;
use sqlx::{Error, Pool, Postgres};
use tracing::info;
use uuid::Uuid;
use crate::common::sse::sse_builder::{SseBuilder, SseTarget};
use crate::common::sse::sse_emitter::SseBroadcaster;
use crate::{rag, AppState};
use crate::common::agent::agent_model::MediaClassification;
use crate::common::agent::classification::fetch_media_from_storage;
use crate::prompts::PromptRegistry;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub content: String,
    pub similarity: Option<f64>,
}

pub async fn get_embedding(api_key: &str, text: &str) -> anyhow::Result<EmbeddingResponse, String> {
    let client = ReqwestClient::new();
    // Ensure this is NOT set to 768, or explicitly set to 3072
    let res = client
        .post(format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-embedding-2:embedContent?key={}", api_key))
        .json(&json!({
            "content": {
                "parts": [{ "text": &text }]
            },
            "output_dimensionality":3072}))
        .send().await;

    if let Err(err) = res {
        return Err(format!("{:?}", err));
    }
    let response = res.unwrap();
    let status = response.status();

    if !status.is_success() {
        return Err(format!("{:?}", status));
    }
    let data = response.json::<EmbeddingResponse>().await;
    if let Err(err) = data {
        return Err(format!("{:?}", err));
    }

    Ok(data.unwrap())
}

pub async fn search_similar(
    pool: &Pool<Postgres>,
    embedding: Vec<f32>,
    limit: i32,
) -> anyhow::Result<Vec<SearchResult>> {
    let results = sqlx::query_as!(
        SearchResult,
        r#"
        SELECT content, (1 - (embedding <=> $1::vector)) as similarity
        FROM knowledge_base
        ORDER BY embedding <=> $1::vector
        LIMIT $2
        "#,
        embedding as Vec<f32>,
        limit as i64
    )
    .fetch_all(pool)
    .await?;

    Ok(results)
}

pub async fn search_similar_with_summaries(
    pool: &Pool<Postgres>,
    embedding: Vec<f32>,
    limit: i32,
) -> anyhow::Result<Vec<SearchResult>> {
    // Prioritize summaries by using a union or a conditional ordering
    // Here we fetch both summaries and normal entries, ordered by similarity,
    // but giving a slight boost or priority to summaries if they are relevant enough.
    // For simplicity and directness to requirements, we use a query that handles the metadata filtering.
    let results = sqlx::query_as!(
        SearchResult,
        r#"
        SELECT content, (1 - (embedding <=> $1::vector)) as similarity
        FROM knowledge_base
        ORDER BY 
            (metadata->>'type' = 'summary') DESC,
            embedding <=> $1::vector
        LIMIT $2
        "#,
        embedding as Vec<f32>,
        limit as i64
    )
    .fetch_all(pool)
    .await?;

    Ok(results)
}

pub async fn save_to_knowledge_base(
    pool: &Pool<Postgres>,
    content: &str,
    embedding: Vec<f32>,
    metadata: Option<serde_json::Value>,
    conversation_id: Option<uuid::Uuid>,
    prompt_tokens: i32,
    answer_tokens: i32,
    total_tokens: i32,
) -> Result<PgQueryResult, Error> {
    sqlx::query!(
        r#"
        INSERT INTO knowledge_base (content, embedding, metadata, conversation_id, prompt_tokens, answer_tokens, total_tokens)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        content,
        embedding as Vec<f32>,
        metadata.unwrap_or(json!({})),
        conversation_id,
        prompt_tokens,
        answer_tokens,
        total_tokens
    )
    .execute(pool)
    .await
}



pub(crate) async fn trigger_memory_consolidation(
    pool: sqlx::PgPool,
    gemini: std::sync::Arc<gemini_rust::Gemini>,
    gemini_api_key: String,
    conversation_id: Uuid,
    sse: Arc<SseBroadcaster>,
) -> anyhow::Result<()> {
    // 1. Get the last summary's timestamp and last_message_id
    let last_summary = sqlx::query!(
        r#"
        SELECT
            created_at as "last_summary_at!",
            metadata->>'last_message_id' as last_message_id
        FROM knowledge_base
        WHERE metadata->>'type' = 'summary'
        AND metadata->>'conversation_id' = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
        conversation_id.to_string()
    )
        .fetch_optional(&pool)
        .await?;

    // Default to Unix Epoch (1970-01-01) if no previous summary exists
    let start_timestamp = last_summary
        .as_ref()
        .map(|s| s.last_summary_at)
        .unwrap_or_else(|| {
            chrono::DateTime::from_timestamp(0, 0)
                .unwrap()
                .with_timezone(&chrono::Utc)
        });

    // 2. Fetch new messages ordered by created_at ASC
    let new_messages = sqlx::query!(
        r#"
        SELECT id, role, content
        FROM messages
        WHERE conversation_id = $1
        AND created_at > $2
        ORDER BY created_at ASC
        "#,
        conversation_id,
        start_timestamp
    )
        .fetch_all(&pool)
        .await
        .unwrap_or(Vec::new());

    info!("Check memory consolidation: {}", new_messages.len());
    // 3. Threshold check
    if new_messages.len() >= 10 {
        info!(conversation_id = %conversation_id, "Memory consolidation triggered ({} new messages)", new_messages.len());

        let last_processed_id = new_messages.last().map(|m| m.id).unwrap();
        let mut summary_input = String::new();
        for msg in new_messages {
            summary_input.push_str(&format!("{}: {}\n", msg.role, msg.content));
        }

        let summarizer_prompt =
            crate::prompts::PromptRegistry::memory_consolidation_summarizer(&summary_input);

        let summary_res = gemini
            .generate_content()
            .with_user_message(summarizer_prompt)
            .execute()
            .await?;

        let raw_json = summary_res.text();
        let parsed_data: serde_json::Value = if let Some(start) = raw_json.find('{') {
            if let Some(end) = raw_json.rfind('}') {
                serde_json::from_str(&raw_json[start..=end])
                    .unwrap_or(json!({ "summary": raw_json, "nodes": [], "edges": [] }))
            } else {
                json!({ "summary": raw_json, "nodes": [], "edges": [] })
            }
        } else {
            json!({ "summary": raw_json, "nodes": [], "edges": [] })
        };

        let summary_text = parsed_data["summary"]
            .as_str()
            .unwrap_or(&raw_json)
            .to_string();

        if let Ok(embedding) = rag::get_embedding(&gemini_api_key, &summary_text).await {
            let metadata = json!({
                "type": "summary",
                "conversation_id": conversation_id.to_string(),
                "last_message_id": last_processed_id.to_string(),
                "graph": {
                    "nodes": parsed_data["nodes"],
                    "links": parsed_data["edges"]
                }
            });

            let usage = summary_res.usage_metadata.unwrap_or(UsageMetadata {
                prompt_token_count: None,
                candidates_token_count: None,
                total_token_count: None,
                thoughts_token_count: None,
                prompt_tokens_details: None,
                cached_content_token_count: None,
                cache_tokens_details: None,
            });
            let p_tokens = usage.prompt_token_count.unwrap_or(0);
            let a_tokens = usage.candidates_token_count.unwrap_or(0);
            let t_tokens = usage.total_token_count.unwrap_or(0);

            let mut tx = pool.begin().await?;

            rag::save_to_knowledge_base(
                &pool,
                &summary_text,
                embedding.embedding.values,
                Some(metadata),
                Some(conversation_id.clone()),
                p_tokens,
                a_tokens,
                t_tokens,
            )
                .await?;

            let updated_row = sqlx::query!(
                "UPDATE conversations SET cumulative_tokens = COALESCE(cumulative_tokens, 0) + $1 WHERE id = $2 RETURNING cumulative_tokens",
                t_tokens,
                conversation_id
            )
                .fetch_one(&mut *tx)
                .await?;

            tx.commit().await?;

            // Broadcast SSE token_update
            let _ = sse
                .send(SseBuilder::new(
                    SseTarget::broadcast("token_update".to_string()),
                    serde_json::json!({
                        "conversation_id": conversation_id,
                        "cumulative_tokens": updated_row.cumulative_tokens
                    }),
                ))
                .await;

            info!(
                conversation_id = %conversation_id,
                total_tokens = %updated_row.cumulative_tokens.unwrap_or(0),
                "Memory consolidation complete"
            );
        }
    }

    Ok(())
}

pub(crate) async fn classify_media_context(
    state: &AppState,
    media_url: &str,
    text_content: Option<String>,
) -> anyhow::Result<MediaClassification> {
    let mut prompt = PromptRegistry::media_classification().to_string();

    if let Some(text) = text_content {
        prompt.push_str(&format!("\n\nUser text provided: \"{}\"", text));
    }

    let (mime_type, base64_data) = fetch_media_from_storage(state, media_url).await?;

    info!(image_url = %media_url);
    info!("MIME type is {}", mime_type);
    // info!("Base64 data is {}", base64_data);
    let res = state
        .gemini
        .generate_content()
        .with_message(Message {
            role: Role::User,
            content: Content {
                parts: Some(vec![
                    Part::Text {
                        text: prompt, // The Instruction + User Context
                        thought: None,
                        thought_signature: None,
                    },
                    Part::InlineData {
                        inline_data: Blob {
                            mime_type,
                            data: base64_data, // The Image
                        },

                        media_resolution: None,
                    },
                ]),
                role: Some(Role::User),
            },
        })
        .execute()
        .await?;

    if let Some(usage) = &res.usage_metadata {
        info!(
            "Media classification tokens: prompt={}, candidates={}, total={}",
            usage.prompt_token_count.unwrap_or(0),
            usage.candidates_token_count.unwrap_or(0),
            usage.total_token_count.unwrap_or(0)
        );
    }

    let text = res.text().trim().to_uppercase();
    if text.contains("EXPENSE_RECEIPT") {
        Ok(MediaClassification::ExpenseReceipt)
    } else if text.contains("MOTORCYCLE_MAINTENANCE") {
        Ok(MediaClassification::MotorcycleMaintenance)
    } else if text.contains("TECHNICAL_DOC") {
        Ok(MediaClassification::TechnicalDoc)
    } else if text.contains("NATURE") {
        Ok(MediaClassification::Nature)
    } else if text.contains("IGNORE") {
        Ok(MediaClassification::Ignore)
    } else {
        Ok(MediaClassification::Other)
    }
}