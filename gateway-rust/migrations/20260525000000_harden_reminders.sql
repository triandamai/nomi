-- Add error_log to reminders for better observability
ALTER TABLE reminders ADD COLUMN error_log TEXT;
