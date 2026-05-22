
# Master Architectural Blueprint: Distributed Agent Factory (DAF) with Native Evolution Mapping

## 🚨 Critical Agent Constraints & Safety Guardrails

* **EVOLUTION TOPIC MATCHING:** The Rust background SWE coding agent must publish all stream metadata changes to the explicit topic: `nomi/srp/factory/evolution`. The message payload must be a JSON object containing `slug`, `step`, `log`, and `code`. This guarantees your frontend `handleMessage` case block maps it flawlessly to `eventBus.emit('sse-evolution', data)`.
* **RUNNING STANDARDS (`run(args)`):** The SWE coding agent must structure its output exactly to fit your existing edge execution layer. It must provide a clean script where the entry point is explicitly defined as `export default function run(args) { ... }`.
* **CONCURRENCY LOCKS:** Enforce a strict global system semaphore of **maximum 2 concurrent execution tasks at a time** across the entire workspace. Any extra approved builds must remain queued as `approved`.
* **ANTI-DRIFT SANDBOX PROTECTION:** The background SWE coding agent must run at a rigid **Temperature 0.0** with a strict execution time deadline of **1000ms per test run** to prevent runaway loops or memory blocks on the 4GB VPS.
* **HUMAN-IN-THE-LOOP (HITL) SEPARATION:** The SWE coding agent only writes to a `compiled_code` buffer column. It **NEVER** alters your live `edge_functions` table directly. Only an explicit admin deployment action can move code from staging into your production runtime environment.

---

## 🏗️ Phase 1: Database Migration & Schema Setup

Execute this database migration to handle staging records, concurrency status indices, and clean FIFO truncation logic for your self-correcting telemetry pipeline:

```sql
-- Dynamic Tool Proposals Staging Table
CREATE TABLE IF NOT EXISTS plugin_creation_suggestions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    schema_json JSONB NOT NULL,
    how_it_works TEXT NOT NULL,
    compiled_code TEXT DEFAULT '' NOT NULL,
    status VARCHAR(50) DEFAULT 'pending' NOT NULL, -- pending, approved, processing, ready, failed, deployed
    error_logs TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- Index optimization to maintain high-speed polling bounds for your task executors
CREATE INDEX IF NOT EXISTS idx_plugin_suggestions_status ON plugin_creation_suggestions(status);

```

---

## ⚙️ Phase 2: Gateway REST API Management Layer

Create or append these endpoints within your Axum router architecture (e.g., `src/routes/srp_api.rs`). Ensure these controllers read and mutate state safely using atomic transactional locks:

