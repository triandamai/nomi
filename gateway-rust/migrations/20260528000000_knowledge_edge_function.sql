-- Create join table for many-to-many relationship between edge functions and knowledge base
CREATE TABLE IF NOT EXISTS knowledge_edge_function (
    knowledge_id UUID REFERENCES knowledge_base(id) ON DELETE CASCADE,
    edge_function_id UUID REFERENCES edge_functions(id) ON DELETE CASCADE,
    PRIMARY KEY (knowledge_id, edge_function_id)
);

-- Remove the old single rag_id link as we now support multiple via the join table
ALTER TABLE edge_functions DROP COLUMN IF EXISTS rag_id;
