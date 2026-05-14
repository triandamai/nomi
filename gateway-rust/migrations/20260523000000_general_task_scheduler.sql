-- Migration to turn reminders into a general-purpose task manager
ALTER TABLE reminders ADD COLUMN task_type VARCHAR(50) DEFAULT 'REMINDER';
ALTER TABLE reminders ADD COLUMN payload JSONB DEFAULT '{}';

-- Update legacy data: Convert existing row content strings into a JSON structure inside the new payload column: {"message": "content_text"}
UPDATE reminders 
SET payload = jsonb_build_object('message', content)
WHERE task_type = 'REMINDER';
