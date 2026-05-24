# Flutter Implementation Plan: Nomi Mobile Migration

## 🎯 Objective
Migrate the full technical and functional capabilities of the Nomi SvelteKit workspace into a high-fidelity, cross-platform Flutter application. The goal is to maintain the "Elite Engineering" aesthetic while providing native mobile performance.

---

## 🛠️ Technical Stack Recommendation

| Layer | Technology | Rationale |
| :--- | :--- | :--- |
| **State Management** | `Riverpod` | Best-in-class for global reactive state, matching the Svelte 5 "Store" logic. |
| **Networking** | `Dio` | Robust HTTP client for handling complex headers, interceptors, and error logging. |
| **Real-time** | `MQTT Client` | Native integration with Nomi's event bus (SSE-to-MQTT bridge). |
| **UI Framework** | `Flutter (Material 3 + Custom)` | Custom glassmorphism and animations to match the "Artifacts" design. |
| **3D Rendering** | `flutter_cube` or `webview_flutter` | For the RAG Knowledge Graph (Three.js integration). |
| **Markdown** | `flutter_markdown` | High-fidelity rendering of Nomi's technical responses. |
| **Authentication** | `flutter_secure_storage` | Secure JWT persistence for OTP-based login. |

---

## 📋 Feature & Component Audit

### 1. Core Pages (Routes)
*   **Authentication:** OTP Request & Verification screens.
*   **Chat Home:** Multi-conversation navigation and active session interface.
*   **RAG Explorer:** 3D Knowledge Graph with temporal filtering (Month/Year).
*   **System Utilities:** Centralized grid for all technical modules.
*   **Agent Factory:** Staging queue, build telemetry, and source code viewer.
*   **SRP Dashboard:** Reinforcement pass monitor and simulation interface.

### 2. High-Fidelity Artifacts (Components)
*   **ChatBubble:** 
    *   Markdown content rendering.
    *   Deep Thought (<thinking>) expandable blocks.
    *   Native Reply context UI.
    *   Media support (Images, Audio, Stickers).
*   **Artifact Cards:**
    *   `ReminderCard`: Real-time task status.
    *   `FinanceCard`: Transaction details with line-item breakdown.
    *   `ProposalCard`: Factory blueprint status and action buttons.
*   **Navigation:**
    *   Drawer-based Sidebar (Discord-style) for conversation switching.
    *   Floating Utility Grid.

### 3. Logic & State (Stores)
*   **Orchestrator Logic:** Real-time turn management (typing, tool use, synthesis).
*   **History Hydration:** High-fidelity `<MessageEntry>` parsing for context.
*   **Identity Sync:** Profile management and channel connectivity status.

---

## 🚀 Implementation Roadmap

### Phase 1: Infrastructure & Auth (DONE)
*   [x] Initialize Flutter project with Riverpod and Dio.
*   [x] Implement `AuthRepository` (OTP Request/Verify).
*   [x] Create Login UI with high-fidelity "Glass" inputs.
*   [x] Implement Secure Token Persistence.

### Phase 2: Chat & Real-time Connectivity (DONE)
*   [x] Integrate `MQTT` client for real-time telemetry.
*   [x] Build `ChatStore` logic (Streaming chunks, Thoughts, Tool status).
*   [x] Implement `ChatPage` with specialized `ChatBubble`.
*   [x] Add Media handling (Image/Audio downloads via Storage API).

### Phase 3: Live Artifacts & Utilities (Week 3)
*   [ ] Implement `UtilityGrid` dashboard.
*   [ ] Create `ReminderCard` and `FinanceCard` with self-hydrating logic.
*   [x] Implement Native Reply UI (Swipe-to-reply mechanics).
*   [x] Build `ConversationStore` for sidebar switching.

### Phase 4: Factory & SRP (Week 4)
*   [ ] Build `FactoryStore` (Proposals queue & Telemetry).
*   [ ] Implement `ProposalCard` and Build/Deploy action flows.
*   [ ] Create `MonacoEditor` alternative (Syntax highlighted code viewer).
*   [ ] Build SRP Simulation interface.

### Phase 5: RAG & Optimization (Week 5)
*   [ ] Integrate 3D Knowledge Graph (Three.js Webview or Native port).
*   [ ] Implement Monthly/Yearly temporal filters.
*   [ ] Perform Performance audit (FPS stabilization on large message logs).
*   [ ] UI/UX Polishing (Transitions, Haptic feedback).

---

## 🎨 Visual Identity Standards
*   **Colors:** Deep Slate (`#020617`), Emerald (`#10b981`), Indigo (`#6366f1`).
*   **FX:** Heavy use of `BackdropFilter` for blur and subtle `InnerShadow` for depth.
*   **Typography:** JetBrains Mono for technical IDs and Inter for conversational content.
