use crate::common::tools::ToolDispatcher;
use crate::services::ambient_soul::TokenMetrics;
use anyhow::{anyhow, Result};
use gemini_rust::{Part, Blob, Message, Role, Content};
use tracing::info;
use base64::Engine;

pub struct MediaInterpreterService;

impl MediaInterpreterService {
    pub async fn hydrate_media_context_string(
        &self,
        dispatcher: &ToolDispatcher,
        incoming_text_with_links: &str,
        mime_type: &str,
    ) -> Result<(String, TokenMetrics)> {
        info!("MediaInterpreter: Hydrating context for mime_type: {}", mime_type);

        // 1. URL Detection & Parsing
        let s3_path = incoming_text_with_links.trim();
        
        // 2. Download from S3
        let bytes = self.download_from_s3(dispatcher, s3_path).await?;
        let base64_data = base64::engine::general_purpose::STANDARD.encode(bytes);

        // 3. Gemini Multimodal Inference
        let system_prompt = if mime_type.starts_with("image/") {
            "Analyze this image block completely. Extract all OCR text strings, transactional amounts, code exceptions, visible items, or environmental scenery. Summarize the content succinctly inside bracket markers."
        } else if mime_type.starts_with("audio/") {
            "Transcribe the exact vocal spoken wording inside this clip cleanly into text."
        } else {
            "Analyze this media file and provide a concise summary of its contents."
        };

        let res = dispatcher.gemini.generate_content()
            .with_system_prompt(system_prompt)
            .with_message(Message {
                role: Role::User,
                content: Content {
                    role: Some(Role::User),
                    parts: Some(vec![
                        Part::InlineData {
                            inline_data: Blob {
                                mime_type: mime_type.to_string(),
                                data: base64_data,
                            },
                            media_resolution: None,
                        }
                    ]),
                }
            })
            .execute()
            .await?;

        let mut metrics = TokenMetrics::default();
        if let Some(usage) = &res.usage_metadata {
            metrics.input_tokens = usage.prompt_token_count.unwrap_or(0) as u32;
            metrics.output_tokens = usage.candidates_token_count.unwrap_or(0) as u32;
            metrics.total_tokens = usage.total_token_count.unwrap_or(0) as u32;
        }

        let description = res.text();
        let description_trimmed = description.trim();
        
        // 4. Token Usage Logging (Parallel/Non-blocking)
        let pool_clone = dispatcher.pool.clone();
        let conv_id = dispatcher.conversation_id;
        let user_id = dispatcher.user_id;
        let i_tokens = metrics.input_tokens as i64;
        let o_tokens = metrics.output_tokens as i64;
        let t_tokens = metrics.total_tokens as i64;

        tokio::spawn(async move {
            let _ = crate::services::ambient_soul::AmbientSoulService::log_token_transaction(
                &pool_clone,
                conv_id,
                None,
                user_id,
                "knowledge",
                "system",
                i_tokens,
                o_tokens,
                t_tokens,
            ).await;
        });

        // 5. Synthesize Result
        let hydrated_text = format!("[Media Context Description: {}] {}", description_trimmed, incoming_text_with_links);
        
        Ok((hydrated_text, metrics))
    }

    async fn download_from_s3(&self, dispatcher: &ToolDispatcher, full_path: &str) -> Result<Vec<u8>> {
        // Use the existing StorageClient from AppState
        let storage = &dispatcher.app_state.storage;
        
        // Extract bucket and key if format is "bucket/key", otherwise default to "conversations"
        let (bucket, key) = if full_path.contains('/') && !full_path.starts_with('/') {
            let parts: Vec<&str> = full_path.splitn(2, '/').collect();
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("conversations".to_string(), full_path.trim_start_matches('/').to_string())
        };

        info!("MediaInterpreter: Downloading from bucket: {}, key: {}", bucket, key);

        let data = storage.get_file(bucket, key).await
            .map_err(|e| anyhow!("Storage error: {}", e))?;

        Ok(data.to_vec())
    }
}
