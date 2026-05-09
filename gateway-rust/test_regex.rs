fn strip_thinking_tags(text: &str) -> String {
    let re = regex::Regex::new(r"(?s)<thinking>.*?(?:</thinking>|$)").unwrap();
    re.replace_all(text, "").trim().to_string()
}

fn main() {
    let text = "<thinking>\nstuff\n</thinking>\nHello world";
    println!("{}", strip_thinking_tags(text));
    
    let text2 = "<thinking>\nunclosed thought\n";
    println!("'{}'", strip_thinking_tags(text2));
}
