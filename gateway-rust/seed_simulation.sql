-- Nomi Autonomous HTO Multi-Party Simulation Seeding Script
-- Use this query to establish a real Owner, Target, channels, sub-conversations, and active autonomous task for testing.

-- 1. Insert Owner User (Trian)
INSERT INTO users (id, name, display_name, email, role, is_verified)
VALUES (
    'a1b2c3d4-e5f6-7a8b-9c0d-1e2f3a4b5c6d', 
    'trian', 
    'Trian', 
    'trian@nomi.ai', 
    'user', 
    true
)
ON CONFLICT (id) DO NOTHING;

-- 2. Insert Target User (Triandamai)
INSERT INTO users (id, name, display_name, email, role, is_verified)
VALUES (
    'f1e2d3c4-b5a6-7988-9700-112233445566', 
    'triandamai', 
    'Triandamai', 
    'triandamai@nomi.ai', 
    'user', 
    true
)
ON CONFLICT (id) DO NOTHING;

-- 3. Insert Parent Conversation (Owner Chat with Nomi)
INSERT INTO conversations (id, title, conversation_type, soul_content, bootstrap_content)
VALUES (
    '99999999-8888-7777-6666-555555555555',
    'Trian & Nomi (Main Room)',
    'private',
    'You are Nomi, Trian''s highly capable AI teammate. Use friendly teammate slang like "gua", "aman", "otw", "sip".',
    'Core HTO operational rules and persona details.'
)
ON CONFLICT (id) DO NOTHING;

-- 4. Insert Sub-Conversation (Sub-Chat with Triandamai)
INSERT INTO conversations (id, title, conversation_type, soul_content, bootstrap_content)
VALUES (
    '88888888-7777-6666-5555-444444444444',
    'Nomi & Triandamai (WhatsApp Sub-Chat)',
    'channel_subchat',
    'You are Nomi, Trian''s AI teammate representing him in a conversation with Triandamai.',
    'Negotiate politely but directly.'
)
ON CONFLICT (id) DO NOTHING;

-- 5. Map Conversation Members for Parent Room
INSERT INTO conversation_members (conversation_id, user_id)
VALUES (
    '99999999-8888-7777-6666-555555555555',
    'a1b2c3d4-e5f6-7a8b-9c0d-1e2f3a4b5c6d'
)
ON CONFLICT (conversation_id, user_id) DO NOTHING;

-- 6. Map Channels for Parent Room (simulating Owner Telegram/Web channel)
INSERT INTO channels (id, channel_type, external_id, external_chat_id, conversation_id)
VALUES (
    '77777777-6666-5555-4444-333333333333',
    'telegram',
    'telegram_owner_123',
    'telegram_owner_123',
    '99999999-8888-7777-6666-555555555555'
)
ON CONFLICT (channel_type, external_chat_id) DO NOTHING;

-- 7. Map Channels for Target Sub-Conversation (Triandamai JID WhatsApp channel)
INSERT INTO channels (id, channel_type, external_id, external_chat_id, conversation_id)
VALUES (
    '66666666-5555-4444-3333-222222222222',
    'whatsapp',
    'triandamai@s.whatsapp.net',
    'triandamai@s.whatsapp.net',
    '88888888-7777-6666-5555-444444444444'
)
ON CONFLICT (channel_type, external_chat_id) DO NOTHING;

-- 8. Insert a Demo Autonomous Task currently waiting for Triandamai's feedback
INSERT INTO autonomous_tasks (id, conversation_id, sub_conversation_id, title, global_goal, status, current_step_index, checkpoints)
VALUES (
    '44444444-3333-2222-1111-000000000000',
    '99999999-8888-7777-6666-555555555555',
    '88888888-7777-6666-5555-444444444444',
    'Acquire Triandamai''s Email',
    'Ask Triandamai for his email address so we can schedule the tomorrow interview.',
    'waiting_external_feedback',
    0,
    '[{"step": "Ask Triandamai on WhatsApp", "status": "completed"}, {"step": "Receive response and clarify if needed", "status": "pending"}]'::jsonb
)
ON CONFLICT (id) DO NOTHING;

-- 9. Insert initial message from Nomi to Triandamai in the sub-conversation
INSERT INTO messages (id, conversation_id, role, content, user_id)
VALUES (
    '11111111-2222-3333-4444-555555555555',
    '88888888-7777-6666-5555-444444444444',
    'assistant',
    'Hi Triandamai, Trian asked me to reach out and get your email address for tomorrow''s job interview. Could you share it with me?',
    'f1e2d3c4-b5a6-7988-9700-112233445566'
)
ON CONFLICT (id) DO NOTHING;
