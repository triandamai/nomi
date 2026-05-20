use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::tools_model::ReadWorkSpaceParameters;
use crate::common::tools::ToolDispatcher;
use futures::future::{BoxFuture, FutureExt};
use gemini_rust::FunctionDeclaration;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use tracing::info;

pub struct ReadWorkspaceFilePlugin;

impl NomiToolPlugin for ReadWorkspaceFilePlugin {
    fn schema(&self) -> Value {
        serde_json::to_value(
            FunctionDeclaration::new(
                "read_workspace_file",
                "Read content of file in workspace",
                None,
            )
            .with_parameters::<ReadWorkSpaceParameters>(),
        )
        .unwrap()
    }

    fn rules(&self) -> &str {
        ""
    }

    fn matching_intents(&self) -> &[&str] {
        &["READ_FILE", "WORKSPACE_INQUIRY", "STORAGE", "FULL_REGISTRY"]
    }

    fn execute<'a>(
        &'a self,
        dispatcher: &'a ToolDispatcher,
        args: Value,
    ) -> BoxFuture<'a, anyhow::Result<String>> {
        async move {
            let params: ReadWorkSpaceParameters = serde_json::from_value(args)?;
            info!(path = %params.path, "Executing read_workspace_file via plugin");

            let requested_path = PathBuf::from(&params.path);

            if requested_path.is_absolute() || params.path.contains("..") {
                return Ok("Error: Access denied. Only relative paths within the workspace are allowed.".to_string());
            }

            let full_path = dispatcher.workspace_root.join(requested_path);

            match fs::read_to_string(full_path) {
                Ok(result) => Ok(result),
                Err(error) => Ok(format!("Error reading file: {}", error)),
            }
        }
        .boxed()
    }
}
