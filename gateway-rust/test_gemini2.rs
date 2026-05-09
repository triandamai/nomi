use gemini_rust::{Content, Part, Message, Role, tools::FunctionResponse};

fn test() {
    let call_parts = vec![Part::FunctionCall {
        function_call: gemini_rust::tools::FunctionCall {
            name: "test".to_string(),
            args: serde_json::json!({}),
        },
        thought_signature: None,
    }];
    let content = Content {
        parts: Some(call_parts),
        role: Some(Role::Model),
    };
    let _ = Message {
        content,
        role: Role::Model,
    };
}
