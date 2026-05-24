-- Add external_id to users if not exists
ALTER TABLE users ADD COLUMN IF NOT EXISTS external_id TEXT UNIQUE;

-- Ensure index exists
CREATE INDEX IF NOT EXISTS idx_users_external_id ON users(external_id);
