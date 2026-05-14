use tracing::{error, info};

/// Task 1: The Regex Auto-Fixer (Fast Path)
/// Fixes common thinking tag errors in AI responses.
pub fn heal_thinking_tags(text: &str) -> String {
    let mut healed = text.trim().to_string();

    // Pattern 1: If the text starts with "thinking" (no brackets), wrap it.
    // Case-insensitive check.
    if healed.to_lowercase().starts_with("thinking") && !healed.starts_with("<thinking>") {
        info!("Healing: Wrapping 'thinking' start with tags");
        // Find where the thinking ends and response starts.
        if let Some(pos) = healed.find("

") {
            let (thought, response) = healed.split_at(pos);
            healed = format!("<thinking>{}</thinking>{}", thought, response);
        } else {
            healed = format!("<thinking>{}</thinking>", healed);
        }
    }

    // Pattern 2: If there is a <thinking> tag but no </thinking>, append the closing tag.
    if healed.contains("<thinking>") && !healed.contains("</thinking>") {
        info!("Healing: Appending missing </thinking> tag");
        if let Some(thinking_start) = healed.find("<thinking>") {
            let content_after_start = &healed[thinking_start + 10..];
            if let Some(pos) = content_after_start.find("

") {
                let actual_pos = thinking_start + 10 + pos;
                let (thought_part, response_part) = healed.split_at(actual_pos);
                healed = format!("{}</thinking>{}", thought_part, response_part);
            } else {
                healed = format!("{}</thinking>", healed);
            }
        }
    }

    healed
}

/// Task 2: The 'Refiner' Utility (The Fixer)
/// If the output is still logically broken, send it back to Gemini for refinement.
pub async fn refine_output(
    broken_output: &str,
    gemini: &gemini_rust::Gemini,
) -> Result<String, String> {
    info!("Triggering 'Refiner' Utility for broken output");
    
    let refiner_prompt = format!(
        r#"The following output is improperly formatted. Extract the "thinking" and "response" sections and return them in valid XML tags. 

Output: 

{}

Format your response exactly as: 

<thinking>... reasoning ...</thinking>
Actual response to the user here."#,
        broken_output
    );

    match gemini
        .generate_content()
        .with_message(gemini_rust::Message {
            role: gemini_rust::Role::User,
            content: gemini_rust::Content {
                parts: Some(vec![gemini_rust::Part::Text {
                    text: refiner_prompt,
                    thought: None,
                    thought_signature: None,
                }]),
                role: Some(gemini_rust::Role::User),
            },
        })
        .with_max_output_tokens(2048)
        .with_temperature(1.0) // Tiny, high-temperature prompt
        .execute()
        .await
    {
        Ok(res) => {
            let text = res.text();
            info!("Refiner successfully fixed the output");
            Ok(text)
        }
        Err(e) => {
            error!("Refiner failed: {}", e);
            Err(e.to_string())
        }
    }
}
