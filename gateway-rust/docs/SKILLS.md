# Nomi Skill Creation Protocol (v1.0)

This document serves as the technical blueprint for the **Distributed Agent Factory (DAF)** and the **SWE Agent**. All synthesized plugins MUST adhere to these architectural standards.

## 🏗️ Execution Lifecycle (Edge Runner)

Nomi uses a high-performance **Bun/V8 Sandbox** to execute dynamic plugins. This environment is isolated from the main Rust gateway for safety.

1.  **Injection**: The raw TypeScript source is piped into the sandbox.
2.  **Context Hydration**: The system injects `incoming` (message metadata) and `workspace` (conversation metadata) context.
3.  **Execution**: The `run(args)` entrypoint is invoked with the provided parameters.
4.  **Telemetry**: All `console.log` and `console.error` calls are captured and streamed back to the Factory Console.

## 📜 Entrypoint Standard

Every plugin MUST export a default function matching this exact signature:

```typescript
/**
 * @param args - The parameters defined in your tool's JSON schema.
 * @returns - A string (conversational response) or a JSON object.
 */
export default async function run(args: any) {
  // 1. Extract variables from args
  const { query, category } = args;

  try {
    // 2. Implementation logic
    // ...
    
    return `Successfully processed ${query}! 🚀`;
  } catch (error) {
    console.error("Execution failed:", error);
    throw error;
  }
}
```

## 🛠️ Built-in Capabilities

Dynamic plugins can interact with the Nomi ecosystem using standard `fetch` calls to the internal bridge.

### 1. External Data Access
You have full access to the `fetch` API. You can scour the web or hit external APIs.
*   **Safety**: Ensure all external calls use `https`.

### 2. Internal Bridge API
You can communicate back to the Gateway using the `TEMP_BRIDGE_TOKEN` (automatically injected).
*   **Endpoint**: `${api_base_url}/internal/rpc/...`
*   **Authorization**: Bearer token via `bridge_token`.

## 🛡️ Coding Guardrails

1.  **Atomic Execution**: Scripts should be optimized for speed (target < 500ms).
2.  **No Global State**: Do not rely on persistent global variables; use the `knowledge_base` for long-term memory.
3.  **Graceful Errors**: Always use `try/catch` blocks and provide meaningful error messages to `console.error`.
4.  **No Obfuscation**: Code must be readable and well-commented for the Admin Review pass.

## 🎨 Schema Mapping

Your `schema_json` defines how the model sees your tool.
*   **Descriptions**: Use high-fidelity descriptions with examples to improve intent matching.
*   **Types**: Strictly define `string`, `number`, `integer`, or `boolean`.

---
*This protocol is synchronized with the Nomi Gateway v2.0 runtime.*
