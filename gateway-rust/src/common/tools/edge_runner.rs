use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{Duration, timeout};

pub struct BunEdgeExecutor {
    pub slug: String,
    pub script_code: String,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeExecutionResult {
    pub logs: String,
    pub result: String,
}

impl BunEdgeExecutor {
    fn build_runtime_template(
        &self,
        incoming: &serde_json::Value,
        payload: &serde_json::Value,
        workspace: &serde_json::Value,
        bridge_token: &str,
        api_base_url: &str,
    ) -> String {
        format!(
            r#"
const NomiArgs = {{
    incoming: {incoming},
    payload: {payload},
    workspace: {workspace}
}};
const BRIDGE_TOKEN = '{bridge_token}';
const API_BASE_URL = '{api_base_url}';

/** Built-in: Semantic Knowledge Retrieval */
async function retrieve_knowledge(query, limit = 5) {{
    try{{
        const res = await fetch(`${{API_BASE_URL}}/api/internal/rpc/retrieve-knowledge`, {{
            method: 'POST',
            headers: {{
                'Content-Type': 'application/json',
                'X-Bridge-Token': BRIDGE_TOKEN
            }},
            body: JSON.stringify({{ query, limit }})
        }});
        if(res.status <= 200 && res.status >= 209){{
           return {{results:[]}}
        }}
        return await res.json();
    }}catch(e){{
        return {{results:[]}}
    }}
}}

{script_code}

// Nomi execution wrapper
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
            incoming = incoming.to_string(),
            payload = payload.to_string(),
            workspace = workspace.to_string(),
            bridge_token = bridge_token,
            api_base_url = api_base_url,
            script_code = self.script_code
        )
    }

    pub async fn run(
        &self,
        payload: serde_json::Value,
        incoming: serde_json::Value,
        workspace: serde_json::Value,
        bridge_token: &str,
        api_base_url: &str,
    ) -> anyhow::Result<EdgeExecutionResult> {
        let result_marker_start = "___NOMI_RESULT_START___";
        let result_marker_end = "___NOMI_RESULT_END___";

        let unified_script = self.build_runtime_template(
            &incoming,
            &payload,
            &workspace,
            bridge_token,
            api_base_url,
        );

        // Bun doesn't support -e reliably, so we write to a temp file
        let temp_dir = std::env::temp_dir();
        let file_name = format!("nomi_edge_{}_{}.ts", self.slug, uuid::Uuid::new_v4());
        let file_path = temp_dir.join(file_name);

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
            Err(_) => {
                Err(anyhow::anyhow!(
                    "Execution timed out. Process killed to protect server memory limits."
                ))
            },
        };



        // Clean up temp file
        let _ = tokio::fs::remove_file(&file_path).await;
        exec_res
    }
}
