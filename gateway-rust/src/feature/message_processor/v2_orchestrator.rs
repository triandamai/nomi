use crate::AppState;
use crate::common::identity::UserIdentity;
use crate::common::repository::message_repo::save_message;
use crate::feature::message_processor::v2_agent_orchestrator::V2AgentOrchestrator;
use crate::feature::{ UnifiedMessage};
use crate::models::Conversation;
use serde_json::json;
use tracing::info;
use uuid::Uuid;

pub async fn process_v2_message(
    state: AppState,
    convo: Conversation,
    msg: UnifiedMessage,
) -> anyhow::Result<()> {
    let conversation_id = msg.conversation_id;
    let text_content = msg.text_content.clone();

    info!(
        conversation_id = %conversation_id,
        user_id = ?msg.user_id,
        source = ?msg.source,
        "Processing unified message v2"
    );

    // 0. Silent Media Buffer: Media with EMPTY text and NO trigger
    let has_media = msg.image_url.is_some()
        || msg.video_url.is_some()
        || msg.audio_url.is_some()
        || msg.doc_url.is_some()
        || msg.sticker_url.is_some();

    let text_trimmed = text_content.trim();
    let has_only_trigger = text_trimmed.to_lowercase() == "nom" || text_trimmed == "/cmd";
    let is_skip = text_trimmed.eq_ignore_ascii_case("skip");

    let members = sqlx::query!(
        "SELECT m.user_id FROM conversation_members as m WHERE m.conversation_id = $1",
        conversation_id
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or(Vec::new());

    if has_media && text_trimmed.is_empty() {
        let media_url = msg
            .image_url
            .as_ref()
            .or(msg.video_url.as_ref())
            .or(msg.audio_url.as_ref())
            .or(msg.doc_url.as_ref())
            .or(msg.sticker_url.as_ref())
            .unwrap();

        let media_type = if msg.image_url.is_some() {
            "image"
        } else if msg.video_url.is_some() {
            "video"
        } else if msg.audio_url.is_some() {
            "audio"
        } else if msg.doc_url.is_some() {
            "document"
        } else {
            "sticker"
        };

        // 1. Save to messages table for history
        let save_user_message = save_message(
            &state.pool,
            conversation_id,
            "user",
            "",
            None,
            msg.user_id,
            0,
            0,
            0,
            msg.image_url.clone(),
            msg.video_url.clone(),
            msg.audio_url.clone(),
            msg.doc_url.clone(),
            msg.sticker_url.clone(),
        )
        .await;

        if let Ok(saved_message) = save_user_message {
            // 2. Broadcast to members via SSE
            for member in members {
                let _ = state.send_sse_to_user(
                    member.user_id.to_string().as_str(),
                    "message",
                    json!({
                        "id": saved_message.id,
                        "conversation_id": conversation_id,
                        "role": saved_message.role,
                        "content": saved_message.content.clone(),
                        "thought": saved_message.thought,
                        "user_id": saved_message.user_id,
                        "total_tokens": 0,
                        "image_url": saved_message.image_url.as_ref().map(|path| state.storage.get_full_url(path).to_string()),
                        "video_url": saved_message.video_url.as_ref().map(|path| state.storage.get_full_url(path).to_string()),
                        "audio_url": saved_message.audio_url.as_ref().map(|path| state.storage.get_full_url(path).to_string()),
                        "document_url": saved_message.document_url.as_ref().map(|path| state.storage.get_full_url(path).to_string()),
                        "sticker_url": saved_message.sticker_url.as_ref().map(|path| state.storage.get_full_url(path).to_string()),
                        "created_at": saved_message.created_at
                    })).await;
            }
        }

        // 3. Save to pending_media table for Media Checkpoint System
        let _ = crate::common::repository::pending_media_repo::upsert_pending_media(
            &state.pool,
            conversation_id,
            media_url,
            media_type,
            None,
        )
        .await;

        info!("[Orchestrator] 🖼️ Media saved and buffered silently. No LLM turn triggered.");

        // STRICT ACTION: Terminate the orchestrator execution loop immediately for this event.
        return Ok(());
    }

    let orchestrator = V2AgentOrchestrator::new(
        state.clone(),
        Some(convo),
        Some(UserIdentity {
            id: msg.user_id.unwrap_or(Uuid::nil()),
            display_name: "".to_string(),
        }),
        members.iter().map(|v| v.user_id.clone()).collect(),
    );

    if has_media && has_only_trigger {
        let media_url = msg
            .image_url
            .as_ref()
            .or(msg.video_url.as_ref())
            .or(msg.audio_url.as_ref())
            .or(msg.doc_url.as_ref())
            .or(msg.sticker_url.as_ref())
            .unwrap();

        let media_type = if msg.image_url.is_some() {
            "image"
        } else if msg.video_url.is_some() {
            "video"
        } else if msg.audio_url.is_some() {
            "audio"
        } else if msg.doc_url.is_some() {
            "document"
        } else {
            "sticker"
        };

        info!(
            "Triggered media clarification (poke detected) for {}: {}",
            media_type, media_url
        );

        // Save to pending_media
        let _ = crate::common::repository::pending_media_repo::upsert_pending_media(
            &state.pool,
            conversation_id,
            media_url,
            media_type,
            None,
        )
        .await;

        let injected_system_prompt =
            crate::prompts::PromptRegistry::zero_intent_clarification().to_string();

        orchestrator
            .process_v2_message_with_intent(
                state.clone(),
                msg,
                format!("[User poked about this {}]", media_type),
                Some(injected_system_prompt),
            )
            .await
    } else if has_media && !is_skip {
        // Task 1: If message.text is NOT empty: Do not ask for clarification.
        // Combine the text and the image into a single multi-part prompt for Gemini.
        let injected_system_prompt =
            crate::prompts::PromptRegistry::media_with_text_instruction().to_string();
        orchestrator
            .process_v2_message_with_intent(
                state.clone(),
                msg,
                text_content,
                Some(injected_system_prompt),
            )
            .await
    } else {
        orchestrator
            .process_v2_message_with_intent(state.clone(), msg, text_content, None)
            .await
    }
}