# Master Architectural Blueprint: Serverless TypeScript Dynamic Plugin Engine

## 🚨 Critical Agent Constraints & Safety Guardrails

* **DO NOT overwrite or alter existing static production code files or EMQX messaging channels.** All existing infrastructure is running smoothly and must remain completely untouched.
* **MEMORY CONSERVATION:** Every subprocess allocation must be strictly bound by operating system execution timeouts to protect resource limits.
* **STATELESS SIMPLICITY:** The Rust gateway must act as a stateless REST executor. Avoid complex cross-thread memory registration tracking blocks; read configuration definitions dynamically from the database or handle them directly via the incoming REST request payload.
* **INFRASTRUCTURE INTEGRITY:** Preserve the Alpine Linux base, `SQLX_OFFLINE=true` build steps, and migration copying exactly as defined in the target Dockerfile.

---

## 🏗️ Phase 1: Gateway Core, REST Routing & The Execution Runtime (Rust / Axum)

### Step 1.1: Database Schema Migration & Core Data Structs

Create a database migration and a Rust data layout that supports versioning, slugs, and intent classification embeddings:

1. **Database Schema (`edge_functions`):**
```sql
CREATE TABLE edge_functions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug VARCHAR(255) UNIQUE NOT NULL,      -- URL and routing friendly handle (e.g., 'crypto_tracker') use '_' for slug
    name VARCHAR(255) NOT NULL,             -- Human readable label
    description TEXT NOT NULL,              -- Documentation context fed to the LLM agent
    schema_json JSONB NOT NULL,             -- JSON Schema describing required argument parameters
    rules_text TEXT NOT NULL,               -- Operational constraints for model generation boundaries
    script_code TEXT NOT NULL,              -- The raw TypeScript source code string
    embedding vector(1536),                 -- Vector embedding representation of the description for RAG intent classification
    version INT DEFAULT 1 NOT NULL,         -- Incremental change tracker
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

```


2. **The Rust Dynamic Abstract Integration Trait:**
   Define the updated dynamic trait structure in your tools module:
```rust
pub trait NomiDynamicPlugin: Send + Sync {
    fn slug(&self) -> &str;
    fn schema(&self) -> serde_json::Value;
    fn rules(&self) -> &str;
    fn version(&self) -> i32;
    fn execute<'a>(&'a self, args: serde_json::Value) -> futures::future::BoxFuture<'a, anyhow::Result<String>>;
}

```



### Step 1.2: Implement the Bun Subprocess Execution Engine

Create `src/plugins/edge_runner.rs`. This struct implements your execution interface by piping TypeScript parameters natively through an ephemeral Bun instance inside your Alpine container:

```rust
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

pub struct BunEdgeExecutor {
    pub slug: String,
    pub script_code: String,
}

impl BunEdgeExecutor {
    pub async fn run(&self, args: serde_json::Value, bridge_token: &str) -> anyhow::Result<String> {
        // Prepend arguments, global context variables, and internal bridge access tokens
        let unified_script = format!(
            "const BunArgs = {};\nconst BRIDGE_TOKEN = '{}';\n{}",
            args.to_string(),
            bridge_token,
            self.script_code
        );

        // 🚨 ALPINE DOCKER ENV COMPATIBILITY: Target the absolute container binary path
        let mut child = Command::new("/usr/local/bin/bun")
            .args(["run", "-e", &unified_script])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow::anyhow!("Alpine container lacked Bun system capabilities: {}", e))?;

        // Enforce a strict 5-second timeout to crush runaway loops instantly
        match timeout(Duration::from_secs(5), child.wait_with_output()).await {
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
                let _ = child.kill().await;
                Err(anyhow::anyhow!("Execution timed out. Process killed to protect server memory limits."))
            }
        }
    }
}

```

### Step 1.3: Secure Localhost-Only RPC Bridge (Calling Internal Rust Code)

To let your TypeScript scripts read incoming messages or execute knowledge base vector retrievals, expose a secure **Localhost-Only API Route** inside Axum.

1. **The Guard Middleware:** Any incoming connection to `/internal/rpc/*` must verify that the connection originates strictly from local loopback (`127.0.0.1`) and matches a cryptographically secure `BRIDGE_TOKEN` generated per execution turn.
2. **The RPC Endpoints:** Create dedicated internal controller endpoints at `src/routes/internal_rpc.rs`:
* `POST /internal/rpc/retrieve-knowledge` -> Invokes your native Rust RAG context matching matrix.
* `POST /internal/rpc/incoming-history` -> Queries your Postgres database to return the active conversation window.



