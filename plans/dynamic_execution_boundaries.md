
# Master Blueprint: Dynamic Execution Boundaries (DEB) with Redis Caching & SvelteKit Management UI

## 🚨 Critical Agent Constraints & Safety Guardrails

* **REDIS KEY CONVENTION:** Store all volatile room configuration state metrics using the explicit memory string template handle layout: `nomi:srp:thresholds:{conversation_id}`.
* **ENFORCE BOUNDS:** All threshold values must be strictly forced between float metrics `0.0` and `1.0`. Lowering the `interaction_gate` boundary lets more casual text signals trigger Nomi.
* **DUAL-WRITE INTEGRITY:** Any mutation (manual web input sliders or natural language AI tool triggers) **must update the Postgres table and hot-patch the Redis cache key concurrently** to prevent state drift.

---

## 🏗️ Phase 1: Storage Layer Migration (Postgres)

Execute this migration to append the structured boundary metadata config row blocks directly onto your active conversation workspace configurations:

```sql
-- Add dynamic threshold orchestration configuration columns to your existing conversation matrix layout
ALTER TABLE conversations 
ADD COLUMN IF NOT EXISTS gateway_thresholds JSONB DEFAULT '{
    "interaction_gate": 0.5,
    "intent_classification": 0.7,
    "guardrails": 0.85
}'::JSONB NOT NULL;

-- Optimize index checks to guarantee immediate relational lookups
CREATE INDEX IF NOT EXISTS idx_conversations_gateway_thresholds ON conversations USING gin (gateway_thresholds);

```

---

## ⚙️ Phase 2: High-Speed Caching Layer (Rust Redis Service)

Create `gateway-rust/src/services/deb_cache.rs`. This handles type-safe mode translation wrappers and thread-safe cache lookup hits to protect your 4GB VPS processing memory lines:

```rust
use redis::AsyncCommands;
use serde_json::{json, Value};
use sqlx::PgPool;
use tracing::{info, error};

const CACHE_PREFIX: &str = "nomi:srp:thresholds";

pub enum InteractionMode {
    Proactive,
    Balanced,
    Conservative,
    Silent,
}

impl InteractionMode {
    pub fn from_threshold(val: f64) -> Self {
        if val <= 0.25 { InteractionMode::Proactive }
        else if val <= 0.50 { InteractionMode::Balanced }
        else if val <= 0.75 { InteractionMode::Conservative }
        else { InteractionMode::Silent }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            InteractionMode::Proactive => "Proactive Mode",
            InteractionMode::Balanced => "Balanced Mode",
            InteractionMode::Conservative => "Conservative Mode",
            InteractionMode::Silent => "Silent Monitor Mode",
        }
    }
}

pub async fn get_execution_boundaries(
    redis_conn: &mut redis::aio::Connection,
    pool: &PgPool,
    conversation_id: uuid::Uuid,
) -> anyhow::Result<Value> {
    let cache_key = format!("{}:{}", CACHE_PREFIX, conversation_id);

    // 1. Try high-speed Redis memory read lines
    if let Ok(cached_json) = redis_conn.get::<_, String>(&cache_key).await {
        if let Ok(parsed_val) = serde_json::from_str::<Value>(&cached_json) {
            return Ok(parsed_val);
        }
    }

    // 2. Cache Miss Fallback: Query permanent Postgres table records
    info!("🪹 DEB Cache Miss for room [{}]. Loading from Postgres.", conversation_id);
    let record = sqlx::query!(
        "SELECT gateway_thresholds FROM conversations WHERE id = $1 LIMIT 1",
        conversation_id
    )
    .fetch_one(pool).await?;

    let thresholds = record.gateway_thresholds;

    // 3. Hydrate Redis cache with a safe 1-hour expiration Time-To-Live window
    let serialized = thresholds.to_string();
    let _: redis::RedisResult<()> = redis_conn.set_ex(&cache_key, serialized, 3600).await;

    Ok(thresholds)
}

```

---

## 📡 Phase 3: Web Control & Adjustment API Layer (Axum Router)

Create or append these two REST API endpoints inside your core gateway controller modules (e.g., `src/routes/srp_api.rs`) to allow smooth UI state bindings and natural language conversions:

