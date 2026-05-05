-- Create pairing_rooms table to handle linking external platforms (WA/Telegram) to web conversations
CREATE TABLE IF NOT EXISTS pairing_rooms (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    pairing_code VARCHAR(10) UNIQUE NOT NULL,
    user_id UUID REFERENCES users(id), -- This will be populated once the pairing is successful
    created_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ DEFAULT (NOW() + INTERVAL '10 minutes')
);

-- Index for faster lookup of pairing codes
CREATE INDEX IF NOT EXISTS idx_pairing_rooms_code ON pairing_rooms(pairing_code);
