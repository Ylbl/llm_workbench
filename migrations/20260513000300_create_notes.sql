CREATE TABLE IF NOT EXISTS notes (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_item_id uuid NOT NULL REFERENCES workspace_items(id) ON DELETE CASCADE,
    title text NOT NULL,
    document_json jsonb NOT NULL DEFAULT '{"type":"doc","content":[]}',
    plain_text text NOT NULL DEFAULT '',
    format text NOT NULL DEFAULT 'tiptap_json',
    metadata jsonb NOT NULL DEFAULT '{}',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS note_revisions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    note_id uuid NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    document_json jsonb NOT NULL,
    plain_text text NOT NULL DEFAULT '',
    reason text,
    created_by text NOT NULL DEFAULT 'user',
    agent_run_id uuid,
    created_at timestamptz NOT NULL DEFAULT now()
);
