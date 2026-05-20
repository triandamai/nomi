
 ### Task: Implement Isolated Sticker Generator Plugin via NomiToolPlugin Trait


 We need to build a brand-new, self-contained tool module at `src/plugins/sticker_generator.rs` that implements our unified `NomiToolPlugin` trait. This tool will take an input image storage URL, download the raw image buffer, process and transcode it into an optimized, transparent WebP format matching WhatsApp sticker requirements, and save it back to our S3 storage bucket.
 #### ⚠️ CRITICAL AGENT RULES: ZERO-TOUCH SAFETY BOUNDARIES


 * **DO NOT modify, alter, or overwrite any existing code** inside your working `IntentClassifier`, `MediaInterpreterService`, or active network gateways.
 * **CREATE NEW FILES ONLY** (`src/plugins/sticker_generator.rs`) to maintain a 100% safe, modular refactoring boundary.
 * Ensure all network streaming, image processing, and file handling run inside thread-safe asynchronous handlers using non-blocking primitives.


 ---
 ### Step 1: Implement the Sticker Generator Plugin


 Create a brand-new file at `src/plugins/sticker_generator.rs`. Implement the `NomiToolPlugin` trait explicitly for a new `StickerGeneratorPlugin` struct:
 ```rust
 use crate::common::tools::{NomiToolPlugin, ToolDispatcher};
 use serde_json::{json, Value};
 use futures::future::BoxFuture;
 
 pub struct StickerGeneratorPlugin;
 
 impl NomiToolPlugin for StickerGeneratorPlugin {
     /// Defines the declarative JSON schema Gemini uses to understand the tool parameters
     fn schema(&self) - Value {
         json!({
             "name": "generate_sticker",
             "description": "Converts a specified message image URL or attachment file into a WhatsApp-compatible WebP sticker layout.",
             "parameters": {
                 "type": "object",
                 "properties": {
                     "image_url": {
                         "type": "string",
                         "description": "The secure S3 storage link containing the inbound source image file."
                     }
                 },
                 "required": ["image_url"]
             }
         })
     }
 
     /// Maps the unique intent strings that link back to our vector classification engine
     fn matching_intents(&self) - &[&str] {
         &["MAKE_STICKER", "CONVERT_TO_STICKER", "BIKIN_STICKER"]
     }
 
     /// The execution pipeline handler
     fn execute<'a(
         &'a self,
         dispatcher: &'a ToolDispatcher,
         args: Value
     ) - BoxFuture<'a, anyhow::Result<String {
         Box::pin(async move {
             // 1. Parameter Extraction
             let image_url = args["image_url"].as_str()
                 .ok_or_else(|| anyhow::anyhow!("Missing target image_url parameter"))?;
 
             // 2. Fetch Source Bytes
             // Use a lightweight async HTTP client client or direct S3 client hook to pull the image bytes from your bucket
             let source_bytes = reqwest::get(image_url).await?.bytes().await?;
 
             // 3. Image Processing & WebP Encoding Pass
             // Load the image bytes using a fast encoder (like the Rust native `image` crate or `webp` wrapper).
             // Resize the image to WhatsApp specifications (exactly 512x512 pixels) and encode it as a WebP byte buffer.
             let processed_webp_bytes = match self.transcode_to_sticker_webp(&source_bytes) {
                 Ok(bytes) = bytes,
                 Err(e) = return Err(anyhow::anyhow!("Failed resizing/encoding to sticker WebP: {}", e)),
             };
 
             // 4. Persistent S3 Storage Dump
             let sticker_uuid = uuid::Uuid::new_v4();
             let storage_key = format!("stickers/{}.webp", sticker_uuid);
             
             let uploaded_sticker_url = dispatcher.storage
                 .upload_bytes(processed_webp_bytes, &storage_key, "image/webp")
                 .await?;
 
             // 5. Build Final Payload Metadata
             // Format the output payload clearly so the downstream Orchestrator loop identifies this as an active media attachment event
             let success_payload = json!({
                 "status": "success",
                 "media_type": "sticker",
                 "sticker_url": uploaded_sticker_url
             });
 
             Ok(success_payload.to_string())
         })
     }
 }
 
 impl StickerGeneratorPlugin {
     /// Helper module to enforce 512x512 canvas sizing and apply clean WebP compression limits
     fn transcode_to_sticker_webp(&self, raw_data: &[u8]) - anyhow::Result<Vec<u8 {
         // Implement image decoding, 512x512 dimensional aspect cropping, and return encoded WebP vector bytes
         // Keep allocations lightweight to guard our 4GB VPS memory pools
         todo!("Implement fast 512x512 WebP resizing logic here")
     }
 }
 
 ```
---

 ### Step 2: Workspace Integration

 * Register the new plugin path file inside your plugins module tree.
 * Ensure your primary setup file instantiates `StickerGeneratorPlugin` and registers it directly into the `ToolDispatcher.plugins` map wrapper at application startup.
 * Run a workspace compile verification sweep to guarantee everything resolves cleanly across your trait boundaries.

