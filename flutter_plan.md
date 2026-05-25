# Flutter Implementation Plan: Nomi Mobile Migration

## 🎯 Objective
Migrate the full technical and functional capabilities of the Nomi SvelteKit workspace into a high-fidelity, cross-platform Flutter application. The goal is to maintain a professional, high-density industrial aesthetic ("Artifacts" design) while providing an ultra-responsive, offline-first experience.

## 🛠️ Architecture
*   **Framework:** Flutter (3.29+)
*   **State Management:** Riverpod (Runes-parity logic)
*   **Networking:** Dio (Standardized `ApiResponse` handling)
*   **Persistence:** Drift (SQLite) for high-performance Offline-First core.
*   **Real-time:** MQTT (Global UI Isolate Service)
*   **Design System:** Glassmorphism (Liquid Glass), Deep Slate (#020617), Emerald (#10b981).

## 🚀 Roadmap

### Phase 1: Infrastructure & Auth [DONE]
*   [x] Project scaffolding & High-fidelity theme.
*   [x] Secure Storage (JWT & Device ID persistence).
*   [x] OTP Auth Pipeline (Channel-aware).
*   [x] Profile Management.

### Phase 2: Chat & Real-time Connectivity [DONE]
*   [x] Native Message Replies (Recursive Join).
*   [x] Multimodal Rendering (Images, Stickers, Video, Audio).
*   [x] Global MQTT Service with Structured Client IDs.
*   [x] Reverse-descending reactive chat list.
*   [x] Real-time Presence (Bouncing Typing Indicators, Streaming Thoughts).
*   [x] Offline-First Persistent Core (SQLite/Drift).

### Phase 3: Live Artifacts & Utilities [IN PROGRESS]
*   [x] Utility 1: Reminders (Durable Offline Tasks).
*   [x] Utility 2: Money Tracking (Financial History & Precision metrics).
*   [x] Utility 3: Health & Vitality Hub (Biometric Trends & Charts).
*   [x] Utility 4: System Blueprint (High-Fidelity WebView Graph).
*   [x] Utility 5: Edge Plugins (Registry Console).
*   [ ] **Utility 5.1: Plugin Editor (High-Fidelity IDE Experience)**.
    *   [ ] Monaco Editor WebView Bridge (TS/IntelliSense).
    *   [ ] Dual-Pane Metadata Management (Intents/Rules/Schema).
    *   [ ] Plugin Execution & Log Streaming.
*   [ ] Utility 6: Factory Console (SRP Operations & Log Aggregate).

### Phase 4: Factory Console & SRP Dashboard Port [TODO]
*   [ ] Master operation view.
*   [ ] Technical log aggregation.

### Phase 5: Knowledge Graph Enhancements [TODO]
*   [ ] Advanced interactions within the Blueprint WebView.

## 🎨 Aesthetic Mandates
*   **Theme:** Deep Slate (`#020617`), Emerald (`#10b981`), Indigo (`#6366f1`).
*   **FX:** Heavy use of `BackdropFilter` (sigma 40) for "Liquid Glass".
*   **Typography:** JetBrains Mono for technical IDs and Inter for content.
