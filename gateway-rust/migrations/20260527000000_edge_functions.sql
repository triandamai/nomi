CREATE TABLE edge_functions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug VARCHAR(255) UNIQUE NOT NULL,      -- URL and routing friendly handle (e.g., 'crypto_tracker') use '_' for slug
    name VARCHAR(255) NOT NULL,             -- Human readable label
    description TEXT NOT NULL,              -- Documentation context fed to the LLM agent
    schema_json JSONB NOT NULL,             -- JSON Schema describing required argument parameters
    rules_text TEXT NOT NULL,               -- Operational constraints for model generation boundaries
    script_code TEXT NOT NULL,              -- The raw TypeScript source code string
    intents TEXT[] NOT NULL DEFAULT '{}',   -- List of intents to route the request to this plugin
    rag_id UUID REFERENCES knowledge_base(id) ON DELETE SET NULL, -- Reference to the knowledge base for semantic routing
    version INT DEFAULT 1 NOT NULL,         -- Incremental change tracker
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);
