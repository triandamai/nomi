-- Create pending_media table for Media Checkpoint System
CREATE TABLE IF NOT EXISTS pending_media (
    conversation_id UUID PRIMARY KEY REFERENCES conversations(id) ON DELETE CASCADE,
    media_url TEXT NOT NULL,
    media_type TEXT NOT NULL,
    classification TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for cleanup or lookup if needed
CREATE INDEX IF NOT EXISTS idx_pending_media_created_at ON pending_media(created_at);
