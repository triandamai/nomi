-- Secure Environment Variable Storage for Edge Plugins
CREATE TABLE IF NOT EXISTS environment_edge_functions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    function_id UUID NOT NULL REFERENCES edge_functions(id) ON DELETE CASCADE,
    key VARCHAR(255) NOT NULL,
    value TEXT NOT NULL,
    visibility VARCHAR(50) DEFAULT 'private', -- 'private' (not sent to client), 'public' (visible in IDE)
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    UNIQUE(function_id, key)
);

-- Trigger for updated_at
CREATE OR REPLACE FUNCTION update_edge_env_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER trg_update_edge_env_updated_at
    BEFORE UPDATE ON environment_edge_functions
    FOR EACH ROW
    EXECUTE FUNCTION update_edge_env_updated_at();
