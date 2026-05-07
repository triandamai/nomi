ALTER TABLE knowledge_base ADD COLUMN conversation_id UUID REFERENCES conversations(id) ON DELETE CASCADE;
CREATE INDEX idx_knowledge_base_conversation_id ON knowledge_base(conversation_id);
