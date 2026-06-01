pub mod rag_model;
pub mod retrieval;
pub use retrieval::RagRetrieval;

use crate::prompts::PromptRegistry;
use crate::rag::rag_model::EmbeddingResponse;
use anyhow::anyhow;
use gemini_rust::UsageMetadata;
use reqwest::Client as ReqwestClient;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Error, Pool, Postgres};
use tracing::info;
use uuid::Uuid;


#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub content: String,
    pub similarity: Option<f64>,
}

pub async fn get_embedding(api_key: &str, text: &str) -> anyhow::Result<EmbeddingResponse, String> {
    let client = ReqwestClient::new();
    let base_url = std::env::var("GEMINI_BASE_URL")
        .unwrap_or_else(|_| "https://generativelanguage.googleapis.com".to_string());

    let res = client
        .post(format!("{}/v1beta/models/gemini-embedding-2:embedContent?key={}", base_url, api_key))
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
) -> Result<Uuid, Error> {
    match sqlx::query!(
        r#"
        INSERT INTO knowledge_base (content, embedding, metadata, conversation_id, prompt_tokens, answer_tokens, total_tokens)
        VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id
        "#,
        content,
        embedding as Vec<f32>,
        metadata.unwrap_or(json!({})),
        conversation_id,
        prompt_tokens,
        answer_tokens,
        total_tokens
    )
        .fetch_one(pool)
        .await{
        Ok(value) => Ok(value.id),
        Err(err) => Err(err)
    }
}

pub(crate) async fn trigger_memory_consolidation(
    pool: sqlx::PgPool,
    gemini: std::sync::Arc<gemini_rust::Gemini>,
    gemini_api_key: String,
    conversation_id: Uuid,
) -> anyhow::Result<(Uuid, i64)> {
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
    if new_messages.len() < 10 {
        info!("Message length under 10, aborting");
        return Err(anyhow!("Message length is under 10"));
    }
    info!(conversation_id = %conversation_id, "Memory consolidation triggered ({} new messages)", new_messages.len());

    let last_processed_id = new_messages.last().map(|m| m.id).unwrap();
    let mut summary_input = String::new();
    for msg in new_messages {
        summary_input.push_str(&format!("{}: {}\n", msg.role, msg.content));
    }

    let summarizer_prompt = PromptRegistry::memory_consolidation_summarizer(&summary_input);

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
    let embedding = get_embedding(&gemini_api_key, &summary_text).await;
    if let Err(err) = embedding {
        info!("Error get embedding for {}", err);
        return Err(anyhow!("Error get embedding for {}", err));
    }
    let embedding = embedding.unwrap();
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

    let save_knowledge = save_to_knowledge_base(
        &pool,
        &summary_text,
        embedding.embedding.values,
        Some(metadata),
        Some(conversation_id.clone()),
        p_tokens,
        a_tokens,
        t_tokens,
    )
    .await;
    if let Err(err) = save_knowledge {
        info!("Error save knowledge for {}", err);
        return Err(anyhow!("Error save knowledge"));
    }

    let tx = pool.begin().await;
    if let Err(err) = tx {
        info!("Failed start trx {}", err);
        return Err(anyhow!("Failed begin trx :{}", err));
    }
    let mut tx = tx?;

    let updated_row = sqlx::query!(
        "UPDATE conversations SET cumulative_tokens = COALESCE(cumulative_tokens, 0) + $1 WHERE id = $2 RETURNING cumulative_tokens",
        t_tokens,conversation_id
    ).fetch_one(&mut *tx).await;

    if let Err(err) = updated_row {
        info!("Error update row {}", err);
        let _ = tx.rollback().await;
        return Err(anyhow!("Error update row {}", err));
    }
    
    if let Err(err) = tx.commit().await {
        info!("Error update row  {}", err);
        return Err(anyhow!("Error update row  {}", err));
    }
    let updated_row = updated_row.unwrap();

    // Parallel background telemetry logging (Moved AFTER commit)
    let pool_clone = pool.clone();
    let conv_id = conversation_id.clone();
    let input_tokens = p_tokens as i64;
    let output_tokens = a_tokens as i64;
    let tot_tokens = t_tokens as i64;
    
    tokio::spawn(async move {
        let _ = crate::services::ambient_soul::AmbientSoulService::log_token_transaction(
            &pool_clone,
            Some(conv_id),
            None,
            None,
            "knowledge",
            "system",
            input_tokens,
            output_tokens,
            tot_tokens,
        ).await;
    });

    info!(
        conversation_id = %conversation_id,
        total_tokens = %updated_row.cumulative_tokens.unwrap_or(0),
        "Memory consolidation complete"
    );

    Ok((
        conversation_id,
        updated_row
            .cumulative_tokens
            .unwrap_or(0)
            .to_i64()
            .unwrap_or(0),
    ))
}
