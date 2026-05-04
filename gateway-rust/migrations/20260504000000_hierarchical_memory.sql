-- Migration: Add HNSW index to knowledge_base and ensure vector size is correct (though it's 1536 in initial schema, user asked for 768 for gemini-embedding-2, but initial schema says 1536. I will stick to 1536 as per initial schema unless I see a reason to change, but the prompt specifically says 768. I'll use 768 for the summary embeddings if needed or stick to the schema's 1536. Actually, gemini-embedding-004 is 768. I will update knowledge_base to 768 if it's meant for gemini-embedding-2).
-- Wait, the initial schema already has vector(1536). Changing it might be breaking if there's data. 
-- However, the prompt says "Ensure embedding is vector(768) (matching gemini-embedding-2)".

-- Drop index if exists (just in case)
DROP INDEX IF EXISTS knowledge_base_embedding_idx;

-- Add HNSW index
CREATE INDEX ON knowledge_base USING hnsw (embedding vector_cosine_ops);

-- If we need to change the vector size, we'd have to drop and recreate the column if there's data, or just alter it.
-- ALTER TABLE knowledge_base ALTER COLUMN embedding TYPE vector(768);
-- But I'll leave it as is for now if it already exists, or just ensure the new index is there.
-- The user instructions say: "Ensure embedding is vector(768)". 

ALTER TABLE knowledge_base ALTER COLUMN embedding TYPE vector(768);
ALTER TABLE messages ALTER COLUMN embedding TYPE vector(768);
