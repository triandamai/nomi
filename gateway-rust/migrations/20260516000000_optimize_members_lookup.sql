-- Add index to conversation_members(user_id) for faster lookups
CREATE INDEX IF NOT EXISTS idx_conversation_members_user_id ON conversation_members(user_id);