### Step 1.4: Complete REST CRUD + Embedding Automation Engine

Expose standard REST management points at `src/routes/plugins.rs` for dynamic control from SvelteKit:

* `GET /api/plugins` -> Fetches all rows from `edge_functions`.
* `POST /api/plugins` -> Creates a plugin row. **CRITICAL:** Call your embedding provider using the plugin's `description` field string, and write the generated vector into the `embedding` column for semantic intent classification.
* `PUT /api/plugins/:slug` -> Increments `version`, re-generates description vector data layers, and updates code fields.
* `DELETE /api/plugins/:slug` -> Drops the row smoothly from your table.

---

## 🎨 Phase 2: SvelteKit Plugin Management & Playground Dashboard

Create a dynamic development environment interface at `src/routes/dashboard/plugins/[slug]/+page.svelte` (and an identical empty template creation layout at `/plugins/new`).

### Step 2.1: Lightweight Code Editor UI with Type Context Definitions

Build a clean, dual-pane grid layout page using SvelteKit:

* **Left Sidebar Configuration Panel:** Input text blocks for `name`, `slug`, `description`, `rules_text`, and a formatted text box for the parameter `schema_json` configurations.
* **Right Code Canvas Area:** Implement a lightweight text field configured for code formatting, utilizing pre-loaded global signature context mappings:

```typescript
// Auto-completion & standard context declarations injected for the plugin canvas:
declare const BunArgs: Record<string, any>; // Parsed parameters defined by your JSON Schema
declare const BRIDGE_TOKEN: string;          // Token used to verify internal calls

/**
 * Accesses your Rust gateway's native vector data pipeline
 */
async function callInternalKnowledgeBase(query: string, limit = 3): Promise<any> {
    const res = await fetch("http://localhost:8000/internal/rpc/retrieve-knowledge", {
        method: "POST",
        headers: { "Content-Type": "application/json", "X-Bridge-Token": BRIDGE_TOKEN },
        body: JSON.stringify({ query, limit })
    });
    return res.json();
}

```

### Step 2.2: Live Playground Testing Loop Simulation

Add an interactive **"Simulate Plugin Execution"** testing interface console area at the bottom of your code dashboard canvas page:

1. **Dynamic Input Binder:** Parses the schema string on the fly and auto-generates input form testing fields for each parameter property.
2. **The Test Runner Trigger:** Clicking the simulation action button triggers an asynchronous client fetch to the gateway execution endpoint (`POST /api/edge/execute`), capturing stdout streams or runtime exceptions inside a minimalist debugging console component element.

---

## 🐳 Phase 3: Infrastructure & Multi-Stage Alpine Dockerfile Refactor

Refactor the project's production `Dockerfile` to extract the official Alpine-compatible Bun binary and layer it alongside the offline SQLX `gateway-rust` build:

```dockerfile
# --- Stage 1: Build the high-performance Rust gateway binary ---
FROM rust:1.95-alpine AS builder
RUN apk add --no-cache musl-dev pkgconfig perl make gcc g++
WORKDIR /app
COPY . .
# Enforce offline SQLX compilation
RUN SQLX_OFFLINE=true cargo build --release

# --- Stage 2: Extract official Alpine Bun engine assets ---
FROM oven/bun:alpine AS bun-assets

# --- Stage 3: Final consolidated runtime environment ---
FROM alpine:latest
# CRITICAL: Add libstdc++ and gcompat to ensure the Bun binary executes flawlessly on musl libc
RUN apk add --no-cache libgcc openssl ca-certificates libstdc++ gcompat

WORKDIR /app

# A) Copy the bare-metal compiled Rust binary and migrations
COPY --from=builder /app/target/release/gateway-rust .
COPY --from=builder /app/migrations ./migrations
COPY README.md ./README.md

# B) Copy the Bun binary directly from the Alpine Bun image
COPY --from=bun-assets /usr/local/bin/bun /usr/local/bin/bun

# Verify runtimes are accessible natively within the container file system at build time
RUN /usr/local/bin/bun --version

EXPOSE 8000
CMD ["./gateway-rust"]

```

---

### Step 3.1: Final Verification Compile Pass

* Run a clean workspace compilation check to verify type alignment across Axum request streams.
* Verify that the multi-stage Docker configuration compiles into a lean image layout, preserving the offline SQLX migration requirements.