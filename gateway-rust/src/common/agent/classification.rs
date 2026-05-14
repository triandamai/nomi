use crate::common::agent::agent_model::{ExpenseData, MaintenanceData, MediaClassification};
use crate::feature::message_processor::model::UnifiedMessage;
use crate::feature::message_processor::v2_orchestrator::send_status_update;
use crate::prompts::{PromptRegistry, StatusRegistry};
use crate::rag::classify_media_context;
use crate::{AppState, rag};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use chrono::{Datelike, Utc};
use gemini_rust::{Content, Message};
use serde_json::json;
use tracing::info;
use uuid::Uuid;

pub async fn classification(
    state: &AppState,
    members: Vec<Uuid>,
    conversation_id: Uuid,
    msg: &UnifiedMessage,
    text_content: String,
    injected_system_prompt: Option<String>,
) -> (String, String, bool) {
    let mut media_context = String::new();
    let mut should_ignore = false;
    if let Some(ref image_url) = msg.image_url {
        let is_empty = text_content.trim().is_empty();
        let is_vague = text_content.trim().len() < 10; // Basic heuristic for "vague"

        // Task 1 & 3: Only trigger classify_media_context if text is empty or vague.
        if is_empty || is_vague {
            info!(
                "Media detected (vague/empty text), classifying: {}",
                image_url
            );

            send_status_update(
                &state,
                members.clone(),
                conversation_id,
                msg.source.clone(),
                msg.is_group,
                "tool_start".to_string(),
                StatusRegistry::random_action_phrase("analyze_media"),
            );

            let classification = classify_media_context(&state, &image_url, Some(text_content.clone()))
                .await
                .unwrap_or(MediaClassification::Other);

            if let MediaClassification::Ignore = classification {
                should_ignore = true;
            }

            // Task 1: If text is empty, we do the full proceeding (extraction/KB saving)
            // If it's vague, we might still want basic context for proactive suggestion.
            if is_empty {
                media_context =
                    proceed_classification(&classification, &state, image_url, conversation_id)
                        .await;
            } else {
                media_context = match classification {
                    MediaClassification::ExpenseReceipt => "[SYSTEM: This image appears to be an expense receipt. You might want to suggest logging it.]".to_string(),
                    MediaClassification::MotorcycleMaintenance => "[SYSTEM: This image appears to be a motorcycle maintenance record.]".to_string(),
                    MediaClassification::TechnicalDoc => "[SYSTEM: This image appears to be a technical document.]".to_string(),
                    MediaClassification::Nature => "[SYSTEM: This is a nature photo.]".to_string(),
                    MediaClassification::Ignore => "[SYSTEM: User requested to ignore this media.]".to_string(),
                    MediaClassification::Other => "[SYSTEM: Uncategorized image.]".to_string(),
                };
            }

            let classification_str = match classification {
                MediaClassification::ExpenseReceipt => Some("EXPENSE_RECEIPT"),
                MediaClassification::MotorcycleMaintenance => Some("MOTORCYCLE_MAINTENANCE"),
                MediaClassification::TechnicalDoc => Some("TECHNICAL_DOC"),
                MediaClassification::Nature => Some("NATURE"),
                MediaClassification::Ignore => Some("IGNORE"),
                MediaClassification::Other => Some("OTHER"),
            };

            // Save to pending_media table for Media Checkpoint System
            let pool = state.pool.clone();
            let i_url = image_url.to_string();
            tokio::spawn(async move {
                let _ = crate::common::repository::pending_media_repo::upsert_pending_media(
                    &pool,
                    conversation_id,
                    &i_url,
                    "image",
                    classification_str,
                )
                .await;
            });
        } else {
            // Specific text provided: Just ensure it's in pending_media for tools to find.
            let pool = state.pool.clone();
            let i_url = image_url.to_string();
            tokio::spawn(async move {
                let _ = crate::common::repository::pending_media_repo::upsert_pending_media(
                    &pool,
                    conversation_id,
                    &i_url,
                    "image",
                    None,
                )
                .await;
            });
        }
    }

    let mut augmented_text = if let Some(ref image_url) = msg.image_url {
        format!("[Image: {}]\n{}{}", image_url, text_content, media_context)
    } else {
        format!("{}{}", text_content, media_context)
    };

    if let Some(injected) = injected_system_prompt {
        augmented_text.push_str("\n\n");
        augmented_text.push_str(&injected);
    }
    return (augmented_text, media_context, should_ignore);
}

