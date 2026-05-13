CREATE TABLE IF NOT EXISTS agent_configs (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_item_id uuid REFERENCES workspace_items(id) ON DELETE CASCADE,
    name text NOT NULL,
    description text,
    enabled boolean NOT NULL DEFAULT true,
    llm_request_profile_id uuid REFERENCES llm_request_profiles(id) ON DELETE SET NULL,
    system_prompt text,
    selected_prompt_block_ids uuid[] NOT NULL DEFAULT '{}',
    tool_permissions jsonb NOT NULL DEFAULT '{}',
    runtime_config jsonb NOT NULL DEFAULT '{}',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS agent_runs (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_config_id uuid REFERENCES agent_configs(id) ON DELETE CASCADE,
    conversation_id uuid REFERENCES conversations(id) ON DELETE SET NULL,
    status text NOT NULL DEFAULT 'pending',
    input jsonb NOT NULL DEFAULT '{}',
    output jsonb,
    error text,
    started_at timestamptz,
    finished_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
);
