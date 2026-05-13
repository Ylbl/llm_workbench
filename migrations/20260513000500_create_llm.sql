CREATE TABLE IF NOT EXISTS llm_providers (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name text NOT NULL,
    base_url text NOT NULL,
    api_key text,
    default_model text,
    enabled boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS llm_request_profiles (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name text NOT NULL,
    provider_id uuid REFERENCES llm_providers(id) ON DELETE SET NULL,
    endpoint_path text NOT NULL DEFAULT '/chat/completions',
    method text NOT NULL DEFAULT 'POST',
    base_body jsonb NOT NULL DEFAULT '{}',
    headers jsonb NOT NULL DEFAULT '{}',
    message_injection_mode text NOT NULL DEFAULT 'replace_messages',
    enabled boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS message_llm_calls (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id uuid REFERENCES conversations(id) ON DELETE SET NULL,
    message_id uuid REFERENCES messages(id) ON DELETE SET NULL,
    agent_run_id uuid,
    provider_id uuid REFERENCES llm_providers(id) ON DELETE SET NULL,
    request_profile_id uuid REFERENCES llm_request_profiles(id) ON DELETE SET NULL,
    request_body jsonb NOT NULL DEFAULT '{}',
    response_body jsonb,
    status text NOT NULL DEFAULT 'pending',
    error text,
    started_at timestamptz NOT NULL DEFAULT now(),
    finished_at timestamptz
);