pub async fn proceed_classification(
    classification: &MediaClassification,
    state: &AppState,
    image_url: &str,
    conversation_id: Uuid,
) -> String {
    let mut media_context = String::new();
    match classification {
        MediaClassification::ExpenseReceipt => {
            if let Ok(expense) = extract_expense_data(&state, &image_url).await {
                media_context = crate::prompts::PromptRegistry::media_context_expense(
                    &expense.merchant,
                    &expense.total.to_string(),
                    &expense.category,
                    &expense
                        .items
                        .iter()
                        .map(|i| i.name.clone())
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                let memory_content = format!(
                    "Expense at {}: {} ({})",
                    expense.merchant, expense.total, expense.category
                );
                if let Ok(embedding) =
                    rag::get_embedding(&state.gemini_api_key, &memory_content).await
                {
                    let metadata = json!({
                        "type": "memory",
                        "source": "image_classification",
                        "classification": "EXPENSE_RECEIPT",
                        "data": expense,
                        "image_url": image_url
                    });
                    let _ = rag::save_to_knowledge_base(
                        &state.pool,
                        &memory_content,
                        embedding.embedding.values,
                        Some(metadata),
                        Some(conversation_id),
                        0,
                        0,
                        0,
                    )
                    .await;
                }
            }
            media_context
        }
        MediaClassification::MotorcycleMaintenance => {
            if let Ok(maint) = extract_maintenance_data(&state, &image_url).await {
                media_context = crate::prompts::PromptRegistry::media_context_maintenance(
                    &maint.part_names.join(", "),
                    &maint.service_details,
                );
                let memory_content = format!(
                    "Motorcycle Maintenance: {} - Parts: {}",
                    maint.service_details,
                    maint.part_names.join(", ")
                );
                if let Ok(embedding) =
                    rag::get_embedding(&state.gemini_api_key, &memory_content).await
                {
                    let metadata = json!({
                        "type": "memory",
                        "source": "image_classification",
                        "classification": "MOTORCYCLE_MAINTENANCE",
                        "graph": {
                            "nodes": maint.part_names.iter().map(|p| json!({"id": p.to_lowercase().replace(' ', "_"), "label": p, "node_type": "MaintenanceLog"})).collect::<Vec<_>>(),
                            "links": maint.part_names.iter().map(|p| json!({"source": "motorcycle", "target": p.to_lowercase().replace(' ', "_"), "relationship": "replaced_part"})).collect::<Vec<_>>()
                        },
                        "data": maint,
                        "image_url": image_url
                    });
                    let _ = rag::save_to_knowledge_base(
                        &state.pool,
                        &memory_content,
                        embedding.embedding.values,
                        Some(metadata),
                        Some(conversation_id),
                        0,
                        0,
                        0,
                    )
                    .await;
                }
            }
            media_context
        }
        MediaClassification::TechnicalDoc => {
            if let Ok(content) = extract_technical_doc(&state, &image_url).await {
                let summary = if content.len() > 100 {
                    &content[..100]
                } else {
                    &content
                };
                media_context = crate::prompts::PromptRegistry::media_context_technical(summary);
                if let Ok(embedding) = rag::get_embedding(&state.gemini_api_key, &content).await {
                    let metadata = json!({
                        "type": "memory",
                        "source": "image_classification",
                        "classification": "TECHNICAL_DOC",
                        "image_url": image_url
                    });
                    let _ = rag::save_to_knowledge_base(
                        &state.pool,
                        &content,
                        embedding.embedding.values,
                        Some(metadata),
                        Some(conversation_id),
                        0,
                        0,
                        0,
                    )
                    .await;
                }
            }
            media_context
        }
        MediaClassification::Nature => {
            media_context = crate::prompts::PromptRegistry::media_context_nature().to_string();
            media_context
        }
        MediaClassification::Ignore => {
            media_context = "".to_string();
            media_context
        }
        MediaClassification::Other => {
            media_context = crate::prompts::PromptRegistry::media_context_other().to_string();
            media_context
        }
    }
}

pub(crate) async fn extract_expense_data(
    state: &AppState,
    image_url: &str,
) -> anyhow::Result<ExpenseData> {
    let prompt = PromptRegistry::expense_extraction();

    let (mime_type, base64_data) = fetch_media_from_storage(state, image_url).await?;

    let res = state
        .gemini
        .generate_content()
        .with_user_message(prompt)
        .with_message(Message {
            role: gemini_rust::Role::User,
            content: Content::inline_data(mime_type, base64_data),
        })
        .execute()
        .await?;

    if let Some(usage) = &res.usage_metadata {
        info!(
            "Expense extraction tokens: prompt={}, candidates={}, total={}",
            usage.prompt_token_count.unwrap_or(0),
            usage.candidates_token_count.unwrap_or(0),
            usage.total_token_count.unwrap_or(0)
        );
    }

    let text = res.text();
    let json_str = if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            &text[start..=end]
        } else {
            text.as_str()
        }
    } else {
        text.as_str()
    };

    let data: ExpenseData = serde_json::from_str(json_str)?;
    Ok(data)
}

