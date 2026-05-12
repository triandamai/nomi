use crate::AppState;
use crate::feature::message_processor::media::{ExpenseData, MaintenanceData, MediaClassification};
use crate::feature::message_processor::model::UnifiedMessage;
use crate::feature::{InboundMessage, OutboundMessage};
use crate::rag;
use chrono::{Datelike, Utc};
use gemini_rust::{Blob, Content, Message, Part, Role, UsageMetadata};
use serde_json::json;
use uuid::Uuid;

use crate::common::repository::{channel_repo, pairing_repo};
use crate::prompts::PromptRegistry;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rand::RngExt;
use tracing::{error, info};

pub async fn process_incoming_message(state: AppState, msg: UnifiedMessage) -> anyhow::Result<()> {
    if msg.v2 {
        return crate::feature::message_processor::v2_orchestrator::process_v2_message(state, msg).await;
    }
    info!("Received v1 message: {:?}, deprecated, ignoring", msg);
    Ok(())
}

pub(crate) async fn trigger_memory_consolidation(
    pool: sqlx::PgPool,
    gemini: std::sync::Arc<gemini_rust::Gemini>,
    gemini_api_key: String,
    conversation_id: Uuid,
) -> anyhow::Result<()> {
    // 1. Get the last summarized message ID
    let last_summary = sqlx::query!(
        r#"
        SELECT metadata->>'last_message_id' as last_message_id
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

    let last_msg_id = last_summary
        .and_then(|r| r.last_message_id)
        .and_then(|id| Uuid::parse_str(&id).ok());

    // 2. Fetch new messages
    let new_messages = sqlx::query!(
        r#"
        SELECT id, role, content 
        FROM messages 
        WHERE conversation_id = $1 
        AND ($2::uuid IS NULL OR created_at > (SELECT created_at FROM messages WHERE id = $2))
        ORDER BY created_at ASC
        "#,
        conversation_id,
        last_msg_id
    )
    .fetch_all(&pool)
    .await?;

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

            sqlx::query!(
                "UPDATE conversations SET cumulative_tokens = cumulative_tokens + $1 WHERE id = $2",
                t_tokens,
                conversation_id
            )
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;
            info!(conversation_id = %conversation_id, "Memory consolidation complete");
        }
    }

    Ok(())
}

