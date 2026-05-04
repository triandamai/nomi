-- 1. Ensure the pgvector extension is active
CREATE EXTENSION IF NOT EXISTS vector;

-- 2. Drop the existing index if it exists (since we are changing types)
DROP INDEX IF EXISTS knowledge_base_embedding_idx;

-- 3. Alter the column to halfvec(3072)
-- We cast the existing data to halfvec to prevent data loss
ALTER TABLE knowledge_base
ALTER COLUMN embedding TYPE halfvec(3072)
USING embedding::halfvec(3072);

-- 4. Create the HNSW index using halfvec_cosine_ops
-- This bypasses the 2000-dimension limit for standard float vectors
CREATE INDEX knowledge_base_embedding_idx ON knowledge_base
    USING hnsw (embedding halfvec_cosine_ops);