-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS vector;



-- User
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT,
    wa_id TEXT,
    tele_id TEXT,
    external_id TEXT UNIQUE NOT NULL, -- WA JID or Telegram ID
    display_name TEXT,
    email TEXT,
    is_verified boolean DEFAULT false,
    role TEXT DEFAULT 'user'::TEXT,
    auth_metadata jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
-- Conversations table
CREATE TABLE IF NOT EXISTS conversations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title TEXT,
    soul_content TEXT, -- Saved as Markdown
    bootstrap_content TEXT, -- Saved as Markdown
    channel_group_id TEXT,
    user_id UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Channels mapping for external integration
CREATE TABLE IF NOT EXISTS channels (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    channel_type TEXT NOT NULL, -- 'whatsapp', 'telegram'
    external_id TEXT NOT NULL, -- WA JID or Telegram User ID
    external_chat_id TEXT NOT NULL, -- WA Group JID or Telegram Chat ID
    conversation_id UUID REFERENCES conversations(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(channel_type, external_chat_id)
);


-- Messages table
CREATE TABLE IF NOT EXISTS messages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    conversation_id UUID REFERENCES conversations(id),
    role TEXT NOT NULL, -- 'user', 'assistant', 'system'
    content TEXT NOT NULL, -- Saved as Markdown for user
    thought TEXT DEFAULT '',
    embedding vector(1536), -- For RAG (Gemini embedding size)
    user_id UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Knowledge Base (for RAG)
CREATE TABLE IF NOT EXISTS knowledge_base (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    content TEXT NOT NULL,
    embedding vector(1536),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Soul History
CREATE TABLE IF NOT EXISTS soul_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    soul_content TEXT NOT NULL,
    bootstrap TEXT,
    change_reason TEXT NOT NULL,
    version_number INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (conversation_id, version_number)
);

-- Many-to-many relationship for conversations (optional but good for future group chats)
CREATE TABLE IF NOT EXISTS conversation_members (
    conversation_id UUID REFERENCES conversations(id),
    user_id UUID REFERENCES users(id),
    joined_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (conversation_id, user_id)
);


CREATE INDEX IF NOT EXISTS idx_soul_history_conversation_id_created_at
    ON soul_history (conversation_id, created_at DESC);


CREATE UNIQUE INDEX idx_conversations_channel_group_id_unique
    on conversations (channel_group_id)
    where (channel_group_id IS NOT NULL);