pub(crate) async fn classify_media_context(
    state: &AppState,
    media_url: &str,
) -> anyhow::Result<MediaClassification> {
    let prompt = PromptRegistry::media_classification();

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
                        text: prompt.to_string(), // The Instruction
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
    } else {
        Ok(MediaClassification::Other)
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

async fn fetch_media_from_storage(
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

pub async fn process_generate_pairing(
    state: &AppState,
    msg: &InboundMessage,
    user_id: Uuid,
) -> anyhow::Result<()> {
    let conv_id = if msg.is_group {
        sqlx::query!(
            "SELECT c.id
            FROM channel_group as g
            RIGHT JOIN conversations as c ON c.id = g.conversation_id
            WHERE g.external_group_id = $1 AND g.channel = $2",
            msg.conversation_id,
            msg.channel
        )
        .fetch_one(&state.pool)
        .await
        .map_or_else(|_| Uuid::nil(), |result| result.id)
    } else {
        sqlx::query!(
            "SELECT c.id
            FROM channels as g
            RIGHT JOIN conversations as c ON c.id = g.conversation_id
            WHERE g.external_chat_id = $1 AND g.channel_type = $2",
            msg.conversation_id,
            msg.channel
        )
        .fetch_one(&state.pool)
        .await
        .map_or_else(|_| Uuid::nil(), |result| result.id)
    };
    if conv_id.is_nil() {
        info!("Failed get conversation_id");
        let _ = state
            .publish_outbond(&OutboundMessage {
                is_group: true,
                sender_id: "nomi".to_string(),
                conversation_id: msg.conversation_id.clone(),
                text: "Whoops! 🏍️💨 Only a registered Nomi user can pair me with a group."
                    .to_string(),
                channel: msg.channel.clone(),
                video_url: None,
                image_url: None,
                audio_url: None,
                doc_url: None,
                sticker_url: None,
                metadata: msg.metadata.clone(),
            })
            .await;
        return Ok(());
    }
    match pairing_repo::create_pairing_code(&state, conv_id, user_id).await {
        Ok(code) => {
            let _ = state
                .publish_outbond(&OutboundMessage {
                    is_group: true,
                    sender_id: "nomi".to_string(),
                    conversation_id: msg.conversation_id.clone(),
                    text: format!("/pair {}", code.pairing_code),
                    channel: msg.channel.clone(),
                    video_url: None,
                    image_url: None,
                    audio_url: None,
                    doc_url: None,
                    sticker_url: None,
                    metadata: msg.metadata.clone(),
                })
                .await;

            let _ = state
                .publish_outbond(&OutboundMessage {
                    is_group: true,
                    sender_id: "nomi".to_string(),
                    conversation_id: msg.conversation_id.clone(),
                    text: format!(
                        "User the code to conversation you want pair with. \n Expired at:{}",
                        code.expires_at.to_rfc3339()
                    ),
                    channel: msg.channel.clone(),
                    video_url: None,
                    image_url: None,
                    audio_url: None,
                    doc_url: None,
                    sticker_url: None,
                    metadata: msg.metadata.clone(),
                })
                .await;

            Ok(())
        }
        Err(err) => {
            info!("Failed create pairing code :{}", err);
            let _ = state
                .publish_outbond(&OutboundMessage {
                    is_group: true,
                    sender_id: "nomi".to_string(),
                    conversation_id: msg.conversation_id.clone(),
                    text: "Whoops! 🏍️💨 Only a registered Nomi user can pair me with a group."
                        .to_string(),
                    channel: msg.channel.clone(),
                    video_url: None,
                    image_url: None,
                    audio_url: None,
                    doc_url: None,
                    sticker_url: None,
                    metadata: msg.metadata.clone(),
                })
                .await;
            Ok(())
        }
    }
}
pub async fn process_pairing(
    state: &AppState,
    msg: &InboundMessage,
    text: &str,
    user_id: Uuid,
) -> anyhow::Result<()> {
    let parts: Vec<&str> = text.split_whitespace().collect();
    if parts.len() >= 2 {
        let code = parts[1].to_uppercase();
        if let Some(conv_id) = pairing_repo::validate_pairing_code(&state.pool, &code).await? {
            let display_name = match msg.metadata.clone() {
                None => None,
                Some(meta) => meta
                    .get("display_name")
                    .map_or_else(|| None, |v| Some(v.to_string())),
            };

            pairing_repo::complete_pairing(&state.pool, &code, user_id).await?;
            channel_repo::link_channel(
                &state.pool,
                &msg.channel,
                &msg.sender_id,
                &msg.conversation_id,
                conv_id,
                user_id,
                display_name,
            )
            .await?;

            let _ = state
                .send_to_user(
                    user_id.to_string().as_str(),
                    "pairing_success",
                    json!({
                        "conversation_id": conv_id,
                        "platform": msg.channel,
                        "message": format!("Successfully paired with {}!", msg.channel)
                    }),
                    &OutboundMessage {
                        is_group: msg.is_group,
                        sender_id: msg.sender_id.clone(),
                        conversation_id: msg.conversation_id.clone(),
                        text: "Pairing successful! This conversation is now linked.".to_string(),
                        channel: msg.channel.clone(),
                        video_url: None,
                        image_url: None,
                        audio_url: None,
                        doc_url: None,
                        sticker_url: None,
                        metadata: msg.metadata.clone(),
                    },
                )
                .await;

            return Ok(());
        }
    }
    Ok(())
}

pub async fn process_register(state: &AppState, msg: &InboundMessage) -> anyhow::Result<()> {
    info!(
        "start registering from channel {} sender_id {}",
        msg.channel, msg.sender_id
    );

    if msg.is_group {
        return process_group_registration(state, msg).await;
    }
    let channel_exists = sqlx::query!("SELECT u.id as user_id FROM channels c JOIN users u ON u.id = c.user_id WHERE c.channel_type = $1 AND c.external_chat_id = $2",msg.channel,msg.conversation_id)
        .fetch_optional(&state.pool)
        .await;
    if let Err(err) = channel_exists {
        info!("failed register because error getting information: {}", err);
        let _ = state
            .publish_outbond(&OutboundMessage {
                is_group: msg.is_group,
                sender_id: msg.sender_id.clone(),
                conversation_id: msg.conversation_id.clone(),
                text: crate::prompts::PromptRegistry::error_general_trouble().to_string(),
                channel: msg.channel.clone(),
                video_url: None,
                image_url: None,
                audio_url: None,
                doc_url: None,
                sticker_url: None,
                metadata: msg.metadata.clone(),
            })
            .await;
        return Ok(());
    }
    let channel_result = channel_exists?;
    if let Some(value) = channel_result {
        info!("failed register because user exist: {}", value.user_id);
        let _ = state
            .publish_outbond(&OutboundMessage {
                is_group: msg.is_group,
                sender_id: msg.sender_id.clone(),
                conversation_id: msg.conversation_id.clone(),
                text: crate::prompts::PromptRegistry::error_account_exists().to_string(),
                channel: msg.channel.clone(),
                video_url: None,
                image_url: None,
                audio_url: None,
                doc_url: None,
                sticker_url: None,
                metadata: msg.metadata.clone(),
            })
            .await;

        return Ok(());
    }

    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            error!("Failed to start transaction: {}", e);
            let _ = state
                .publish_outbond(&OutboundMessage {
                    is_group: msg.is_group,
                    sender_id: msg.sender_id.clone(),
                    conversation_id: msg.conversation_id.clone(),
                    text: "Internal server error".to_string(),
                    channel: msg.channel.clone(),
                    video_url: None,
                    image_url: None,
                    audio_url: None,
                    doc_url: None,
                    sticker_url: None,
                    metadata: msg.metadata.clone(),
                })
                .await;
            return Ok(());
        }
    };

    info!("begin create user \n");

    let display_name = match msg.metadata.clone() {
        None => msg.sender_id.clone(),
        Some(meta) => meta
            .get("display_name")
            .map_or_else(|| msg.sender_id.clone(), |v| v.to_string()),
    };

    let u_id = match sqlx::query!(
            "INSERT INTO users (external_id, display_name) VALUES ($1, $2) ON CONFLICT (external_id) DO UPDATE SET display_name = EXCLUDED.display_name RETURNING id",
            msg.sender_id,
            display_name
        ).fetch_one(&mut *tx).await {
        Ok(r) => r.id,
        Err(e) => {
            error!("Failed to resolve user: {}", e);
            let _ = tx.rollback().await;
            let _ = state.publish_outbond(&crate::feature::OutboundMessage {
                is_group: msg.is_group,
                sender_id: msg.sender_id.clone(),
                conversation_id: msg.conversation_id.clone(),
                text: "Failed to resolve user".to_string(),
                channel: msg.channel.clone(),
                video_url: None,
                image_url: None,
                audio_url: None,
                doc_url: None,
                sticker_url: None,
                metadata: msg.metadata.clone(),
            }).await;

            return Ok(());
        }
    };

    info!("begin create conversation \n");
    // Create new conversation
    let conv_id = Uuid::new_v4();
    let title = format!("{} via {}", msg.conversation_id, msg.channel);

    if let Err(e) = sqlx::query!(
        "INSERT INTO conversations (id, title,soul_content,bootstrap_content) VALUES ($1, $2,$3,$4)",
        conv_id,
        title,
        PromptRegistry::default_soul_prompts(),
        PromptRegistry::default_bootstrap_content()
    )
    .execute(&mut *tx)
    .await
    {
        error!("Failed to create conversation: {}", e);
        let _ = tx.rollback().await;
        let _ = state
            .publish_outbond(&crate::feature::OutboundMessage {
                is_group: msg.is_group,
                sender_id: msg.sender_id.clone(),
                conversation_id: msg.conversation_id.clone(),
                text: "Failed to create conversation".to_string(),
                channel: msg.channel.clone(),
                video_url: None,
                image_url: None,
                audio_url: None,
                doc_url: None,
                sticker_url: None,
                metadata: msg.metadata.clone(),
            })
            .await;

        return Ok(());
    }

    info!("begin create channels");
    if let Err(e) = sqlx::query!(
            "INSERT INTO channels (channel_type, external_id, external_chat_id, conversation_id, user_id) VALUES ($1, $2, $3, $4, $5)",
            msg.channel,
            msg.sender_id,
            msg.conversation_id,
            conv_id,
            u_id
        ).execute(&mut *tx).await {
        error!("Failed to link channel: {}", e);
        let _ = tx.rollback().await;

        let _ = state.publish_outbond(&crate::feature::OutboundMessage {
            is_group: msg.is_group,
            sender_id: msg.sender_id.clone(),
            conversation_id: msg.conversation_id.clone(),
            text: "Failed to link channel".to_string(),
            channel: msg.channel.clone(),
            video_url: None,
            image_url: None,
            audio_url: None,
            doc_url: None,
            sticker_url: None,
            metadata: msg.metadata.clone(),
        }).await;

        return Ok(());
    }

    if let Err(e) = sqlx::query!(
            "INSERT INTO conversation_members (conversation_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            conv_id,
            u_id
        ).execute(&mut *tx).await {
        error!("Failed to add member: {}", e);
        let _ = tx.rollback().await;
        let _ = state.publish_outbond(&crate::feature::OutboundMessage {
            is_group: msg.is_group,
            sender_id: msg.sender_id.clone(),
            conversation_id: msg.conversation_id.clone(),
            text: "Failed to join conversation".to_string(),
            channel: msg.channel.clone(),
            video_url: None,
            image_url: None,
            audio_url: None,
            doc_url: None,
            sticker_url: None,
            metadata: msg.metadata.clone(),
        }).await;

        return Ok(());
    }

    if let Err(e) = tx.commit().await {
        error!("Failed to commit registration: {}", e);
        let _ = state
            .publish_outbond(&OutboundMessage {
                is_group: msg.is_group,
                sender_id: msg.sender_id.clone(),
                conversation_id: msg.conversation_id.clone(),
                text: "Failed to register".to_string(),
                channel: msg.channel.clone(),
                video_url: None,
                image_url: None,
                audio_url: None,
                doc_url: None,
                sticker_url: None,
                metadata: msg.metadata.clone(),
            })
            .await;

        return Ok(());
    }

    let _ = state
        .publish_outbond(&OutboundMessage {
            is_group: msg.is_group,
            sender_id: msg.sender_id.clone(),
            conversation_id: msg.conversation_id.clone(),
            text: "Success register account, you can now /login for access dashboard".to_string(),
            channel: msg.channel.clone(),
            video_url: None,
            image_url: None,
            audio_url: None,
            doc_url: None,
            sticker_url: None,
            metadata: msg.metadata.clone(),
        })
        .await;
    Ok(())
}

