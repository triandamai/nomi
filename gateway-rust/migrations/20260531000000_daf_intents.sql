-- Add explicit intent support to DAF Staging
ALTER TABLE plugin_creation_suggestions 
ADD COLUMN IF NOT EXISTS intents TEXT[] DEFAULT '{}'::TEXT[] NOT NULL;
