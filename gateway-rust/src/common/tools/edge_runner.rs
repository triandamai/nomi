use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

pub struct BunEdgeExecutor {
    pub slug: String,
    pub script_code: String,
}

impl BunEdgeExecutor {
    pub async fn run(
        &self, 
        payload: serde_json::Value, 
        incoming: serde_json::Value,
        workspace: serde_json::Value,
        bridge_token: &str,
        api_base_url: &str,
    ) -> anyhow::Result<String> {
        let unified_script = format!(
            "const NomiArgs = {{\n  incoming: {},\n  payload: {},\n  workspace: {}\n}};\nconst BRIDGE_TOKEN = '{}';\nconst API_BASE_URL = '{}';\n\n\
            /** Built-in: Semantic Knowledge Retrieval */\n\
            async function retrieve_knowledge(query, limit = 5) {{\n  \
                const res = await fetch(`${{API_BASE_URL}}/internal/rpc/retrieve-knowledge`, {{\n    \
                    method: 'POST',\n    \
                    headers: {{ 'Content-Type': 'application/json', 'X-Bridge-Token': BRIDGE_TOKEN }},\n    \
                    body: JSON.stringify({{ query, limit }})\n  \
                }});\n  \
                return await res.json();\n\
            }}\n\n\
            {}\n\n\
            // Nomi execution wrapper\n\
            (async () => {{\n    \
                try {{\n        \
                    const result = await run(NomiArgs);\n        \
                    console.log(typeof result === 'object' ? JSON.stringify(result, null, 2) : String(result));\n    \
                }} catch (e) {{\n        \
                    console.error(e);\n        \
                    process.exit(1);\n    \
                }}\n\
            }})();",
            incoming.to_string(),
            payload.to_string(),
            workspace.to_string(),
            bridge_token,
            api_base_url,
            self.script_code
        );

        // Bun doesn't support -e reliably, so we write to a temp file
        let temp_dir = std::env::temp_dir();
        let file_name = format!("nomi_edge_{}_{}.ts", self.slug, uuid::Uuid::new_v4());
        let file_path = temp_dir.join(file_name);

        tokio::fs::write(&file_path, &unified_script).await?;

        let mut child = Command::new("bun")
            .args(["run", file_path.to_str().unwrap()])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .or_else(|_| {
                Command::new("npx")
                    .args(["bun", "run", file_path.to_str().unwrap()])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
            })
            .map_err(|e| anyhow::anyhow!("Container lacked Bun system capabilities: {}", e))?;

        let result = match timeout(Duration::from_secs(5), child.wait_with_output()).await {
            Ok(Ok(output)) => {
                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
                } else {
                    let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    Err(anyhow::anyhow!("TypeScript Runtime Exception:\n{}", err))
                }
            }
            Ok(Err(e)) => Err(anyhow::anyhow!("Subprocess execution failed: {}", e)),
            Err(_) => {
                Err(anyhow::anyhow!("Execution timed out. Process killed to protect server memory limits."))
            }
        };

        // Clean up temp file
        let _ = tokio::fs::remove_file(&file_path).await;

        result
    }
}