pub async fn process_group_registration(
    state: &AppState,
    msg: &InboundMessage,
) -> anyhow::Result<()> {
    info!(
        "Registering group: {} on channel {}",
        msg.conversation_id, msg.channel
    );

    let mut tx = state.pool.begin().await?;
    let existing_channel =
        channel_repo::get_channel_group_info(&state.pool, &msg.channel, &msg.sender_id).await;
    if let Err(err) = existing_channel {
        info!("Only registered user can pair group:{}", err);
        let _ = state
            .publish_outbond(&OutboundMessage {
                is_group: true,
                sender_id: "nomi".to_string(),
                conversation_id: msg.conversation_id.clone(),
                text: "Whoops! 🏍️💨 Only a registered Nomi user can pair me with a group."
                    .to_string(),
                channel: msg.channel.clone(),
                video_url: None,
                image_url: None,
                audio_url: None,
                doc_url: None,
                sticker_url: None,
                metadata: msg.metadata.clone(),
            })
            .await;
        return Ok(());
    }

    let existing_channel = existing_channel?;
    if let None = existing_channel {
        info!("Only registered user can pair group");
        let _ = state
            .publish_outbond(&OutboundMessage {
                is_group: true,
                sender_id: "nomi".to_string(),
                conversation_id: msg.conversation_id.clone(),
                text: "Whoops! 🏍️💨 Only a registered Nomi user can pair me with a group. 🚀"
                    .to_string(),
                channel: msg.channel.clone(),
                video_url: None,
                image_url: None,
                audio_url: None,
                doc_url: None,
                sticker_url: None,
                metadata: msg.metadata.clone(),
            })
            .await;
        return Ok(());
    }

    if existing_channel.is_some() {
        let _ = state
            .publish_outbond(&OutboundMessage {
                is_group: true,
                sender_id: "nomi".to_string(),
                conversation_id: msg.conversation_id.clone(),
                text: "This group is already registered! 🚀".to_string(),
                channel: msg.channel.clone(),
                video_url: None,
                image_url: None,
                audio_url: None,
                doc_url: None,
                sticker_url: None,
                metadata: msg.metadata.clone(),
            })
            .await;

        let existing_group_channel = existing_channel.unwrap();
        let _ = channel_repo::link_channel_group(
            &state.pool,
            &msg.channel,
            &msg.conversation_id,
            existing_group_channel.conversation_id,
        )
        .await;
        return Ok(());
    }

    // 2. Create new conversation for this group
    let conv_id = Uuid::new_v4();
    let title = format!("Group: {} via {}", msg.conversation_id, msg.channel);

    let trx_convo = sqlx::query!(
        "INSERT INTO conversations (id, title,soul_content,bootstrap_content) VALUES ($1, $2,$3,$4) RETURNING id",
        conv_id,
        title,
        PromptRegistry::default_soul_prompts(),
        PromptRegistry::default_bootstrap_content()
    )
    .fetch_one(&mut *tx)
    .await?;

    let trx = tx.commit().await;
    if let Ok(_) = trx {
        let _ = channel_repo::link_channel_group(
            &state.pool,
            &msg.channel,
            &msg.conversation_id,
            trx_convo.id,
        )
        .await;
    }

    let _ = state
        .publish_outbond(&OutboundMessage {
            is_group: true,
            sender_id: "nomi".to_string(),
            conversation_id: msg.conversation_id.clone(),
            text: "Group registered! I'm ready to help here whenever you mention me. 🚀"
                .to_string(),
            channel: msg.channel.clone(),
            video_url: None,
            image_url: None,
            audio_url: None,
            doc_url: None,
            sticker_url: None,
            metadata: msg.metadata.clone(),
        })
        .await;

    Ok(())
}

