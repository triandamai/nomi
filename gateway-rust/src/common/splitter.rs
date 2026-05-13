use regex::Regex;

/// Splits a message into multiple bubbles based on double newlines.
/// Also performs deduplication of common introductory phrases.
pub fn split_into_bubbles(text: &str) -> Vec<String> {
    if text.trim().is_empty() {
        return Vec::new();
    }

    // 1. Split by double newlines
    let bubbles: Vec<String> = text
        .split("\n\n")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if bubbles.is_empty() {
        return Vec::new();
    }

    // 2. Deduplication logic
    let mut final_bubbles = Vec::new();
    let mut seen_intro = std::collections::HashSet::new();

    // Regex for common intro phrases like "Alright, Trian!", "Got it!", "Hey Trian! ✨"
    // Usually followed by some punctuation and maybe an emoji.
    let intro_re = Regex::new(r"^(?i)(Alright|Got it|Hey|Hi|Okay|Sure),?\s+([a-zA-Z]+)?[!.]?\s*[\p{Extended_Pictographic}]*").unwrap();

    for bubble in bubbles {
        let mut processed_bubble = bubble.clone();
        
        // Check if the bubble starts with an intro phrase we've seen before
        if let Some(mat) = intro_re.find(&bubble) {
            let intro = mat.as_str().trim().to_lowercase();
            if seen_intro.contains(&intro) {
                // Strip the intro and leading whitespace/newlines
                processed_bubble = bubble[mat.end()..].trim().to_string();
            } else {
                seen_intro.insert(intro);
            }
        }

        if !processed_bubble.is_empty() {
            final_bubbles.push(processed_bubble);
        }
    }

    final_bubbles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_simple() {
        let input = "Bubble 1\n\nBubble 2";
        let result = split_into_bubbles(input);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "Bubble 1");
        assert_eq!(result[1], "Bubble 2");
    }

    #[test]
    fn test_deduplicate_intro() {
        let input = "Alright, Trian! I'll look into that.\n\nAlright, Trian! Here is what I found.";
        let result = split_into_bubbles(input);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "Alright, Trian! I'll look into that.");
        assert_eq!(result[1], "Here is what I found.");
    }
    
    #[test]
    fn test_deduplicate_with_emoji() {
        let input = "Got it! ✨ I'm on it.\n\nGot it! ✨ Here's the result.";
        let result = split_into_bubbles(input);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "Got it! ✨ I'm on it.");
        assert_eq!(result[1], "Here's the result.");
    }
}
