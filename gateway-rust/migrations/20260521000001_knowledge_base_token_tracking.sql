-- Add token tracking to knowledge_base
ALTER TABLE knowledge_base
ADD COLUMN prompt_tokens INTEGER DEFAULT 0,
ADD COLUMN answer_tokens INTEGER DEFAULT 0,
ADD COLUMN total_tokens INTEGER DEFAULT 0;
