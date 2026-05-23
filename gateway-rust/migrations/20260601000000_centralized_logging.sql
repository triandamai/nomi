-- Centralized System Logging & Token Tracking
CREATE TABLE IF NOT EXISTS system_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    log_type VARCHAR(50) NOT NULL, -- 'swe_build', 'srp_reinforcement', 'system_event'
    target_slug VARCHAR(255),      -- plugin slug or tool handle
    event_step VARCHAR(50),        -- 'thinking', 'sandboxing', 'healing', 'success', 'failed'
    message TEXT NOT NULL,
    metadata JSONB DEFAULT '{}'::JSONB,
    prompt_tokens INTEGER DEFAULT 0 NOT NULL,
    completion_tokens INTEGER DEFAULT 0 NOT NULL,
    total_tokens INTEGER DEFAULT 0 NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- Optimization Index for Log Auditing
CREATE INDEX IF NOT EXISTS idx_system_logs_type_target ON system_logs(log_type, target_slug);
CREATE INDEX IF NOT EXISTS idx_system_logs_created ON system_logs(created_at DESC);