pub(crate) async fn log_expense_transaction(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    conversation_id: Option<Uuid>,
    data: &ExpenseData,
) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;

    // 1. Get or create category
    let category_id = sqlx::query!(
        "SELECT id FROM categories WHERE slug = $1 OR name = $1 LIMIT 1",
        data.category.to_lowercase()
    )
    .fetch_optional(&mut *tx)
    .await?
    .map(|r| r.id);

    // 2. Insert main record
    let record = sqlx::query!(
        r#"
        INSERT INTO money_tracking (
            user_id, conversation_id, category_id, category, merchant_name,
            total_amount, tax, service, discount
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id
        "#,
        user_id,
        conversation_id,
        category_id,
        data.category,
        data.merchant,
        rust_decimal::Decimal::from_f64_retain(data.total).unwrap_or_default(),
        data.tax
            .map(|v| rust_decimal::Decimal::from_f64_retain(v).unwrap_or_default()),
        data.service
            .map(|v| rust_decimal::Decimal::from_f64_retain(v).unwrap_or_default()),
        data.discount
            .map(|v| rust_decimal::Decimal::from_f64_retain(v).unwrap_or_default())
    )
    .fetch_one(&mut *tx)
    .await?;

    // 3. Insert items
    for item in &data.items {
        sqlx::query!(
            r#"
            INSERT INTO money_tracking_items (money_tracking_id, name, quantity, total_amount)
            VALUES ($1, $2, $3, $4)
            "#,
            record.id,
            item.name,
            item.quantity,
            rust_decimal::Decimal::from_f64_retain(item.amount).unwrap_or_default()
        )
        .execute(&mut *tx)
        .await?;
    }

    // 4. Update Summary
    let now = Utc::now().date_naive();
    let period_start = now.with_day(1).unwrap_or(now);

    sqlx::query!(
        r#"
        INSERT INTO money_tracking_summary (user_id, total_expenses, period, updated_at)
        VALUES ($1, $2, $3, now())
        ON CONFLICT (user_id, period) DO UPDATE
        SET total_expenses = money_tracking_summary.total_expenses + EXCLUDED.total_expenses,
            updated_at = now()
        "#,
        user_id,
        rust_decimal::Decimal::from_f64_retain(data.total).unwrap_or_default(),
        period_start
    )
    .execute(&mut *tx)
    .await?;

    // 5. Cleanup Pending Media if it exists for this conversation
    if let Some(cid) = conversation_id {
        let _ = sqlx::query!("DELETE FROM pending_media WHERE conversation_id = $1", cid)
            .execute(&mut *tx)
            .await;
    }

    tx.commit().await?;
    Ok(())
}

pub(crate) async fn extract_maintenance_data(
    state: &AppState,
    image_url: &str,
) -> anyhow::Result<MaintenanceData> {
    let prompt = crate::prompts::PromptRegistry::maintenance_extraction();

    let (mime_type, base64_data) = fetch_media_from_storage(state, image_url).await?;

    let res = state
        .gemini
        .generate_content()
        .with_user_message(prompt)
        .with_message(Message {
            role: gemini_rust::Role::User,
            content: Content::inline_data(mime_type, base64_data),
        })
        .execute()
        .await?;

    if let Some(usage) = &res.usage_metadata {
        info!(
            "Maintenance extraction tokens: prompt={}, candidates={}, total={}",
            usage.prompt_token_count.unwrap_or(0),
            usage.candidates_token_count.unwrap_or(0),
            usage.total_token_count.unwrap_or(0)
        );
    }

    let text = res.text();
    let json_str = if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            &text[start..=end]
        } else {
            text.as_str()
        }
    } else {
        text.as_str()
    };

    let data: MaintenanceData = serde_json::from_str(json_str)?;
    Ok(data)
}

pub(crate) async fn extract_technical_doc(
    state: &AppState,
    image_url: &str,
) -> anyhow::Result<String> {
    let prompt = crate::prompts::PromptRegistry::technical_doc_summarization();

    let (mime_type, base64_data) = fetch_media_from_storage(state, image_url).await?;

    let res = state
        .gemini
        .generate_content()
        .with_user_message(prompt)
        .with_message(Message {
            role: gemini_rust::Role::User,
            content: Content::inline_data(mime_type, base64_data),
        })
        .execute()
        .await?;

    if let Some(usage) = &res.usage_metadata {
        info!(
            "Technical doc extraction tokens: prompt={}, candidates={}, total={}",
            usage.prompt_token_count.unwrap_or(0),
            usage.candidates_token_count.unwrap_or(0),
            usage.total_token_count.unwrap_or(0)
        );
    }

    Ok(res.text())
}

pub(crate) async fn fetch_media_from_storage(
    state: &AppState,
    image_url: &str,
) -> anyhow::Result<(String, String)> {
    let bucket = "conversations";
    let internal_path = state.storage.get_full_url(image_url);

    // image_url from channel is typically just the filename/path in storage
    let data = state
        .storage
        .get_file(bucket.to_string(), image_url.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("Storage error: {}", e))?;

    let mime_type = mime_guess::from_path(&internal_path)
        .first_or_octet_stream()
        .to_string();

    let b64 = BASE64.encode(data.to_vec());
    Ok((mime_type, b64))
}
