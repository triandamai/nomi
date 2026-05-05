# AGENTS.md

## Project Overview

Open Agent is an agentic workspace built with:

- Backend: Rust, Axum, Tokio, SQLx, PostgreSQL, pgvector
- Frontend: SvelteKit, Svelte 5, TypeScript, Tailwind CSS, Lucide icons
- Realtime: Server-Sent Events (SSE) for streamed thoughts, tool events, and answers
- Data: PostgreSQL migrations live in `gateway-rust/migrations`

When changing behavior, think through the full path: database schema, Rust models/handlers/services, SSE/API contracts, Svelte stores, and UI components.

## Repository Layout

- `gateway-rust/`: Rust Axum API gateway
- `gateway-rust/src/routes.rs`: top-level API routing
- `gateway-rust/src/main.rs`: application startup and shared state setup
- `gateway-rust/src/common/`: shared API response, SSE, agent, and tool helpers
- `gateway-rust/src/feature/`: feature modules such as conversation, graph, realtime
- `gateway-rust/src/rag/`: retrieval and memory-related code
- `gateway-rust/migrations/`: SQLx/Postgres migrations
- `ui-sveltekit/`: SvelteKit frontend
- `ui-sveltekit/src/lib/api/`: frontend API client code
- `ui-sveltekit/src/lib/stores/`: Svelte state stores
- `ui-sveltekit/src/lib/components/`: reusable Svelte components
- `ui-sveltekit/src/routes/`: SvelteKit routes and page-level styles
- `docker-compose.yml`: local pgvector/Postgres service

## Common Commands

Backend:

```sh
cd gateway-rust
cargo fmt
cargo check
cargo test
cargo run
```

Frontend:

```sh
cd ui-sveltekit
npm run check
npm run build
npm run dev
```

Database:

```sh
docker compose up -d db
cd gateway-rust
sqlx migrate run
```

Use `gateway-rust/.env.example` as the local backend environment template.

## Coding Guidelines

Rust:

- Prefer explicit types at API and database boundaries.
- Use `anyhow` for application-level error propagation unless a typed error is required.
- Keep Axum handlers small; put feature logic in the relevant `feature` or `rag` module.
- Keep SQLx query types aligned with migrations and Rust response models.
- Preserve async correctness; avoid blocking operations inside Tokio request handlers.
- Run `cargo fmt` after Rust edits.

SvelteKit:

- Use Svelte 5 patterns already present in the repo.
- Keep shared state in `src/lib/stores` and API calls in `src/lib/api`.
- Prefer small, focused components in `src/lib/components`.
- Use Tailwind utilities consistently with the existing UI.
- Run `npm run check` after TypeScript or Svelte edits.

SSE and API contracts:

- Treat streamed events as public contracts between `gateway-rust` and `ui-sveltekit`.
- When changing SSE payloads, update Rust emitters/models, frontend parsing, and UI rendering together.
- Avoid large unstructured text blobs where typed events would be clearer.

Database and RAG:

- Any schema change requires a new migration in `gateway-rust/migrations`.
- For memory/RAG features, consider chunking, embedding storage, retrieval filters, and graph/memory relationships together.
- Keep pgvector usage explicit and document assumptions about embedding dimensions or providers in code or migrations.

## Agent Workflow

- Inspect existing patterns before editing.
- Do not overwrite unrelated user changes.
- Prefer minimal, focused diffs that preserve existing architecture.
- Update tests or checks when behavior changes.
- If a change touches both backend and frontend, verify both sides when possible.
- Never commit changes unless the user explicitly asks.

## Local Notes

- The project name in prior instructions is Open Agent.
- The product direction uses an artifacts-style UI where generated code and previews can appear in a side panel.
- Existing GEMINI.md contains persona-oriented guidance; this AGENTS.md is the operational source for coding-agent behavior.