pub async fn is_group_registered(pool: &sqlx::PgPool, external_id: &str, channel: &str) -> bool {
    sqlx::query!(
        "SELECT is_active FROM channel_group WHERE external_group_id = $1 AND channel = $2",
        external_id,
        channel
    )
    .fetch_optional(pool)
    .await
    .map(|r| r.map(|row| row.is_active.unwrap_or(true)).unwrap_or(false))
    .unwrap_or(false)
}

pub async fn process_login(state: &AppState, msg: &InboundMessage) -> anyhow::Result<()> {
    info!(
        "start login from channel {} sender_id {}",
        msg.channel, msg.sender_id
    );
    // Check if user/channel exists
    let channel_exists = sqlx::query!(
            "SELECT u.id as user_id FROM channels c JOIN users u ON u.id = c.user_id WHERE c.channel_type = $1 AND c.external_chat_id = $2",
            msg.channel,
            msg.conversation_id
        ).fetch_optional(&state.pool).await;

    if let Err(err) = channel_exists {
        info!("failed get channel data: {}", err);
        let _ = state
            .publish_outbond(&OutboundMessage {
                is_group: msg.is_group,
                sender_id: msg.sender_id.clone(),
                conversation_id: msg.conversation_id.clone(),
                text: "We having trouble for getting information, meanwhile we fixing you can try again later.".to_string(),
                channel: msg.channel.clone(),
                video_url: None,
                image_url: None,
                audio_url: None,
                doc_url: None,
                sticker_url: None,
                metadata: msg.metadata.clone(),
            })
            .await;

        return Ok(());
    }
    if let Ok(None) = channel_exists {
        info!("channel doesnt exist:");
        let _ = state
            .publish_outbond(&crate::feature::OutboundMessage {
                is_group: msg.is_group,
                sender_id: msg.sender_id.clone(),
                conversation_id: msg.conversation_id.clone(),
                text: "Channel not registered, Use /register for new user use, if you already had account, get pairing code from dashboard and use /pair <PAIRING CODE>".to_string(),
                channel: msg.channel.clone(),
                video_url: None,
                image_url: None,
                audio_url: None,
                doc_url: None,
                sticker_url: None,
                metadata: msg.metadata.clone(),
            })
            .await;

        return Ok(());
    }

    let channel_data = channel_exists.unwrap().unwrap();
    let user_id = channel_data.user_id;

    // Generate OTP
    let otp_code: u32 = rand::rng().random_range(100000..999999);
    let otp_str = otp_code.to_string();
    let redis_key = format!("otp:{}", user_id);

    if let Err(e) = state.redis.set_ex(&redis_key, &otp_str, 300).await {
        error!("Failed to store OTP in Redis: {}", e);
        let _ = state
            .publish_outbond(&crate::feature::OutboundMessage {
                is_group: msg.is_group,
                sender_id: msg.sender_id.clone(),
                conversation_id: msg.conversation_id.clone(),
                text: "Database error".to_string(),
                channel: msg.channel.clone(),
                video_url: None,
                image_url: None,
                audio_url: None,
                doc_url: None,
                sticker_url: None,
                metadata: msg.metadata.clone(),
            })
            .await;

        return Ok(());
    }

    let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
    let login_url = format!("{}/login?id={}", app_url, user_id);

    //hack: we need to sent message twice because it will sent as 2 bubble
    let outbound_text = format!(
        "Your verification code is: {}",
        otp_str
    );

    let outbound = OutboundMessage {
        is_group: msg.is_group,
        sender_id: "nomi_auth".to_string(),
        conversation_id: msg.conversation_id.clone(),
        text: outbound_text,
        channel: msg.channel.clone(),
        video_url: None,
        image_url: None,
        audio_url: None,
        doc_url: None,
        sticker_url: None,
        metadata: msg.metadata.clone(),
    };

    let outbound_text = format!(
        "Click here to login: {}",
        login_url
    );

    if let Err(e) = state.redis.publish_event("nomi:outbound", &outbound).await {
        error!("Failed to publish OTP to nomi:outbound: {}", e);
        return Ok(());
    }

    let outbound = OutboundMessage {
        is_group: msg.is_group,
        sender_id: "nomi_auth".to_string(),
        conversation_id: msg.conversation_id.clone(),
        text: outbound_text,
        channel: msg.channel.clone(),
        video_url: None,
        image_url: None,
        audio_url: None,
        doc_url: None,
        sticker_url: None,
        metadata: msg.metadata.clone(),
    };

    if let Err(e) = state.redis.publish_event("nomi:outbound", &outbound).await {
        error!("Failed to publish OTP to nomi:outbound: {}", e);
        return Ok(());
    }

    Ok(())
}

