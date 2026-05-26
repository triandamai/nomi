-- Up Migration

-- 1. Main Hierarchical Task Ledger
CREATE TABLE IF NOT EXISTS autonomous_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID REFERENCES conversations(id) ON DELETE CASCADE NOT NULL,
    source_message_id UUID REFERENCES messages(id) ON DELETE SET NULL, -- Saves where the instruction began
    sub_conversation_id UUID REFERENCES conversations(id) ON DELETE SET NULL, -- Spawned isolate for external chats (WA/TG Merchant conversations)
    title VARCHAR(255) NOT NULL,
    global_goal TEXT NOT NULL,
    status VARCHAR(50) DEFAULT 'running' NOT NULL, -- 'running', 'paused_for_input', 'completed', 'failed'
    current_step_index INT DEFAULT 0 NOT NULL,
    checkpoints JSONB NOT NULL DEFAULT '[]'::JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- 2. Audit & Timeline Logging Ledger
CREATE TABLE IF NOT EXISTS autonomous_task_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID REFERENCES autonomous_tasks(id) ON DELETE CASCADE NOT NULL,
    step_index INT NOT NULL,
    event_type VARCHAR(50) NOT NULL, -- 'step_start', 'tool_execution', 'outbound_msg', 'human_response', 'system_error', 'step_end'
    log_content TEXT NOT NULL,       -- Conversational slang details or system error details
    raw_payload JSONB DEFAULT '{}'::JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- 3. Performance Lookups Indexes
CREATE INDEX IF NOT EXISTS idx_autonomous_tasks_room_lookup ON autonomous_tasks(conversation_id, status);
CREATE INDEX IF NOT EXISTS idx_autonomous_task_logs_timeline ON autonomous_task_logs(task_id, created_at ASC);