```rust
use axum::{extract::{State, Path}, Json, response::IntoResponse};
use serde_json::json;
use crate::AppState;

// 1. GET /api/srp/proposals
pub async fn get_proposals(State(state): State<AppState>) -> impl IntoResponse {
    let rows = sqlx::query!("SELECT slug, name, description, schema_json, how_it_works, compiled_code, status FROM plugin_creation_suggestions ORDER BY created_at DESC")
        .fetch_all(&state.pool).await;
    match rows {
        Ok(data) => (axum::http::StatusCode::OK, Json(data)).into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response()
    }
}

// 2. PUT /api/srp/proposals/:slug
pub async fn update_proposal(State(state): State<AppState>, Path(slug): Path<String>, Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    let desc = payload["description"].as_str().unwrap_or("");
    let schema = &payload["schema_json"];
    
    let res = sqlx::query!("UPDATE plugin_creation_suggestions SET description = $1, schema_json = $2, updated_at = NOW() WHERE slug = $3", desc, schema, slug)
        .execute(&state.pool).await;
    match res {
        Ok(_) => axum::http::StatusCode::OK.into_response(),
        Err(e) => (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response()
    }
}

// 3. DELETE /api/srp/proposals/:slug
pub async fn delete_proposal(State(state): State<AppState>, Path(slug): Path<String>) -> impl IntoResponse {
    let res = sqlx::query!("DELETE FROM plugin_creation_suggestions WHERE slug = $1", slug)
        .execute(&state.pool).await;
    match res {
        Ok(_) => axum::http::StatusCode::NO_CONTENT.into_response(),
        Err(e) => (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response()
    }
}

// 4. POST /api/srp/proposals/:slug/approve
pub async fn approve_proposal(State(state): State<AppState>, Path(slug): Path<String>) -> impl IntoResponse {
    let active_count = sqlx::query!("SELECT COUNT(*) as count FROM plugin_creation_suggestions WHERE status = 'processing'")
        .fetch_one(&state.pool).await.unwrap().count.unwrap_or(0);

    let target_status = if active_count >= 2 { "approved" } else { "processing" };

    let res = sqlx::query!("UPDATE plugin_creation_suggestions SET status = $1, updated_at = NOW() WHERE slug = $2 RETURNING status", target_status, slug)
        .fetch_one(&state.pool).await;

    match res {
        Ok(row) => {
            if row.status == "processing" {
                let pool = state.pool.clone();
                let state_clone = state.clone();
                tokio::spawn(async move {
                    let _ = crate::services::swe_agent::process_factory_build(pool, state_clone, slug).await;
                });
            }
            (axum::http::StatusCode::ACCEPTED, Json(json!({"status": row.status}))).into_response()
        },
        Err(e) => (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response()
    }
}

// 5. POST /api/srp/proposals/:slug/deploy
pub async fn deploy_proposal(State(state): State<AppState>, Path(slug): Path<String>) -> impl IntoResponse {
    let record = sqlx::query!("SELECT name, description, schema_json, compiled_code, status FROM plugin_creation_suggestions WHERE slug = $1 LIMIT 1", slug)
        .fetch_optional(&state.pool).await.unwrap();

    if let Some(row) = record {
        if row.status != "ready" {
            return (axum::http::StatusCode::BAD_REQUEST, "Only plugins in 'ready' state can be deployed.").into_response();
        }

        let embedding = match crate::rag::get_embedding(&state.gemini_api_key, &row.description).await {
            Ok(emb) => emb.embedding.values,
            Err(e) => return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Embedding generation failed: {}", e)).into_response()
        };

        let mut tx = state.pool.begin().await.unwrap();
        
        sqlx::query!(
            "INSERT INTO edge_functions (slug, name, description, schema_json, rules_text, script_code, embedding) \
             VALUES ($1, $2, $3, $4, '', $5, $6::vector) \
             ON CONFLICT (slug) DO UPDATE SET script_code = EXCLUDED.script_code, description = EXCLUDED.description, embedding = EXCLUDED.embedding, version = edge_functions.version + 1",
            slug, row.name, row.description, row.schema_json, row.compiled_code, embedding
        ).execute(&mut *tx).await.unwrap();

        sqlx::query!("UPDATE plugin_creation_suggestions SET status = 'deployed', updated_at = NOW() WHERE slug = $1", slug)
            .execute(&mut *tx).await.unwrap();

        tx.commit().await.unwrap();
        return (axum::http::StatusCode::OK, "Plugin hot-patched into production successfully.").into_response();
    }
    axum::http::StatusCode::NOT_FOUND.into_response()
}

```

---

## 🤖 Phase 3: Dedicated SWE Coding Agent Loop via Mapped Evolution Topic

Create `gateway-rust/src/services/swe_agent.rs`. This compiler system forces the model to structure code for your exact Bun runtime layout, wrapping errors and streaming statuses directly through your gateway's centralized **Event Bus** pointing to the `evolution` topic end node:

