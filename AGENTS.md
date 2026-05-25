# Nomi: Operational Guide for Agents

## Project Overview
Nomi is an autonomous agentic workspace built for high-performance, multimodal orchestration across Web, Mobile, Telegram, and WhatsApp.

- **Backend**: Rust, Axum, Tokio, SQLx, PostgreSQL, pgvector.
- **Frontend**: SvelteKit, Svelte 5, TypeScript, Tailwind CSS.
- **Mobile/Desktop**: Flutter (nomi_mobile) for iOS, Android, macOS, and Windows.
- **Realtime**: MQTT (Mosquitto) for streaming thoughts and state synchronization.
- **Data**: pgvector (halfvec 3072) for high-fidelity RAG and graph memories.

## Repository Layout (Gateway Rust)
- `gateway-rust/src/routes.rs`: Top-level API routing and middleware.
- `gateway-rust/src/common/repository/`: Centralized data access layer (Source of Truth).
  - `conversation_repo.rs`: Redis-cached conversation and DEB threshold management.
  - `message_repo.rs`: Durable message persistence and telemetry logging.
  - `channel_repo.rs`: Identity and channel mapping (WhatsApp/Telegram).
- `gateway-rust/src/services/`: Core logic services.
  - `interaction_gate.rs`: Isolated pre-filtering for group chats (Momentum-aware).
  - `intent_classifier.rs`: Two-step confidence-gated intent discovery.
  - `guardrail.rs`: Multilingual prompt injection detection.
  - `ambient_soul.rs`: Background memory extraction and proactive initiative.
- `gateway-rust/src/feature/message_processor/`: The orchestrator brain.
  - `v2_orchestrator.rs`: Main entry point for inbound signals.
  - `v2_agent_orchestrator.rs`: The autonomous reasoning loop (Think-Act-Observe).
  - `history_utils.rs`: High-fidelity history formatting with quoted context.

## Core Workflows

1. **Context Hydration**: MediaAttachments (Image/Audio) are intercepted by the `MediaInterpreterService` and transcribed into text context before classification.
2. **Intent Classification**: Confirmed confidence-based classification using DEB thresholds. Short-circuits to "CHITCHAT" if below boundary.
3. **Dynamic Execution Boundaries (DEB)**: Behavior is tuned per-conversation. Always load `gateway_thresholds` from the `conversation_repo`.
4. **Agentic Reasoning**: Multi-turn autonomous loop using tools to fulfill user directives.
5. **Memory Consolidation**: Passive extraction of user facts into the RAG memory store (pgvector).

## Coding Guidelines

### Rust
- **Type Safety**: Use the repository layer instead of raw `sqlx` queries in handlers.
- **Caching**: Benefit from the `ConversationCache` in Redis for soul and bootstrap prompts.
- **Consistency**: Maintain the `UnifiedMessage` and `MessageSource` abstractions for cross-platform support.
- **WhatsApp**: clean LIDs/JIDs (remove `:xx`) for `external_id`; use phone-based IDs for `external_chat_id`.

### Svelte 5
- **Reactivity**: Use `$state` and `$derived` for clear UI state flow.
- **MQTT**: State synchronization is managed in `src/lib/stores/chat.svelte.ts`.
- **Components**: Follow the "Artifacts" design pattern (Side panels for detailed info).

### Database
- All schema changes must include a SQL migration in `gateway-rust/migrations`.
- Use JSONB for flexible configuration (like `gateway_thresholds` and `metadata`).

## Agentic Mandate
- **Context First**: You MUST read the root `README.md` and `gateway-rust/README.md` to orient yourself with the latest diagrams and workflow descriptions before modifying core logic.
- **Architecture First**: Understand the flow from Redis/MQTT through the Rust Services to the Postgres Schema.
- **Memory is Permanent**: Every interaction should ideally contribute to the long-term context of the user.
- **Verification**: Always run `cd gateway-rust && cargo check` after backend changes and `cd nomi_mobile && dart run build_runner build` for model/serializer updates.

**Principle**: Transition from a static assistant to a self-expanding Agentic Operating System.
