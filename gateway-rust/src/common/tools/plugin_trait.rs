use crate::common::tools::ToolDispatcher;
use crate::common::tools::tools_model::ToolResult;
use serde_json::Value;
use futures::future::BoxFuture;

pub trait NomiToolPlugin: Send + Sync {
    fn schema(&self) -> Value;
    fn rules(&self) -> &str;
    fn matching_intents(&self) -> &[&str];
    fn execute<'a>(&'a self, dispatcher: &'a ToolDispatcher, args: Value) -> BoxFuture<'a, anyhow::Result<ToolResult>>;
}