pub async fn get_help_command(
    state: &AppState, msg: &InboundMessage
)->anyhow::Result<()>{
    let message = format!(
        "Hello there! 👋 \n
            I'm **Nomi**, Trian's AI collaborator. I help him manage his projects, track his adventures on the road, and keep his digital ecosystem running smoothly. \n
            If you're a friend of Trian's, I'd love to get to know you! To get started and secure your access to our chat, could you please use one of the commands below?\n
                {} — If this is your first time chatting with me, use this to set up your profile. \n
                {} — If we've spoken before, use this to jump right back into our conversation.\n\
                {} - Ask for help\n
                {} - If you want to connected another channel whatsapp,telegram,slack etc.\n
                {} - Connect app with pairing code,The code are generated  from `/linkapp`.\n
            It’s a pleasure to meet you, and I look forward to assisting you once you're signed in! ✨\n",
        "**`/register`**",
        "**`/login`**",
        "**`/help`**",
        "**`/linkapp`**",
        "**`/pair <PAIRING CODE>`**",
    );

    let _ = state.publish_outbond(
        &OutboundMessage{
            is_group: msg.is_group,
            sender_id: "nomi".to_string(),
            conversation_id: msg.conversation_id.clone(),
            text:message,
            channel: msg.channel.clone(),
            video_url: None,
            image_url: None,
            audio_url: None,
            doc_url: None,
            sticker_url: None,
            metadata: msg.metadata.clone(),
        }
    ).await;
    Ok(())
}
