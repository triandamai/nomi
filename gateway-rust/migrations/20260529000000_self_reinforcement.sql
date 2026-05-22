-- Core Static Plugin Optimization Shadow Table
CREATE TABLE IF NOT EXISTS static_plugin_reinforcements (
    plugin_slug VARCHAR(255) PRIMARY KEY,
    enriched_description TEXT NOT NULL,
    additional_rules TEXT[] DEFAULT '{}'::TEXT[] NOT NULL,
    learned_phrases TEXT[] DEFAULT '{}'::TEXT[] NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- FIFO Stable Array Append Utility Function to prevent infinite text growing
CREATE OR REPLACE FUNCTION id_stable_array_append_fifo(arr text[], element text, max_limit int) 
RETURNS text[] AS $$
BEGIN 
    IF element = '' OR element = ANY(arr) THEN 
        RETURN arr; 
    END IF;
    arr := array_append(arr, element);
    IF cardinality(arr) > max_limit THEN
        RETURN arr[cardinality(arr)-(max_limit-1) : cardinality(arr)];
    END IF;
    RETURN arr;
END; $$ LANGUAGE plpgsql;
