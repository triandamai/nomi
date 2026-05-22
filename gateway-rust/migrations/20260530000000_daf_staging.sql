-- Distributed Agent Factory (DAF) Staging Table
CREATE TABLE IF NOT EXISTS plugin_creation_suggestions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    schema_json JSONB NOT NULL,
    how_it_works TEXT NOT NULL,
    compiled_code TEXT DEFAULT '' NOT NULL,
    status VARCHAR(50) DEFAULT 'pending' NOT NULL, -- pending, approved, processing, ready, failed, deployed
    error_logs TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- Optimization Index
CREATE INDEX IF NOT EXISTS idx_plugin_suggestions_status ON plugin_creation_suggestions(status);
