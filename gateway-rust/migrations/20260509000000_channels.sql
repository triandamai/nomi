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
