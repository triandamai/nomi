use serde_json::json;
use tracing::info;
use uuid::Uuid;
use crate::feature::message_processor::{classify_media_context, extract_expense_data, extract_maintenance_data, extract_technical_doc, send_status_update, UnifiedMessage};
use crate::feature::message_processor::media::MediaClassification;
use crate::{rag, AppState};
use crate::prompts::StatusRegistry;

pub async fn classification(
    state:&AppState,
    conversation_id:Uuid,
    msg:&UnifiedMessage,
    text_content: String,
    injected_system_prompt: Option<String>,
)->(String,String){
    let mut media_context = String::new();
    if let Some(ref image_url) = msg.image_url {
        info!("Media detected, classifying: {}", image_url);

        send_status_update(
            &state,
            conversation_id,
            msg.source.clone(),
            "tool_start".to_string(),
            StatusRegistry::random_action_phrase("analyze_media"),
        );

        let classification = classify_media_context(&state, &image_url)
            .await
            .unwrap_or(MediaClassification::Other);

        media_context = proceed_classification(&classification, &state, image_url, conversation_id).await;
        let classification_str = match classification {
            MediaClassification::ExpenseReceipt => Some("EXPENSE_RECEIPT"),
            MediaClassification::MotorcycleMaintenance => Some("MOTORCYCLE_MAINTENANCE"),
            MediaClassification::TechnicalDoc => Some("TECHNICAL_DOC"),
            MediaClassification::Nature => Some("NATURE"),
            MediaClassification::Other => Some("OTHER"),
        };

        // Save to pending_media table for Media Checkpoint System
        let _ = crate::common::repository::pending_media_repo::upsert_pending_media(
            &state.pool,
            conversation_id,
            &image_url,
            "image",
            classification_str,
        ).await;
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
    return (augmented_text, media_context);
}

pub async fn proceed_classification(
    classification:&MediaClassification,
    state:&AppState,
    image_url: &str,
    conversation_id:Uuid
)->String{
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
                media_context =
                    crate::prompts::PromptRegistry::media_context_technical(summary);
                if let Ok(embedding) = rag::get_embedding(&state.gemini_api_key, &content).await
                {
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
        MediaClassification::Other => {
            media_context = crate::prompts::PromptRegistry::media_context_other().to_string();
            media_context
        }
    }
}