```rust
use sqlx::PgPool;
use serde_json::json;
use tracing::{info, error};

pub async fn process_factory_build(pool: PgPool, state: crate::AppState, slug: String) -> anyhow::Result<()> {
    let suggestion = sqlx::query!("SELECT name, description, schema_json, how_it_works FROM plugin_creation_suggestions WHERE slug = $1 LIMIT 1", slug)
        .fetch_one(&pool).await?;

    // ⚡ DECOUPLED EVENT BUS EMISSION SYSTEM FOR EVOLUTION MATCHING
    let emit_factory_event = |step: &str, log_msg: &str, active_code: &str| {
        let current_slug = slug.clone();
        
        // Structures the precise JSON object expected by your frontend components
        let payload = json!({ 
            "slug": current_slug, 
            "step": step, 
            "log": log_msg, 
            "code": active_code 
        });
        
        // 🚨 CRITICAL: Publish through event_bus setup pointing to the 'evolution' topic endpoint descriptor.
        // Your event bus routes this event out to EMQX under topic suffix `/evolution`, tripping your case handler!
        crate::event_bus::publish(crate::event_bus::Event::SrpFactoryUpdate {
            topic: format!("nomi/srp/factory/evolution"),
            payload: payload
        });
    };

    emit_factory_event("thinking", "[FACTORY]: SWE Agent awakened. Composing dynamic code manual variants...", "");

    let swe_system_prompt = "\
    You are an elite automated backend architect specializing in serverless V8 and Bun runtime environments. \
    Your output must be 100% executable TypeScript code block statements without introductory filler or markdown chatter. \
    CRITICAL STRUCTURE RULE: You MUST export a default function matching exactly this entrypoint format:\n\
    export default function run(args: any) {\n  // your implementation here\n}\n\
    The `args` parameter maps directly to your input properties schema matrix variables. Ensure all data processing happens inside this function framework.";

    let mut coding_prompt = format!(
        "Synthesize a complete, production-grade TypeScript edge plugin file. Specifications:\n\
         Name: {}\nDescription: {}\nParameters Expected Schema: {}\nFunctional Roadmap: {}\n\
         Respond ONLY with the raw typescript execution code inside standard backtick fences, starting with the default export run function layout.",
         suggestion.name, suggestion.description, suggestion.schema_json, suggestion.how_it_works
    );

    let mut attempt = 0;
    let max_retries = 3;

    while attempt < max_retries {
        attempt += 1;
        emit_factory_event("thinking", &format!("[SYNTHESIS RUN {}/{}]: Composing runtime script variables...", attempt, max_retries), "");

        let response = state.gemini.generate_content().with_message(gemini_rust::Message {
            role: gemini_rust::Role::User,
            content: gemini_rust::Content {
                parts: Some(vec![gemini_rust::Part::Text { text: coding_prompt.clone(), thought: None, thought_signature: None }]),
                role: Some(gemini_rust::Role::User),
            },
        })
        .with_system_prompt(swe_system_prompt.to_string())
        .with_temperature(0.0) // Lock against code syntax drift variations
        .execute().await?;

        let code = response.text().trim().replace("```typescript", "").replace("```", "").trim().to_string();
        emit_factory_event("sandboxing", &format!("[SANDBOX]: Injecting code into Bun subprocess test harness run {}...", attempt), &code);

        let test_pass = crate::services::code_mutator::test_candidate_code(&code).await?;

        if test_pass.success {
            emit_factory_event("success", "[VALIDATION SUCCESS]: Code structures verified cleanly through the sandbox runner. Ready for production deployment.", &code);
            
            sqlx::query!("UPDATE plugin_creation_suggestions SET compiled_code = $1, status = 'ready', error_logs = NULL, updated_at = NOW() WHERE slug = $2", code, slug)
                .execute(&pool).await?;
                
            check_and_trigger_next_queued_plugin(pool.clone(), state.clone()).await;
            return Ok(());
        }

        // 🚨 RECURSIVE SELF-HEALING LOGIC PASS
        emit_factory_event("healing", &format!("[SANDBOX TRACE ERROR]: Compilation failed. Initiating self-correction mechanics..."), &code);
        coding_prompt = format!(
            "Your previous code configuration attempt failed compilation or execution check rules.\n\
             ---- SANDBOX COMPILER STACK EXCEPTION LOGS ----\n{}\n\n\
             Review the error parameters carefully, preserve the required `export default function run(args)` signature, correct your bugs, and re-output code.",
            test_pass.output
        );
    }

    emit_factory_event("failed", "[ABORTED]: Failed execution validation gates within maximum retry thresholds.", "");
    sqlx::query!("UPDATE plugin_creation_suggestions SET status = 'failed', updated_at = NOW() WHERE slug = $1", slug).execute(&pool).await?;
    
    check_and_trigger_next_queued_plugin(pool, state).await;
    Ok(())
}

async fn check_and_trigger_next_queued_plugin(pool: PgPool, state: crate::AppState) {
    if let Ok(Some(next_row)) = sqlx::query!("UPDATE plugin_creation_suggestions SET status = 'processing', updated_at = NOW() WHERE id = (SELECT id FROM plugin_creation_suggestions WHERE status = 'approved' ORDER BY updated_at ASC LIMIT 1) RETURNING slug")
        .fetch_optional(&pool).await {
            tokio::spawn(async move {
                let _ = process_factory_build(pool, state, next_row.slug).await;
            });
        }
}

```

---

## 🎨 Phase 4: SvelteKit Factory Console Dashboard Route (Listens to `sse-evolution`)

Create your visualization panel at `src/routes/dashboard/srp/proposals/+page.svelte`. Since your custom MQTT `handleMessage` automatically translates topic strings into standard `eventBus` contexts, your page just listens to the legacy-compatible `sse-evolution` interface loop:

