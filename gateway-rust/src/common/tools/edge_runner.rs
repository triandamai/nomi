use log::info;
use std::collections::HashMap;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{Duration, timeout};

pub struct BunEdgeExecutor {
    pub slug: String,
    pub script_code: String,
}

use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeExecutionResult {
    pub logs: String,
    pub result: String,
}

impl BunEdgeExecutor {
    fn build_runtime_template(&self, internal_code: &str) -> String {
        format!(
            r#"
/** Built-in: Internal Code */
{internal_code}
{script_code}
/** Nomi Execution Runner */
(async () => {{
    try {{
        const result = await run(NomiArgs);
        process.stdout.write('\n___NOMI_RESULT_START___');
        process.stdout.write(typeof result === 'object' ? JSON.stringify(result) : String(result));
        process.stdout.write('___NOMI_RESULT_END___');
    }} catch (e) {{
        console.error(e);
        process.exit(1);
    }}
}})();
"#,
            internal_code = internal_code,
            script_code = self.script_code
        )
    }

    async fn build_internal_code(
        &self,
        bridge_token: &str,
        api_base_url: &str,
        payload: serde_json::Value,
        incoming: serde_json::Value,
        workspace: serde_json::Value,
        env: HashMap<String, String>,
    ) -> anyhow::Result<String> {
        let internal_code = tokio::fs::read_to_string("./docs/internal_rpc.ts").await;

        if let Err(err) = internal_code {
            info!("Failed to read internal code: {}", err);
            return Err(anyhow::anyhow!("Internal Error: {}", err));
        }

        let internal_code = internal_code?;
        let internal_code =
            internal_code.replace("__BASE_URL__", format!("\"{}\"", api_base_url).as_str());
        let internal_code =
            internal_code.replace("__token__", format!("\"{}\"", bridge_token).as_str());
        let internal_code = internal_code.replace("__incoming__", format!("{}", incoming).as_str());
        let internal_code = internal_code.replace("__payload__", format!("{}", payload).as_str());
        let internal_code =
            internal_code.replace("__workspace__", format!("{}", workspace).as_str());
        let internal_code = internal_code.replace("__env__", format!("{}", json!(env)).as_str());

        Ok(format!(
            r#"
            const NomiArgs = {{
                incoming: {incoming},
                payload: {payload},
                workspace: {workspace}
            }};
            
            {internal_code}
            "#,
            internal_code = internal_code,
            payload = payload,
            incoming = incoming,
            workspace = workspace,
        ))
    }

    pub async fn run(
        &self,
        bridge_token: &str,
        api_base_url: &str,
        payload: serde_json::Value,
        incoming: serde_json::Value,
        workspace: serde_json::Value,
        env: HashMap<String, String>,
    ) -> anyhow::Result<EdgeExecutionResult> {
        let result_marker_start = "___NOMI_RESULT_START___";
        let result_marker_end = "___NOMI_RESULT_END___";

        // Bun doesn't support -e reliably, so we write to a temp file
        let temp_dir = std::env::temp_dir();
        let file_name = format!("nomi_edge_{}_{}.ts", self.slug, uuid::Uuid::new_v4());
        let file_path = temp_dir.join(file_name);

        let internal_code = self
            .build_internal_code(
                bridge_token,
                api_base_url,
                payload,
                incoming,
                workspace,
                env,
            )
            .await;
        if let Err(err) = &internal_code {
            info!("Internal Error: {}", err);
            return Err(anyhow::anyhow!("Internal Error: {}", err));
        }
        let internal_code = internal_code?;
        let unified_script = self.build_runtime_template(internal_code.as_str());

        tokio::fs::write(&file_path, &unified_script).await?;

        let child = Command::new("bun")
            .args(["run", file_path.to_str().unwrap()])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .or_else(|_| {
                Command::new("npx")
                    .args(["bun", "run", file_path.to_str().unwrap()])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .kill_on_drop(true)
                    .spawn()
            })
            .map_err(|e| anyhow::anyhow!("Container lacked Bun system capabilities: {}", e))?;

        let exec_res = match timeout(Duration::from_secs(10), child.wait_with_output()).await {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                if output.status.success() {
                    // Extract result between markers
                    if let Some(start_idx) = stdout.find(result_marker_start) {
                        let result_part = &stdout[start_idx + result_marker_start.len()..];
                        if let Some(end_idx) = result_part.find(result_marker_end) {
                            let result = result_part[..end_idx].trim().to_string();
                            let logs = format!("{}{}", &stdout[..start_idx], stderr)
                                .trim()
                                .to_string();

                            Ok(EdgeExecutionResult { logs, result })
                        } else {
                            Err(anyhow::anyhow!(
                                "Malformed execution output: missing end marker"
                            ))
                        }
                    } else {
                        Ok(EdgeExecutionResult {
                            logs: format!("{}{}", stdout, stderr).trim().to_string(),
                            result: "".to_string(),
                        })
                    }
                } else {
                    Err(anyhow::anyhow!("TypeScript Runtime Exception:\n{}", stderr))
                }
            }
            Ok(Err(e)) => Err(anyhow::anyhow!("Subprocess execution failed: {}", e)),
            Err(_) => Err(anyhow::anyhow!(
                "Execution timed out. Process killed to protect server memory limits."
            )),
        };

        // Clean up temp file
        let _ = tokio::fs::remove_file(&file_path).await;
        exec_res
    }
}
