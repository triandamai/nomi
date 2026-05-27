# Technical Checklist Plan: HTO Multi-Resurrection & Dynamic Bridging

This checklist guides the step-by-step technical implementation of the Universal HTO Multi-Resurrection & Dynamic Bidirectional Bridging workflow in `gateway-rust`.

---

## 🛠️ Implementation Checklist

### Step 1: Define Database Helpers & Query Routines
- [x] Create query helpers in `gateway-rust/src/services/task_orchestrator.rs` or `v2_orchestrator.rs` to find tasks waiting for feedback:
  - `get_active_task_waiting_feedback(pool, sub_convo_id)`: Searches for active tasks in status `'waiting_external_feedback'`.
  - `get_active_task_paused_for_input(pool, convo_id)`: Searches for active tasks in status `'paused_for_input'`.
- [x] Ensure query indexes map efficiently to avoid latency on incoming message processing.

### Step 2: Implement the Gateway Dual-Interception Layer
- [x] Inject the interception layer at the top of `process_v2_message` in `gateway-rust/src/feature/message_processor/v2_orchestrator.rs`.
- [x] If a task in `'waiting_external_feedback'` status matches the incoming sub-conversation:
  - Save the incoming message directly under `sub_conversation_id`.
  - Update task status to `'running'`.
  - Spawn the task thread: `spawn_task_loop(state.clone(), task.id)`.
  - Return early with the saved message to bypass standard persona generation.
- [x] If a task in `'paused_for_input'` status matches the incoming parent conversation:
  - Save the incoming message directly under parent `conversation_id`.
  - Update task status to `'running'`.
  - Spawn the task thread: `spawn_task_loop(state.clone(), task.id)`.
  - Return early with the saved message to bypass standard persona generation.

### Step 3: Implement Context Hydrator for Twin-History
- [x] Update `advanced_orchestrate_task_step` in `gateway-rust/src/services/task_orchestrator.rs` to fetch both parent chat history AND sub-chat history (if `sub_conversation_id` is present).
- [x] Hydrate both histories into the background LLM planner's system prompt using distinct, labeled structural headers so the LLM has perfect awareness of both sides.

### Step 4: Inject LLM System Guidelines & Commands
- [x] Update the background LLM planner prompt template in `task_orchestrator.rs` with clear, absolute guidelines for dynamic pauses:
  - If target reply is needed: transition status to `'waiting_external_feedback'`.
  - If owner input/clarification is needed: transition status to `'paused_for_input'`.

### Step 5: Verify & Compile Checklist
- [x] Run `cargo check` to ensure zero compilation warnings and absolute type safety.
- [x] Verify dynamic updates on Svelte graph canvas dashboard to ensure status changes render beautifully.

---

## 🏁 Progress Ledger

* **Status:** ✅ 100% Completed
* **Active Step:** Implementation finished successfully!
