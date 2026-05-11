pub struct PromptRegistry;

impl PromptRegistry {
    // --- SYSTEM PROMPTS ---

    pub fn orchestrator_instructions() -> &'static str {
        "ALL internal reasoning, analysis, and strategy MUST be contained within <thinking>...</thinking> tags. NEVER leak your internal monologue outside these tags.\n\n\
        INTERNAL REASONING (inside <thinking>) must be strictly atomic and technical. **STRICT RULE: Your <thinking> block must be under 200 characters. Use bullet points or short technical phrases. NO PROSE.**\n\n\
        Focus only on: [Intent] -> [Action] -> [Status].\n\n\
        If a user gives an instruction (like log expense or make sticker) but no media is attached to the current message, use the `get_latest_media_context` tool to retrieve the pending file.\n\n\
        If a tool fails, state the error and the fix, then immediately call the tool again.\n\n\
        You are operating in a multi-turn tool-use loop. You MUST wait to gather all necessary data from your tools before providing a final response to the user. Do not answer prematurely. Acknowledge and integrate all tool results into your final answer."
    }

    pub fn tool_usage_guidelines() -> &'static str {
        "**Direct Messaging Flow:**

        - If a user says 'Tell [Name] [Message]', FIRST use `search_users` to find the correct JID.
        - If `search_users` returns multiple results, ask the user for clarification (e.g., 'I found two Billys. Did you mean Billy the Rider or Billy the Coder?').
        - Once the unique JID is identified, use `send_direct_message(recipient_jid, content)`.
        - After sending, confirm to the sender: 'Done! I've sent that message to [Name]. 🚀'

        **Sticker Generation:**

        - If a user asks to turn an image into a sticker (e.g., 'Make this a sticker', 'Sticker-in', 'Jadikan sticker'), use the `make_sticker` tool.

        - If no URL is provided, the tool will automatically use the most recent image from the conversation.

        **Expense Logging:**
        - If a user provides an expense (e.g., 'Log this as expense', 'I spent $50 at Starbucks'), use the `log_expense` tool.
        - If you have an image URL (from current message or pending context), include it in the tool parameters."
    }

    pub fn memory_consolidation_summarizer(conversation_history: &str) -> String {
        format!(
            "Analyze the following conversation and return a JSON object with:
            1. 'summary': A concise summary of permanent facts and project context.
            2. 'nodes': An array of entities ({{'id': 'unique_id', 'label': 'Entity Name', 'node_type': 'Technology|Project|Person|Organization|Vehicle|Location|Peak|Language|Framework|MaintenanceLog|Concept|Event'}}).
            3. 'edges': An array of relationships ({{'source': 'node_id', 'target': 'node_id', 'relationship': 'Description'}}).

            Rules:

            - NEVER create a node with id 'summary' or that represents the conversation summary itself.
            - Extract individual entities.
            - Reuse IDs.
            - 'id' should be snake_case.
            Conversation:

            {}",
            conversation_history
        )
    }

    // --- INTERACTION PROMPTS ---

    pub fn zero_intent_clarification() -> &'static str {
        "[SYSTEM: User uploaded an image without text. Please ask the user for clarification on what this image is for (e.g., log an expense, save to memories, or make a sticker). Keep it witty and helpful.]"
    }

    pub fn pending_media_context(url: &str) -> String {
        format!(
            "### Pending Media Context\n
             [SYSTEM: There is a pending image from the previous turn: {}. If the user's current request implies an action on an image (like 'save as expense', 'make a sticker', or 'save to memory'), use this URL.]",
            url
        )
    }

    pub fn media_context_expense(merchant: &str, total: &str, category: &str, items: &str) -> String {
        format!(
            "[SYSTEM: User uploaded an expense receipt. Merchant: {}, Total: {}, Category: {}. Items: {}]",
            merchant, total, category, items
        )
    }

    pub fn media_context_maintenance(parts: &str, details: &str) -> String {
        format!(
            "[SYSTEM: User uploaded motorcycle maintenance record. Parts: {}. Details: {}]",
            parts, details
        )
    }

    pub fn media_context_technical(summary: &str) -> String {
        format!(
            "[SYSTEM: User uploaded a technical document. Summary: {}]",
            summary
        )
    }

    pub fn media_context_nature() -> &'static str {
        "[SYSTEM: User uploaded a nature photo.]"
    }

    pub fn media_context_other() -> &'static str {
        "[SYSTEM: User uploaded an image (uncategorized).]"
    }

    // --- TOOLS PROMPTS ---

    pub fn media_classification() -> &'static str {
        "Classify this image into exactly one of these categories: EXPENSE_RECEIPT, MOTORCYCLE_MAINTENANCE, TECHNICAL_DOC, NATURE, or OTHER. Return ONLY the category name."
    }

    pub fn expense_extraction() -> &'static str {
        "Extract expense data from this receipt. Return a JSON object with: merchant, total (number), tax (number or null), service (number or null), discount (number or null), items (array of {name, quantity, amount}), and category. Return ONLY the JSON.\n\n\
        RULES:\n\
        - DO NOT GUESS missing data.\n\
        - DO NOT use 'Lorem Ipsum' or placeholder text.\n\
        - If crucial data (especially the total amount) is missing or unreadable, return an error message describing what is missing instead of a JSON object."
    }

    pub fn maintenance_extraction() -> &'static str {
        "Extract motorcycle maintenance data. Return a JSON object with: part_names (array of strings) and service_details. Return ONLY the JSON."
    }

    pub fn technical_doc_summarization() -> &'static str {
        "Summarize the content of this technical document. Focus on key specifications, diagrams, or instructions."
    }

    // --- ERROR MESSAGES & STATUS ---

    pub fn status_analyzing_receipt() -> &'static str {
        "Nomi is analyzing your receipt..."
    }

    pub fn status_thinking() -> &'static str {
        "Nomi is thinking..."
    }

    pub fn status_expense_logged(merchant: &str, total: &str) -> String {
        format!("Expense at {} for {} logged successfully! 💸", merchant, total)
    }

    pub fn error_general_trouble() -> &'static str {
        "We having trouble, meanwhile we on fixing, you can try again later."
    }

    pub fn error_account_exists() -> &'static str {
        "Account already exists. Use /login."
    }
}
