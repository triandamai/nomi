CREATE TABLE IF NOT EXISTS soul_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    soul_content TEXT NOT NULL,
    change_reason TEXT NOT NULL,
    version_number INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (conversation_id, version_number)
);

CREATE INDEX IF NOT EXISTS idx_soul_history_conversation_id_created_at
    ON soul_history (conversation_id, created_at DESC);