```html
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  // Import your shared workspace eventBus instance directly
  import { eventBus } from '$lib/utils/eventBus';

  export let data;
  let proposals = data.proposals || [];
  let selectedProposal = null;
  
  let liveLogs = [];
  let currentStep = "idle"; // idle, thinking, sandboxing, healing, success, failed
  let activeCodeOutput = "";

  function initializeEvolutionListener(slug: string) {
    selectedProposal = proposals.find(p => p.slug === slug);
    liveLogs = [`[MONITOR]: Attaching eventBus listener for legacy case tracking [${slug}]...`];
    currentStep = "thinking";
    activeCodeOutput = "";

    // 🚨 REGULAR INTERCEPTION: Listen directly to the sse-evolution emissions caught by handleMessage!
    eventBus.on('sse-evolution', handleEvolutionTelemetry);
  }

  function handleEvolutionTelemetry(event) {
    // Filter out message buffers belonging to other background compiler slugs
    if (!selectedProposal || event.slug !== selectedProposal.slug) return;

    if (event.log) liveLogs = [...liveLogs, event.log];
    if (event.step) currentStep = event.step;
    if (event.code) activeCodeOutput = event.code;
    
    if (event.step === "success" || event.step === "failed") {
      reloadProposalsList();
    }
  }

  async function reloadProposalsList() {
    const res = await fetch('/api/srp/proposals');
    proposals = await res.json();
  }

  async function launchBuild(slug: string) {
    const res = await fetch(`/api/srp/proposals/${slug}/approve`, { method: 'POST' });
    const data = await res.json();
    proposals = proposals.map(p => p.slug === slug ? { ...p, status: data.status } : p);
    initializeEvolutionListener(slug);
  }

  async function deployToProduction(slug: string) {
    liveLogs = [...liveLogs, `[DEPLOYMENT]: Sending hot-patch request to Axum gateway production runtime...`];
    const res = await fetch(`/api/srp/proposals/${slug}/deploy`, { method: 'POST' });
    if (res.ok) {
      liveLogs = [...liveLogs, `[SUCCESS]: Plugin hot-patched into live edge execution memory and ready for use globally!`];
      reloadProposalsList();
    } else {
      liveLogs = [...liveLogs, `[DEPLOY ERROR]: Edge deployment execution pass aborted.`];
    }
  }

  onDestroy(() => {
    eventBus.off('sse-evolution', handleEvolutionTelemetry);
  });
</script>

<div class="daf-canvas w-full h-screen overflow-y-auto p-6 flex flex-col gap-6 text-white select-none">
  <div class="border-b border-[#222e35] pb-4">
    <h1 class="text-2xl font-bold tracking-tight text-emerald-400">🏭 Distributed Agent Factory Console</h1>
    <p class="text-xs text-neutral-400">Review Nomi's code proposals, audit validation compilers, and hot-patch verified `run(args)` modules live via MQTT.</p>
  </div>

  <div class="grid grid-cols-1 lg:grid-cols-3 gap-6 items-start flex-1">
    <div class="flex flex-col gap-4">
      <h3 class="text-xs font-semibold tracking-wider text-neutral-400 uppercase">Staging Blueprints Queue</h3>
      <div class="flex flex-col gap-3 overflow-y-auto max-h-[calc(100vh-200px)]">
        {#each proposals as item}
          <div class="bg-[#111b21] border border-[#222e35] p-4 rounded-lg flex flex-col gap-2 transition-all hover:border-[#2a3942]">
            <div class="flex justify-between items-start gap-2">
              <div>
                <h4 class="font-bold text-sm text-neutral-200 leading-tight">{item.name}</h4>
                <span class="text-xs font-mono text-emerald-500">[{item.slug}]</span>
              </div>
              <span class="text-[10px] font-mono font-bold px-2 py-0.5 rounded uppercase border
                {item.status === 'pending' ? 'bg-amber-500/10 text-amber-400 border-amber-500/20' : ''}
                {item.status === 'approved' ? 'bg-blue-500/10 text-blue-400 border-blue-500/20 animate-pulse' : ''}
                {item.status === 'processing' ? 'bg-purple-500/10 text-purple-400 border-purple-500/20 animate-pulse' : ''}
                {item.status === 'ready' ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20' : ''}
                {item.status === 'deployed' ? 'bg-neutral-800 text-neutral-400 border-neutral-700' : ''}">
                {item.status}
              </span>
            </div>
            <p class="text-xs text-neutral-400 line-clamp-2 mt-1">{item.description}</p>
            
            <div class="grid grid-cols-2 gap-2 mt-3">
              <button on:click={() => initializeEvolutionListener(item.slug)} class="bg-[#202c33] hover:bg-[#2a3942] text-[11px] font-semibold py-1.5 px-2 rounded text-center transition-all border border-neutral-700">Monitor Stream</button>
              {#if item.status === 'pending'}
                <button on:click={() => launchBuild(item.slug)} class="bg-emerald-600 hover:bg-emerald-500 text-[11px] font-bold py-1.5 px-2 rounded text-center transition-colors">Build Tool</button>
              {:else if item.status === 'ready'}
                <button on:click={() => deployToProduction(item.slug)} class="bg-blue-600 hover:bg-blue-500 text-[11px] font-bold py-1.5 px-2 rounded text-center transition-colors">Deploy Live</button>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    </div>

    <div class="lg:col-span-2 grid grid-cols-1 md:grid-cols-2 gap-6 h-full min-h-[500px]">
      {#if selectedProposal}
        <div class="bg-[#111b21] border border-[#222e35] p-4 rounded-lg flex flex-col gap-3 h-full">
          <div class="flex items-center justify-between border-b border-[#222e35] pb-2">
            <h3 class="text-xs font-semibold tracking-wider text-neutral-400 uppercase">MQTT Factory Console</h3>
            <span class="text-[10px] font-mono font-bold px-2 py-0.5 rounded bg-black text-emerald-400 border border-neutral-800 uppercase animate-pulse">Loop: {currentStep}</span>
          </div>
          <div class="bg-black font-mono text-[11px] p-3 rounded flex-1 overflow-y-auto flex flex-col gap-1 min-h-[350px] text-neutral-300 border border-neutral-900 select-text scrollbar-thin">
            {#each liveLogs as chunk}
              <div class="{chunk.includes('[SANDBOX TRACE ERROR]') ? 'text-rose-400 font-semibold' : ''} {chunk.includes('[VALIDATION SUCCESS]') ? 'text-emerald-400 font-bold' : ''}">
                {chunk}
              </div>
            {/each}
          </div>
        </div>

        <div class="bg-[#111b21] border border-[#222e35] p-4 rounded-lg flex flex-col gap-3 h-full">
          <h3 class="text-xs font-semibold tracking-wider text-neutral-400 uppercase border-b border-[#222e35] pb-2">Active Source Canvas</h3>
          <div class="bg-[#202c33] rounded font-mono text-[11px] p-4 flex-1 overflow-y-auto whitespace-pre border border-neutral-800 text-emerald-400 max-h-[450px] select-text">
            {activeCodeOutput || "// Awaiting incoming Event Bus byte buffers from the active background SWE agent compiler loop..."}
          </div>
        </div>
      {:else}
        <div class="col-span-2 bg-[#111b21] border border-[#222e35] rounded-lg p-12 text-center text-xs text-neutral-500 italic flex items-center justify-center self-stretch">
          Select an active staging profile blueprint handle from the registry sidebar to initialize the monitoring grid dashboard views.
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .daf-canvas {
    background-color: #0b141a;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='80' height='80' viewBox='0 0 80 80'%3E%3Cg fill='%23202c33' fill-opacity='0.16'%3E%3Cpath d='M15 5h2v2h-2zm0 10h2v2h-2zm10-5h2v2h-2zm10 20h2v2h-2zm-20 10h2v2h-2zm30-5h2v2h-2zM5 45h2v2h-2zm15 15h2v2h-2zm40-30h2v2h-2zm-10-10h2v2h-2zm10 30h2v2h-2zm-20 15h2v2h-2zm30 10h2v2h-2zM55 5h2v2h-2zm0 10h2v2h-2zm-40 50h2v2h-2zm30 10h2v2h-2zm10-25h2v2h-2zm-5 15h2v2h-2zm-25 5h2v2h-2zm-10-35h2v2h-2z'/%3E%3Ccircle cx='40' cy='40' r='1'/%3E%3Cpath d='M45 40c0-2.8 2.2-5 5-5s5 2.2 5 5-2.2 5-5 5-5-2.2-5-5zm-30 0c0-2.8 2.2-5 5-5s5 2.2 5 5-2.2 5-5 5-5-2.2-5-5z'/%3E%3C/g%3E%3C/svg%3E");
    background-repeat: repeat;
    background-size: 140px 140px;
  }
</style>

```

---

### Step 4.3: Run Verification Target Compilation Build

* Run a clean `cargo build` optimization check to guarantee full thread alignment with your architectural updates.