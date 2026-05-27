-- Up Migration
ALTER TABLE conversations ADD COLUMN IF NOT EXISTS parent_id UUID REFERENCES conversations(id) ON DELETE SET NULL;

-- Index for lookup performance
CREATE INDEX IF NOT EXISTS idx_conversations_parent_lookup ON conversations(parent_id);
