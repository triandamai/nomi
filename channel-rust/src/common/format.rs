pub fn markdown_to_whatsapp(text: &str) -> String {
    let mut result = text.to_string();
    
    // Bold: **text** -> *text*
    // We use a simple regex-like approach or just direct replacement if we're careful.
    // Note: This is a basic implementation.
    
    // Bold
    result = replace_bold(&result);
    
    // Italic: _text_ or *text* -> _text_
    // LLM often uses _text_ for italics which WA also uses.
    
    // Monospace: `text` -> ```text```
    result = replace_monospace(&result);

    result
}

fn replace_bold(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    let mut inside = false;
    
    while let Some(c) = chars.next() {
        if c == '*' && chars.peek() == Some(&'*') {
            chars.next(); // consume second *
            out.push('*');
            inside = !inside;
        } else {
            out.push(c);
        }
    }
    out
}

fn replace_monospace(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    let mut inside = false;
    
    while let Some(c) = chars.next() {
        if c == '`' {
            if inside {
                out.push_str("```");
            } else {
                out.push_str("```");
            }
            inside = !inside;
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bold() {
        assert_eq!(markdown_to_whatsapp("Hello **world**"), "Hello *world*");
    }

    #[test]
    fn test_monospace() {
        assert_eq!(markdown_to_whatsapp("Use `code`"), "Use ```code```");
    }
}
