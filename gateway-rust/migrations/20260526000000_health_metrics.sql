-- Create health metrics table
CREATE TABLE IF NOT EXISTS user_health_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    log_date DATE NOT NULL,
    metrics JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, log_date)
);

-- Add GIN index for JSONB lookups
CREATE INDEX IF NOT EXISTS idx_user_health_metrics_metrics ON user_health_metrics USING GIN (metrics);

-- Add index for user_id and log_date for faster lookups
CREATE INDEX IF NOT EXISTS idx_user_health_metrics_user_id_log_date ON user_health_metrics (user_id, log_date DESC);
