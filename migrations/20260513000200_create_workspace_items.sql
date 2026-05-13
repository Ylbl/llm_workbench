CREATE TABLE IF NOT EXISTS workspace_items (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    item_type text NOT NULL CHECK (item_type IN ('note', 'chat', 'agent_config', 'file', 'task', 'settings_view')),
    title text NOT NULL,
    parent_id uuid REFERENCES workspace_items(id),
    sort_order int NOT NULL DEFAULT 0,
    metadata jsonb NOT NULL DEFAULT '{}',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
