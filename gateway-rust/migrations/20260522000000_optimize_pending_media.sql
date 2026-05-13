-- Optimize pending_media table
-- conversation_id is already PRIMARY KEY, which has a Unique Index.

-- Task 1: No schema changes needed for unique index as PK is already unique.
-- We will modify the Rust code to remove created_at = now() from the UPDATE clause.

-- Cleanup: The user requested Task 3: Cleanup Policy in Rust, but we can also ensure the index on created_at is there (it is).
