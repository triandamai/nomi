use gemini_rust::UsageMetadata;
fn main() {
    let metadata = UsageMetadata {
        prompt_token_count: 0,
        candidates_token_count: 0,
        total_token_count: 0,
        prompt_tokens_details: vec![],
        candidates_tokens_details: vec![],
    };
}
