-- Create reminders table
CREATE TABLE reminders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    conversation_id UUID REFERENCES conversations(id), -- Nullable for global reminders
    content TEXT NOT NULL,
    due_at TIMESTAMPTZ NOT NULL,
    
    -- Recurrence Logic
    frequency TEXT DEFAULT 'once', -- 'once', 'daily', 'weekly', 'monthly'
    interval_count INT DEFAULT 1,
    max_repeats INT DEFAULT NULL, -- Max times to repeat before archiving
    current_runs INT DEFAULT 0,
    
    -- Status & Noise Control
    status TEXT DEFAULT 'pending', -- 'pending', 'completed', 'archived', 'snoozed'
    snooze_count INT DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_reminders_due_at_status ON reminders(due_at, status) WHERE status = 'pending';
CREATE INDEX idx_reminders_user_id ON reminders(user_id);
