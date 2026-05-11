-- Add migration script here
CREATE TABLE IF NOT EXISTS categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS money_tracking (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    conversation_id UUID REFERENCES conversations(id),
    category_id UUID REFERENCES categories(id),
    category TEXT, -- Fallback or custom category name
    merchant_name TEXT,
    total_amount NUMERIC(15, 2) NOT NULL DEFAULT 0,
    tax NUMERIC(15, 2) DEFAULT 0,
    service NUMERIC(15, 2) DEFAULT 0,
    discount NUMERIC(15, 2) DEFAULT 0,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS money_tracking_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    money_tracking_id UUID NOT NULL REFERENCES money_tracking(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 1,
    total_amount NUMERIC(15, 2) NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS money_tracking_summary (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    total_expenses NUMERIC(15, 2) DEFAULT 0,
    total_income NUMERIC(15, 2) DEFAULT 0,
    expenses_up_trend NUMERIC(5, 2) DEFAULT 0,
    expenses_down_trend NUMERIC(5, 2) DEFAULT 0,
    income_up_trend NUMERIC(5, 2) DEFAULT 0,
    income_down_trend NUMERIC(5, 2) DEFAULT 0,
    period DATE NOT NULL,
    summary TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, period)
);

-- Seed some initial categories
INSERT INTO categories (slug, name) VALUES 
('food_beverage', 'Food & Beverage'),
('transportation', 'Transportation'),
('shopping', 'Shopping'),
('utilities', 'Utilities'),
('entertainment', 'Entertainment'),
('health', 'Health & Medical'),
('others', 'Others')
ON CONFLICT (slug) DO NOTHING;
