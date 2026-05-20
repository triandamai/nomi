use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::ToolDispatcher;
use serde_json::{json, Value};
use futures::future::{BoxFuture, FutureExt};
use tracing::info;
use image::GenericImageView;
use webp::Encoder;
use crate::common::repository::message_repo::get_latest_unprocessed_media;

pub struct StickerGeneratorPlugin;

impl NomiToolPlugin for StickerGeneratorPlugin {
    fn schema(&self) -> Value {
        json!({
            "name": "create_sticker",
            "description": "Converts an image from a URL or the latest unprocessed image in history into a WhatsApp-compatible WebP sticker (512x512).",
            "parameters": {
                "type": "object",
                "properties": {
                    "image_url": {
                        "type": "string",
                        "description": "The URL of the source image. If omitted, the latest unprocessed image in the chat will be used."
                    }
                }
            }
        })
    }

    fn rules(&self) -> &str {
        "### STICKER LOGIC\n- Use `create_sticker` tool to convert images into stickers. USE THE NATIVE TOOL CALL API.\n- If there is an image in the history or current context, pass its URL to `create_sticker`.\n"
    }

    fn matching_intents(&self) -> &[&str] {
        &["MAKE_STICKER", "CONVERT_TO_STICKER", "IMAGE_TO_STICKER", "STICKER"]
    }

    fn execute<'a>(
        &'a self,
        dispatcher: &'a ToolDispatcher,
        args: Value
    ) -> BoxFuture<'a, anyhow::Result<String>> {
        async move {
            let conversation_id = dispatcher.conversation_id
                .ok_or_else(|| anyhow::anyhow!("No active conversation context"))?;

            // 1. Resolve Image URL
            let image_url = if let Some(url) = args["image_url"].as_str() {
                url.to_string()
            } else {
                // Fallback to latest unprocessed media from history
                let latest = get_latest_unprocessed_media(&dispatcher.pool, conversation_id).await?;
                match latest {
                    Some((url, media_type)) if media_type == "image" || media_type == "sticker" => url,
                    _ => return Ok("No recent image found to convert into a sticker. Please upload an image first! 🖼️".to_string()),
                }
            };

            info!("StickerGenerator: Processing image from {}", image_url);

            // 2. Fetch Source Bytes via StorageClient (internal path or full URL)
            let base_url = dotenvy::var("PUBLIC_GATEWAY_URL").unwrap_or("http://localhost:8000/api".to_string());
            let file_path = if image_url.starts_with("http") && image_url.contains(&base_url) {
                image_url.replace(&format!("{}/files/", base_url), "")
            } else {
                image_url.clone()
            };

            let source_data = dispatcher.app_state.storage.get_file("conversations".to_string(), file_path.clone()).await
                .map_err(|e| anyhow::anyhow!("Failed to download source image: {}", e))?;

            // 3. Image Processing & WebP Encoding
            let processed_webp_bytes = self.transcode_to_sticker_webp(&source_data.to_vec())?;

            // 4. Save to S3
            let sticker_uuid = uuid::Uuid::new_v4();
            let storage_key = format!("stickers/{}.webp", sticker_uuid);
            
            let uploaded_path = dispatcher.app_state.storage
                .upload_byte("conversations".to_string(), storage_key.clone(), processed_webp_bytes)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to upload sticker to storage: {}", e))?;

            let full_url = dispatcher.app_state.storage.get_full_url(&uploaded_path);

            // 5. Mark source as processed
            let _ = crate::common::repository::message_repo::mark_last_media_processed(&dispatcher.pool, conversation_id).await;

            // 6. Build Success Payload
            let success_payload = json!({
                "status": "success",
                "media_type": "sticker",
                "sticker_url": full_url,
                "message": "Sticker generated successfully! 🚀"
            });

            Ok(success_payload.to_string())
        }.boxed()
    }
}

impl StickerGeneratorPlugin {
    /// Resizes image to exactly 512x512 and encodes as WebP
    fn transcode_to_sticker_webp(&self, raw_data: &[u8]) -> anyhow::Result<Vec<u8>> {
        // Load image
        let img = image::load_from_memory(raw_data)
            .map_err(|e| anyhow::anyhow!("Failed to decode image: {}", e))?;

        // 512x512 is the WhatsApp standard
        let size: u32 = 512;
        
        // Use fit to maintain aspect ratio and then center on a transparent 512x512 canvas
        let scaled = img.resize(size, size, image::imageops::FilterType::Lanczos3);
        let (sw, sh) = scaled.dimensions();
        
        let mut canvas = image::ImageBuffer::new(size, size);
        // Fill with transparency (RGBA 0,0,0,0) - already zeroed by default
        
        let x_offset = (size - sw) / 2;
        let y_offset = (size - sh) / 2;
        
        image::imageops::overlay(&mut canvas, &scaled, x_offset as i64, y_offset as i64);

        // Encode as WebP
        let dynamic_canvas = image::DynamicImage::ImageRgba8(canvas);
        let encoder = Encoder::from_image(&dynamic_canvas)
            .map_err(|e| anyhow::anyhow!("WebP Encoder error: {}", e))?;
        
        let webp_data = encoder.encode(80.0); // 80 quality is good for stickers
        
        Ok(webp_data.to_vec())
    }
}
