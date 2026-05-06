-- Add user_id to messages table
ALTER TABLE messages ADD COLUMN user_id UUID REFERENCES users(id) ON DELETE SET NULL;

-- Migrate existing messages that belong to a user
-- Since sessions table has user_id and conversations has session_id, we can link them.
UPDATE messages m
SET user_id = s.user_id
FROM conversations c
JOIN sessions s ON c.session_id = s.id
WHERE m.conversation_id = c.id
AND m.role = 'user'
AND m.user_id IS NULL;
