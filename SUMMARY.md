# Open Agent: Project Summary & Technical Documentation

## Project Summary

**Nomi** (formerly Arta or Open Agent) is a sophisticated AI-driven agentic workspace designed for seamless human-AI collaboration. It features a high-performance **Rust** backend (Gateway) that orchestrates interactions between **Google Gemini LLMs**, a **PostgreSQL** database with **pgvector** for long-term hierarchical memory, and multiple communication channels including a **SvelteKit** web UI, **WhatsApp**, and **Telegram**.

The system's core capability is its **autonomous orchestration loop**, which allows the agent to not only chat but also execute complex tools—such as querying its own database, searching the web, managing user memories, and even generating media—while maintaining a persistent and evolving persona (the "Soul").

---

## Technical Documentation

### 1. High-Level Architecture
The project follows a decoupled microservices architecture coordinated via a **Redis** message bus and **Postgres** shared state.

*   **Gateway (Rust):** The central "brain." It handles API requests, manages the LLM orchestration loop, executes tools, and streams real-time updates via Server-Sent Events (SSE).
*   **Channels (Rust):** A bridge service that connects external messaging platforms (WhatsApp, Telegram) to the internal Redis bus.
*   **Web UI (SvelteKit):** A modern "Artifacts" style interface built with Svelte 5, featuring real-time streaming, markdown rendering, and 3D knowledge graph visualization.
*   **Database (PostgreSQL):** Stores relational data and high-dimensional vectors (768-dim) for semantic search.
*   **Redis:** Facilitates inter-service communication (e.g., notifying the Gateway of a new WhatsApp message) and presence tracking.

### 2. Backend (gateway-rust)
Built with **Axum** and **Tokio**, the gateway is designed for high concurrency and type safety.

*   **Orchestration Engine:** Implements a multi-turn autonomous loop using the `gemini-rust` crate. It handles multi-step tool execution before returning a final answer.
*   **Tool System:** The agent can dispatch tools including:
    *   `execute_sql_query`: Direct DB interaction for data retrieval.
    *   `web_search` & `read_web_page`: Real-time information gathering.
    *   `update_knowledge_base`: Appending information to the RAG system.
    *   `update_nomi_soul`: Persisting persona changes.
*   **Real-time Egress:** Uses a custom `SseBroadcaster` to stream thoughts (inside `<thinking>` tags) and answers separately to the UI.

### 3. Frontend (ui-sveltekit)
A reactive dashboard leveraging **Svelte 5** and **Tailwind CSS**.

*   **State Management:** Uses Svelte stores for handling chat history, connection status, and user sessions.
*   **Visualizations:** Includes `3d-force-graph` and `Three.js` to visualize the agent's memory and knowledge relationships.
*   **Real-time:** Establishes a persistent SSE connection to the Gateway to receive incremental LLM updates and system notifications.

### 4. Memory & RAG (Retrieval-Augmented Generation)
Open Agent uses a hierarchical memory structure:

*   **Semantic Memory:** Uses `pgvector` with 768-dimensional embeddings (matching `gemini-embedding-004`).
*   **Indexing:** Utilizes **HNSW** (Hierarchical Navigable Small World) indexes for fast vector similarity searches.
*   **Consolidation:** Periodically triggers memory consolidation to "summarize" and "compress" old conversations into the long-term knowledge base.

### 5. Integration Channels (channel-rust)
This service acts as a stateless bridge:
*   **WhatsApp:** Integrates with local SQLite databases or webhooks to sync messages.
*   **Telegram:** Uses the Telegram Bot API to receive and send messages.
*   **Redis Bus:** Translates external messages into `UnifiedMessage` payloads and publishes them to `nomi:inbound`.

### 6. Deployment & Infrastructure
*   **Docker:** Managed via `docker-compose.yml` for local development.
*   **Environment:** Relies on `.env` files for API keys (Gemini, Database URLs, JWT secrets).
*   **CI/CD:** GitHub Workflows for automated building and pushing of images.