```rust
use axum::{extract::{State, Path}, Json, response::IntoResponse};
use serde_json::json;
use redis::AsyncCommands;
use crate::AppState;

// 1. GET /api/srp/conversations/:id/thresholds
pub async fn get_room_thresholds(State(state): State<AppState>, Path(id): Path<uuid::Uuid>) -> impl IntoResponse {
    let mut redis_conn = match state.redis_pool.get_connection().await {
        Ok(c) => c,
        Err(e) => return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
    };
    
    match crate::services::deb_cache::get_execution_boundaries(&mut redis_conn, &state.pool, id).await {
        Ok(boundaries) => (axum::http::StatusCode::OK, Json(boundaries)).into_response(),
        Err(e) => (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response()
    }
}

// 2. POST /api/srp/conversations/:id/thresholds (Dual-Write Handler)
pub async fn update_room_thresholds(State(state): State<AppState>, Path(id): Path<uuid::Uuid>, Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    let layer = payload["target_layer"].as_str().unwrap_or("");
    let mut val = payload["new_threshold_value"].as_f64().unwrap_or(0.5);
    
    if val < 0.0 { val = 0.0; }
    if val > 1.0 { val = 1.0; }

    // Phase A: Write through persistent Postgres logs
    let db_res = sqlx::query!(
        "UPDATE conversations SET gateway_thresholds = gateway_thresholds || jsonb_build_object($1::text, $2::float8), updated_at = NOW() WHERE id = $3",
        layer, val, id
    ).execute(&state.pool).await;

    if db_res.is_err() {
        return axum::http::StatusCode::BAD_REQUEST.into_response();
    }

    // Extract fully updated object layout to prevent sync parsing mismatches
    let record = sqlx::query!("SELECT gateway_thresholds FROM conversations WHERE id = $1 LIMIT 1", id)
        .fetch_one(&state.pool).await.unwrap();

    // Phase B: Evict and refresh target memory line keys inside Redis cache structures
    if let Ok(mut redis_conn) = state.redis_pool.get_connection().await {
        let cache_key = format!("nomi:srp:thresholds:{}", id);
        let _: redis::RedisResult<()> = redis_conn.set_ex(&cache_key, record.gateway_thresholds.to_string(), 3600).await;
    }

    // Calculate Mode Mapping Labels to emit over the live event bus
    let mode_label = if layer == "interaction_gate" {
        crate::services::deb_cache::InteractionMode::from_threshold(val).as_str()
    } else if layer == "intent_classification" {
        if val <= 0.40 { "Experimental Mode" } else if val <= 0.70 { "Adaptive Mode" } else { "Strict Mode" }
    } else {
        if val <= 0.50 { "Permissive Mode" } else if val <= 0.80 { "Standard Mode" } else { "Hardened Mode" }
    };

    // Emit event bus notification to update active frontend UI charts synchronously
    let ui_event = json!({ "conversation_id": id, "layer": layer, "value": val, "mode": mode_label });
    crate::event_bus::publish(crate::event_bus::Event::SrpThresholdUpdate(ui_event));

    (axum::http::StatusCode::OK, Json(json!({ "status": "synchronized", "mode": mode_label }))).into_response()
}

```

---

## 🤖 Phase 4: Dynamic AI Integration Tool (Rust Plugin)

Create `gateway-rust/src/common/tools/plugins/adjust_thresholds.rs`. This hooks into Nomi's intent layer, turning speech commands (*"be proactive 20%"*) into execution boundary movements:

```rust
use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::ToolDispatcher;
use serde_json::{json, Value};
use futures::future::BoxFuture;

pub struct AdjustThresholdsPlugin;

impl NomiToolPlugin for AdjustThresholdsPlugin {
    fn schema(&self) -> Value {
        json!({
            "name": "adjust_thresholds",
            "description": "DYNAMIC DEB CALIBRATION: Call this tool immediately when the user tells you to change your responsiveness, strictness, proactivity, or safety filters. This modifies runtime threshold parameters.",
            "parameters": {
                "type": "object",
                "properties": {
                    "target_layer": { 
                        "type": "string", 
                        "enum": ["interaction_gate", "intent_classification", "guardrails"],
                        "description": "The specific boundary filtering layer to alter."
                    },
                    "new_threshold_value": { 
                        "type": "number", 
                        "description": "The newly computed parameter decimal. Must scale between 0.0 and 1.0." 
                    },
                    "explanation": { "type": "string", "description": "Brief summary of calculation parameters." }
                },
                "required": ["target_layer", "new_threshold_value", "explanation"]
            }
        })
    }

    fn rules(&self) -> &str { "1. Code percentages logically: 'be proactive 20%' translates to lowering the interaction_gate to let more conversational signals pass through." }
    fn matching_intents(&self) -> &[&str] { &["SYSTEM_CONFIGURATION"] }

    fn execute<'a>(&'a self, dispatcher: &'a ToolDispatcher, args: Value) -> BoxFuture<'a, anyhow::Result<String>> {
        Box::pin(async move {
            let layer = args["target_layer"].as_str().unwrap_or("");
            let val = args["new_threshold_value"].as_f64().unwrap_or(0.5);
            let cid = dispatcher.workspace_id;

            // Trigger the centralized dual-write synchronization handler engine directly
            // (Re-uses your REST handler pipeline logic internally to protect consistency bounds)
            let client = reqwest::Client::new();
            let _ = client.post(format!("http://localhost:3000/api/srp/conversations/{}/thresholds", cid))
                .json(&json!({ "target_layer": layer, "new_threshold_value": val }))
                .send().await?;

            Ok(format!("SYSTEM SIGNAL: Boundary parameters applied successfully. Next conversational turns will read these metrics instantly from the DEB memory cache."))
        })
    }
}

```

---

## 🎨 Phase 5: SvelteKit Web Management Dashboard Component

Create a sub-panel interface tab inside your conversation monitor page configuration files layout: `src/routes/dashboard/srp/monitoring/[conversation_id]/+page.svelte`. This renders manual slider settings directly on top of your dark WhatsApp canvas layout:

