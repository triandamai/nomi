-- Add dynamic threshold orchestration configuration columns to your existing conversation matrix layout
ALTER TABLE conversations 
ADD COLUMN IF NOT EXISTS gateway_thresholds JSONB DEFAULT '{
    "interaction_gate": 0.6,
    "intent_classification": 0.4,
    "guardrails": 0.65
}'::JSONB NOT NULL;

-- Optimize index checks to guarantee immediate relational lookups
CREATE INDEX IF NOT EXISTS idx_conversations_gateway_thresholds ON conversations USING gin (gateway_thresholds);
