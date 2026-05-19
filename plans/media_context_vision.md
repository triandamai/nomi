# Plan: Image Handling & Vision Analysis Implementation

This plan implements the user's request:
1. Nomi asks for intent when an image is uploaded without text.
2. Nomi can analyze images when requested.
3. Image URLs are clearly visible in the history context.

## Implementation Steps

### 1. Tool Models (`tools_model.rs`)
- Add `AnalyzeImageParameters { image_url: Option<String>, instruction: String }`.
- Add `AnalyzeImageResponse { content: String }`.

### 2. Tool Implementation (`mod.rs`)
- Add `AnalyzeImage` to `ArtaTool`.
- Implement `analyze_image` in `ToolDispatcher`:
    - Fetch image using `fetch_image_from_storage`.
    - Send the image BLOB and prompt to Gemini.
- Register `analyze_image` in `generate_tool_for_prompt`.

### 3. Agent Dispatcher (`agent/mod.rs`)
- Add `analyze_image` to the tool execution dispatcher.

### 4. Prompt Refinement (`prompts.rs`)
- Update `orchestrator_instructions` to guide image handling.
- Refine `zero_intent_clarification`.

### 5. Orchestrator Logic (`v2_orchestrator.rs`)
- Refine zero-intent guard to provide clear placeholder text.

## Verification
- Test image upload without text -> Nomi should ask for instructions.
- Test "Analyze this image" -> Nomi should use the new tool.
