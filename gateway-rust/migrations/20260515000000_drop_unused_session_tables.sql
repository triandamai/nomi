-- Drop unused session and auth_sessions tables
-- The system now uses JWT (stateless) and Redis for OTP/temporary state.

-- 1. Remove foreign key constraints and session-related columns
ALTER TABLE conversations DROP COLUMN IF EXISTS session_id;

-- 2. Drop the tables
DROP TABLE IF EXISTS auth_sessions;
DROP TABLE IF EXISTS sessions;