```html
<script lang="ts">
  import { onMount } from 'svelte';
  export let data;
  
  let conversationId = data.conversation_id;
  let thresholds = {
    interaction_gate: 0.5,
    intent_classification: 0.7,
    guardrails: 0.85
  };

  let modeLabels = {
    interaction_gate: "Loading Mode...",
    intent_classification: "Loading Mode...",
    guardrails: "Loading Mode..."
  };

  onMount(async () => {
    // 1. Initial hydration check: pull active config numbers from high-speed Redis route
    const res = await fetch(`/api/srp/conversations/${conversationId}/thresholds`);
    if (res.ok) {
      thresholds = await res.json();
      recalculateAllLabels();
    }
  });

  function getInteractionLabel(val: number) {
    if (val <= 0.25) return 'Proactive Mode 🏁';
    if (val <= 0.50) return 'Balanced Mode 🤝';
    if (val <= 0.75) return 'Conservative Mode 🛡️';
    return 'Silent Monitor Mode 🤫';
  }

  function getIntentLabel(val: number) {
    if (val <= 0.40) return 'Experimental Mode 🧪';
    if (val <= 0.70) return 'Adaptive Mode 🏎️';
    return 'Strict Mode 📐';
  }

  function getGuardrailLabel(val: number) {
    if (val <= 0.50) return 'Permissive Mode 🔓';
    if (val <= 0.80) return 'Standard Mode 👤';
    return 'Hardened Shield Mode 🌋';
  }

  function recalculateAllLabels() {
    modeLabels.interaction_gate = getInteractionLabel(thresholds.interaction_gate);
    modeLabels.intent_classification = getIntentLabel(thresholds.intent_classification);
    modeLabels.guardrails = getGuardrailLabel(thresholds.guardrails);
  }

  async function adjustBoundarySlider(layer: string, value: number) {
    // 2. Continuous manual override: Push sliders shifts straight to dual-write router endpoint
    const res = await fetch(`/api/srp/conversations/${conversationId}/thresholds`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ target_layer: layer, new_threshold_value: value })
    });
    if (res.ok) {
      recalculateAllLabels();
    }
  }
</script>

<div class="deb-management-panel bg-[#111b21] border border-[#222e35] p-5 rounded-lg flex flex-col gap-6 text-white max-w-md shadow-2xl">
  <div class="border-b border-[#222e35] pb-2">
    <h3 class="text-sm font-bold text-emerald-400 uppercase tracking-wider">🏎️ Dynamic Execution Boundaries</h3>
    <p class="text-[11px] text-neutral-400">Manual slider bypass for high-speed cache lines monitoring templates.</p>
  </div>

  <div class="flex flex-col gap-5">
    <div class="flex flex-col gap-2">
      <div class="flex justify-between items-center text-xs">
        <span class="font-semibold text-neutral-300">Sociability (Interaction Gate)</span>
        <span class="font-mono text-emerald-400 text-[11px]">{thresholds.interaction_gate.toFixed(2)}</span>
      </div>
      <input type="range" min="0.0" max="1.0" step="0.05" bind:value={thresholds.interaction_gate} on:input={() => adjustBoundarySlider('interaction_gate', thresholds.interaction_gate)} class="w-full accent-emerald-500 h-1 bg-[#202c33] rounded-lg appearance-none cursor-pointer" />
      <div class="text-[10px] font-mono text-neutral-400 italic">Current: {modeLabels.interaction_gate}</div>
    </div>

    <div class="flex flex-col gap-2">
      <div class="flex justify-between items-center text-xs">
        <span class="font-semibold text-neutral-300">Confidence (Intent Classifier)</span>
        <span class="font-mono text-emerald-400 text-[11px]">{thresholds.intent_classification.toFixed(2)}</span>
      </div>
      <input type="range" min="0.0" max="1.0" step="0.05" bind:value={thresholds.intent_classification} on:input={() => adjustBoundarySlider('intent_classification', thresholds.intent_classification)} class="w-full accent-emerald-500 h-1 bg-[#202c33] rounded-lg appearance-none cursor-pointer" />
      <div class="text-[10px] font-mono text-neutral-400 italic">Current: {modeLabels.intent_classification}</div>
    </div>

    <div class="flex flex-col gap-2">
      <div class="flex justify-between items-center text-xs">
        <span class="font-semibold text-neutral-300">Vigilance (Guardrail Shielding)</span>
        <span class="font-mono text-emerald-400 text-[11px]">{thresholds.guardrails.toFixed(2)}</span>
      </div>
      <input type="range" min="0.0" max="1.0" step="0.05" bind:value={thresholds.guardrails} on:input={() => adjustBoundarySlider('guardrails', thresholds.guardrails)} class="w-full accent-emerald-500 h-1 bg-[#202c33] rounded-lg appearance-none cursor-pointer" />
      <div class="text-[10px] font-mono text-neutral-400 italic">Current: {modeLabels.guardrails}</div>
    </div>
  </div>
</div>

```