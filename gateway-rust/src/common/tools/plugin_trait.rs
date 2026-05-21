use crate::common::tools::ToolDispatcher;
use serde_json::Value;
use futures::future::BoxFuture;

pub trait NomiToolPlugin: Send + Sync {
    fn schema(&self) -> Value;
    fn rules(&self) -> &str;
    fn matching_intents(&self) -> &[&str];
    fn execute<'a>(&'a self, dispatcher: &'a ToolDispatcher, args: Value) -> BoxFuture<'a, anyhow::Result<String>>;
}

pub trait NomiDynamicPlugin: Send + Sync {
    fn slug(&self) -> &str;
    fn schema(&self) -> Value;
    fn rules(&self) -> &str;
    fn version(&self) -> i32;
    fn execute<'a>(&'a self, args: Value) -> BoxFuture<'a, anyhow::Result<String>>;
}
