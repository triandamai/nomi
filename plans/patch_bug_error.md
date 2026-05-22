# Master Patch: Multi-Turn Content Duplication & Datetime Format Alignment

## 🚨 Critical Agent Constraints

* **DO NOT** break the streaming chunk layout or change the underlying `execute_tools` futures.
* **DO** ensure that only the conversational content from the *final successful synthesis turn* is returned to the user when tool retries occur.

---

### Step 1: Fix Content Accumulation in the Orchestrator Loop

Open `gateway-rust/src/feature/message_processor/v2_orchestrator.rs`. Inside the `while loop_count < max_loops` block, change how conversational content is managed.

If Gemini calls a tool and the finish reason is not "stop", **clear out** any conversational text it generated *before* running the tool. This ensures that apologize-style filler text generated during a failed turn doesn't bleed into the final output message:

```rust
// Inside V2AgentOrchestrator loop execution right after function_calls check:
let current_calls: Vec<_> = tool_calls.into_iter().map(|c| c.clone()).collect();
let finish_reason = chunk.finish_reason.clone().unwrap_or_default();

if !current_calls.is_empty() && !finish_reason.eq_ignore_ascii_case("stop") {
    // 🚨 FIX: If the model is calling a tool this turn (and not finished), clear out teaser text.
    // This wipes away "Let me check that for you..." from the final response.
    accumulated_content.clear();
}
```

---

### Step 2: Clear Up Timezone Instructions in Guardrails

Your current system prompt instructions are confusing the model's function parameters. Update your `build_system_prompt` closure or your `PromptRegistry` to explicitly name the parameter variables for reminder lookups:

```rust
// Inside build_system_prompt definition wrapper:
combined.push_str("\n### Timezone & Tool Parameter Instructions\n");
combined.push_str(&format!(
    "The user's current local time is {} (Asia/Jakarta). \n\
     When calling date-range tracking tools like `get_reminder_stats`, you MUST format parameters like `start_after` and `end_before` as absolute strict ISO 8601 strings with offsets.\n\
     For a query about 'today', start_after MUST be formatted exactly as 'YYYY-MM-DDT00:00:00+07:00' and end_before as 'YYYY-MM-DDT23:59:59+07:00'.",
    now_local.format("%H:%M")
));

```

---

### Step 3: Implement Human-Style Synthesis Guardrail

To prevent Nomi from explaining her technical hiccups to Trian altogether, add this explicit rule into your **Guardrails** definition block:

```markdown
4. PROTOCOL ERROR HANDLING: If a tool call fails due to formatting or database exceptions, silently correct the arguments and re-call the tool immediately. NEVER tell the user about formatting bugs, code errors, or date-format retries. Keep the pipeline completely invisible.

```

---

### Step 4: Run a Verification Build Pass

* Ensure your loop targets compile flawlessly with the `.clear()` implementation on string stacks.