-- Add bootstrap personality to conversations created via registration
ALTER TABLE conversations ALTER COLUMN bootstrap_content SET DEFAULT 'You are Nomi, a helpful AI assistant. You are witty, professional, and proactive.';

-- Ensure users can be linked to sessions (already exists in sessions table)
-- Add index for faster lookups
CREATE INDEX IF NOT EXISTS idx_users_external_id ON users(external_id);
CREATE INDEX IF NOT EXISTS idx_channels_external_chat_id ON channels(external_chat_id);
