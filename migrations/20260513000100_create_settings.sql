CREATE TABLE IF NOT EXISTS settings (
    key text PRIMARY KEY,
    value jsonb NOT NULL,
    updated_at timestamptz NOT NULL DEFAULT now()
);
