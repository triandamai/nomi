-- Track specific costs for every turn
ALTER TABLE messages 
ADD COLUMN prompt_tokens INTEGER DEFAULT 0,
ADD COLUMN answer_tokens INTEGER DEFAULT 0,
ADD COLUMN total_tokens INTEGER DEFAULT 0;

-- Track lifetime cost of the conversation
ALTER TABLE conversations 
ADD COLUMN cumulative_tokens INTEGER DEFAULT 0;