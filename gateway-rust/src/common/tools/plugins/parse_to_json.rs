use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::tools_model::ParseToJsonParameters;
use crate::common::tools::ToolDispatcher;
use futures::future::{BoxFuture, FutureExt};
use gemini_rust::FunctionDeclaration;
use serde_json::Value;

pub struct ParseStringToJsonPlugin;

impl NomiToolPlugin for ParseStringToJsonPlugin {
    fn schema(&self) -> Value {
        serde_json::to_value(
            FunctionDeclaration::new(
                "parse_to_json",
                "Parses a string into a JSON object. Use this to ensure structured data outputs.",
                None,
            )
            .with_parameters::<ParseToJsonParameters>(),
        )
        .unwrap()
    }

    fn rules(&self) -> &str {
        ""
    }

    fn matching_intents(&self) -> &[&str] {
        &["PARSE_JSON", "STRUCTURED_DATA", "FULL_REGISTRY"]
    }

    fn execute<'a>(
        &'a self,
        _dispatcher: &'a ToolDispatcher,
        _args: Value,
    ) -> BoxFuture<'a, anyhow::Result<String>> {
        async move {
            // This tool is currently a placeholder in the legacy dispatcher
            Ok("".to_string())
        }
        .boxed()
    }
}
