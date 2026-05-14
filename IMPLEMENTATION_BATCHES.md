# LLM Learning Workspace Implementation Batches

This document splits the approved engineering blueprint into implementation batches.
Each batch is a human checkpoint. Do not start the next batch until the current
batch has been reviewed by a human and accepted.

The project target is a single-user learning workspace with:

- Rust backend core.
- Vue frontend.
- PostgreSQL database.
- Workspace item model.
- Tiptap/ProseMirror structured notes.
- KaTeX math rendering.
- LLM chat panes opened like workspace documents.
- User-defined agents.
- Manually editable OpenAI-compatible request JSON.
- Prompt blocks.
- File tools, scheduler, and write audit added later.

## Global Rules For All Batches

- Keep Rust as the only trusted backend. The frontend must not access the
  database, local filesystem, or LLM provider directly.
- Keep every persisted shape explicit and migration-backed.
- Prefer small vertical slices that can be manually inspected in the UI.
- Every database-changing batch must include migrations and rollback-safe tests.
- Every API batch must include consistent error responses.
- Every streaming or agent batch must persist enough events for replay/debugging.
- Every frontend batch must expose enough UI to verify the backend behavior.
- Do not add login/auth, vector retrieval, full-text search, or multi-user logic
  unless a later plan explicitly moves those out of deferred scope.
- Avoid hard-coded agent types. All agent behavior must come from user config.
- Do not treat Markdown as the note source of truth. Notes are stored as
  Tiptap/ProseMirror JSON.

## Current UI Architecture Hold

Feature-batch development is paused before Batch 12 while the frontend workbench
UI foundation is stabilized. During this hold, do not add new product behavior
or start Tool Registry work. UI work should be limited to architecture,
consistency, refactoring, and visual/interaction foundations that support the
existing completed batches.

### Workbench UI Rules

- The IDE-style workbench shell is first-party UI, not a third-party visual
  component-library skin.
- Use Reka UI only as a headless interaction layer for non-trivial behavior such
  as dropdown menus, dialogs, popovers, tooltips, tabs, focus management, and
  keyboard navigation.
- Wrap Reka primitives in local `Wb*` components under `frontend/src/ui` before
  using them from feature views.
- Feature views should consume `WbButton`, `WbIconButton`, and future `WbSelect`,
  `WbDialog`, `WbPopover`, `WbTree`, and `WbTabs` instead of importing Reka UI
  or styled third-party components directly.
- Visual consistency is owned by `frontend/src/ui/tokens.css` and
  `frontend/src/ui/components.css`; new colors, spacing, radii, and control
  states should be added as tokens or shared component styles first.
- Avoid mixing visual systems. Do not introduce Element Plus, Naive UI,
  PrimeVue, or similar styled component libraries into feature views unless
  their visuals are completely hidden behind `Wb*` wrappers.
- Keep the workbench dense and IDE-like: activity bars, tool windows, editor
  tabs, right-side panels, thin borders, compact rows, modest radii, no
  marketing-style cards, and no decorative gradients in product surfaces.
- Each UI-foundation pass should include frontend tests and a production build.

### Current UI Foundation Trial

- `reka-ui` is installed as the headless UI dependency.
- `frontend/src/ui/tokens.css` defines the current design tokens.
- `frontend/src/ui/components.css` defines base `Wb*` component styling.
- Initial wrappers exist for:
  - `WbButton`
  - `WbIconButton`
  - `WbDropdownMenu`
  - `WbMenuItem`
- `WorkspaceShell.vue` is the trial integration point for the workbench chrome.

## Batch 0: Repository And Development Baseline

### Goal

Create the empty project skeleton, establish tooling, and make sure the repo can
run a minimal Rust service and Vue frontend without implementing product logic.

### Backend Todos

- Create a Rust workspace or single crate for the backend.
- Add Axum, Tokio, serde, serde_json, sqlx, reqwest, tracing, tower-http, and
  configuration dependencies.
- Implement `main.rs` with:
  - config loading from environment variables,
  - tracing setup,
  - Axum router creation,
  - health endpoint.
- Add a shared error type with JSON error responses:
  - `validation_error`,
  - `not_found`,
  - `database_error`,
  - `internal_error`.
- Add a placeholder application state object.
- Add `/api/health`.

### Frontend Todos

- Create Vue 3 + Vite + TypeScript app.
- Add Pinia and Vue Router.
- Add a minimal shell page that calls `/api/health`.
- Add a basic layout with:
  - left workspace sidebar placeholder,
  - main content area,
  - status/error region.

### Infrastructure Todos

- Add `.env.example` with:
  - `APP_HOST`,
  - `APP_PORT`,
  - `DATABASE_URL`,
  - `APP_DATA_DIR`,
  - `RUST_LOG`.
- Add development scripts:
  - run backend,
  - run frontend,
  - run tests,
  - run database migrations.
- Document local PostgreSQL requirement.

### Tests

- Backend health endpoint test.
- Frontend smoke test that renders the app shell.
- Manual run instructions in README.

### Human Checkpoint

- The repo starts from a clean checkout.
- Backend starts without product data.
- Frontend starts and shows health status.
- The project structure is understandable.
- No authentication, vector search, or note/chat logic has been added yet.

## Batch 1: PostgreSQL Foundation And Settings

### Goal

Connect the Rust backend to PostgreSQL, add migrations, and implement a simple
settings store used by later batches.

### Backend Todos

- Add database connection pool initialization.
- Add migration runner.
- Create `settings` table:

```text
settings
- key text primary key
- value jsonb not null
- updated_at timestamptz not null
```

- Implement settings service:
  - get all settings,
  - get one setting,
  - upsert setting,
  - patch multiple settings.
- Implement routes:
  - `GET /api/settings`,
  - `PATCH /api/settings`.
- Ensure settings values are arbitrary JSON.
- Add database health check to `/api/health`.

### Frontend Todos

- Add `settingsStore`.
- Add settings pane placeholder reachable from the shell.
- Show current backend/database health.
- Allow editing raw settings JSON for early debugging.

### Tests

- Migration test against PostgreSQL.
- Settings upsert/read tests.
- API test for settings patch.
- Frontend test for loading settings.

### Human Checkpoint

- A human can create and edit settings from the UI.
- Settings survive backend restart.
- Database connection failures produce readable errors.
- The schema is still small and easy to inspect.

## Batch 2: Workspace Item Model

### Goal

Make every opened thing in the app a workspace item. Notes, chats, agent configs,
files, tasks, and settings views will later plug into this model.

### Database Todos

- Create `workspace_items` table:

```text
workspace_items
- id uuid primary key
- item_type text not null
- title text not null
- parent_id uuid nullable references workspace_items(id)
- sort_order int not null default 0
- metadata jsonb not null default '{}'
- created_at timestamptz not null
- updated_at timestamptz not null
```

- Add constraints or validation for known initial item types:
  - `note`,
  - `chat`,
  - `agent_config`,
  - `file`,
  - `task`,
  - `settings_view`.

### Backend Todos

- Implement workspace service:
  - list items,
  - create item,
  - get item,
  - update item,
  - delete item,
  - reorder item,
  - move item under parent.
- Implement routes:
  - `GET /api/workspace/items`,
  - `POST /api/workspace/items`,
  - `GET /api/workspace/items/:id`,
  - `PATCH /api/workspace/items/:id`,
  - `DELETE /api/workspace/items/:id`.
- Deleting an item should not yet cascade into future domain tables. For this
  batch, only workspace items exist.

### Frontend Todos

- Add `workspaceStore`.
- Implement `WorkspacePage`.
- Implement `WorkspaceTree`.
- Implement route:
  - `/workspace/:item_id`.
- Render a placeholder pane based on `item_type`.
- Support create/rename/delete workspace item in the UI.

### Tests

- Workspace CRUD tests.
- Parent/child ordering tests.
- Frontend test that opening an item renders the matching placeholder pane.

### Human Checkpoint

- A human can create a workspace tree.
- Clicking an item changes the main pane.
- The app now feels like a workspace, not disconnected pages.
- Empty placeholders are acceptable at this checkpoint.

## Batch 3: Shared Math Layer

### Goal

Create the shared frontend math rendering foundation used by both notes and chat.
This batch does not implement notes or chat persistence yet.

### Frontend Todos

- Add KaTeX.
- Add shared math config module:

```text
mathRenderOptions
latexDelimiters
formula error fallback behavior
copy raw latex behavior
CSS classes for inline and block math
```

- Implement `MathRenderer.vue` for read-only rendering of one formula.
- Implement a utility that can detect complete inline/block formulas in text:
  - inline: `$...$`, `\(...\)`,
  - block: `$$...$$`, `\[...\]`.
- The utility must leave incomplete formulas as raw text.
- Add a small demo pane under a test workspace item or development route.

### Behavior Requirements

- Invalid LaTeX must not crash rendering.
- Raw LaTeX must remain available for copy/edit.
- Incomplete streaming-like text must display as raw text until delimiters close.

### Tests

- Render valid inline formula.
- Render valid block formula.
- Invalid formula falls back safely.
- Incomplete formula remains raw.
- Mixed normal text and formulas render in order.

### Human Checkpoint

- A human can visually inspect formulas.
- Formula style is acceptable for both future notes and chat.
- Error fallback is visible and not disruptive.

## Batch 4: Tiptap Notes Core

### Goal

Implement structured notes as workspace items using Tiptap/ProseMirror JSON as
the source of truth.

### Database Todos

- Create `notes` table:

```text
notes
- id uuid primary key
- workspace_item_id uuid references workspace_items(id)
- title text not null
- document_json jsonb not null
- plain_text text not null default ''
- format text not null default 'tiptap_json'
- metadata jsonb not null default '{}'
- created_at timestamptz not null
- updated_at timestamptz not null
```

- Create `note_revisions` table:

```text
note_revisions
- id uuid primary key
- note_id uuid references notes(id)
- document_json jsonb not null
- plain_text text not null default ''
- reason text nullable
- created_by text not null
- agent_run_id uuid nullable
- created_at timestamptz not null
```

### Backend Todos

- Implement notes service:
  - create note with workspace item,
  - get note by id,
  - get note by workspace item id,
  - update note document,
  - update title,
  - list revisions,
  - restore revision.
- On every note update, create a revision of the previous version.
- Generate/update `plain_text` from document JSON.
- Implement routes:
  - `GET /api/notes`,
  - `POST /api/notes`,
  - `GET /api/notes/:id`,
  - `PATCH /api/notes/:id`,
  - `DELETE /api/notes/:id`,
  - `GET /api/notes/:id/revisions`,
  - `POST /api/notes/:id/revisions/:revision_id/restore`.

### Frontend Todos

- Add Tiptap Vue 3.
- Add `@tiptap/starter-kit`.
- Add `@tiptap/extension-mathematics`.
- Add KaTeX CSS.
- Implement `NoteEditorPane`.
- Implement `TiptapNoteEditor`.
- Load/save `document_json`.
- Show save status.
- Show revision list.
- Add note creation from workspace UI.

### Tests

- Create note creates matching workspace item.
- Save paragraph content as Tiptap JSON.
- Plain text projection updates.
- Revisions are created on update.
- Restore revision works.
- Workspace route opens note pane.

### Human Checkpoint

- A human can create a note from the workspace tree.
- A human can type normal text and save it.
- Refreshing the app preserves note content.
- Revisions exist and can be restored.

## Batch 5: Instant Formula Editing In Notes

### Goal

Make note formulas behave like immediate editable rendered math: after typing a
formula and pressing Enter, it becomes a rendered math node.

### Frontend Todos

- Configure Tiptap Mathematics extension for:
  - `inlineMath`,
  - `blockMath`,
  - KaTeX render options with `throwOnError: false`.
- Add input rules or keyboard handlers:
  - `$E = mc^2$` + Enter -> `inlineMath` node,
  - `$$ ... $$` + Enter -> `blockMath` node.
- Add formula insertion commands:
  - insert inline formula,
  - insert block formula.
- Add `FormulaEditPopover`:
  - opens when user clicks formula,
  - shows raw LaTeX,
  - updates `attrs.latex`,
  - rerenders formula.
- Ensure invalid LaTeX does not break the editor.

### Backend Todos

- Validate that note updates can persist `inlineMath` and `blockMath` nodes.
- Ensure `plain_text` projection includes readable LaTeX text.

### Tests

- Typing inline formula and pressing Enter creates `inlineMath` JSON.
- Typing block formula and pressing Enter creates `blockMath` JSON.
- Clicking formula allows raw LaTeX edit.
- Invalid formula persists and shows fallback.
- Save/reload keeps formula nodes intact.

### Human Checkpoint

- A human can type `$E=mc^2$`, press Enter, and see rendered math immediately.
- A human can edit an existing formula without losing raw LaTeX.
- Formula editing feels stable enough to use for real notes.

## Batch 6: Conversations As Workspace Items

### Goal

Implement LLM conversation windows as workspace items opened like notes, without
calling real LLM providers yet.

### Database Todos

- Create `conversations` table:

```text
conversations
- id uuid primary key
- workspace_item_id uuid references workspace_items(id)
- title text not null
- status text not null default 'active'
- created_at timestamptz not null
- updated_at timestamptz not null
```

- Create `messages` table:

```text
messages
- id uuid primary key
- conversation_id uuid references conversations(id)
- parent_message_id uuid nullable
- role text not null
- content text not null
- content_json jsonb nullable
- created_at timestamptz not null
```

- Create `message_events` table:

```text
message_events
- id uuid primary key
- conversation_id uuid references conversations(id)
- message_id uuid nullable
- agent_run_id uuid nullable
- event_type text not null
- payload jsonb not null
- created_at timestamptz not null
```

### Backend Todos

- Implement conversation service:
  - create chat with workspace item,
  - list conversations,
  - get conversation,
  - list messages,
  - append message,
  - append message event.
- Implement routes:
  - `GET /api/conversations`,
  - `POST /api/conversations`,
  - `GET /api/conversations/:id`,
  - `PATCH /api/conversations/:id`,
  - `DELETE /api/conversations/:id`,
  - `GET /api/conversations/:id/messages`,
  - `POST /api/conversations/:id/messages`,
  - `GET /api/conversations/:id/events`.
- Add mock assistant response mode for local testing.

### Frontend Todos

- Implement `ChatPane`.
- Add message list.
- Add message composer.
- Add create chat from workspace UI.
- Render chat messages with shared math renderer.
- Store raw message content.

### Tests

- Create chat creates workspace item.
- Add user message.
- Add assistant mock message.
- Refresh preserves messages.
- Workspace route opens chat pane.
- Chat renderer renders complete formulas and leaves incomplete formulas raw.

### Human Checkpoint

- A human can open chat windows from the same workspace tree as notes.
- Chat windows feel like document-like objects.
- Math in chat output renders consistently with note math.

## Batch 7: SSE Streaming Infrastructure

### Goal

Add streaming infrastructure for chat and future agents, still using a mock
stream before real provider calls.

### Backend Todos

- Add SSE route:
  - `GET /api/events?conversation_id=...&agent_run_id=...`.
- Implement event broadcaster.
- Persist streamed message events in `message_events`.
- Add mock streaming endpoint behavior:
  - emits `llm.delta`,
  - emits `llm.done`,
  - stores final assistant message.

### Frontend Todos

- Add `streamStore`.
- Connect `ChatPane` to SSE.
- Append delta tokens to active message.
- Re-render math safely while streaming.
- On stream completion, run full message render pass.
- Show stream status and errors.

### Tests

- SSE event order test.
- Delta persistence test.
- Frontend token append test.
- Incomplete formula during stream remains raw.
- Completed formula after stream renders.

### Human Checkpoint

- A human can send a mock message and watch it stream.
- Streaming does not corrupt formulas.
- Stored events can be inspected after refresh.

## Batch 8: LLM Providers And Manual Request Profiles

### Goal

Implement OpenAI-compatible provider configuration and manually editable request
profiles. This is a core differentiator from UI-only parameter sliders.

### Database Todos

- Create `llm_providers`:

```text
llm_providers
- id uuid primary key
- name text not null
- base_url text not null
- api_key text nullable
- default_model text nullable
- enabled boolean not null default true
- created_at timestamptz not null
- updated_at timestamptz not null
```

- Create `llm_request_profiles`:

```text
llm_request_profiles
- id uuid primary key
- name text not null
- provider_id uuid references llm_providers(id)
- endpoint_path text not null default '/chat/completions'
- method text not null default 'POST'
- base_body jsonb not null
- headers jsonb not null default '{}'
- message_injection_mode text not null default 'replace_messages'
- enabled boolean not null default true
- created_at timestamptz not null
- updated_at timestamptz not null
```

- Create `message_llm_calls`:

```text
message_llm_calls
- id uuid primary key
- conversation_id uuid references conversations(id)
- message_id uuid nullable references messages(id)
- agent_run_id uuid nullable
- provider_id uuid nullable
- request_profile_id uuid nullable
- request_body jsonb not null
- response_body jsonb nullable
- status text not null
- error text nullable
- started_at timestamptz not null
- finished_at timestamptz nullable
```

### Backend Todos

- Implement provider CRUD.
- Implement request profile CRUD.
- Implement request builder with merge order:
  1. profile `base_body`,
  2. agent runtime LLM overrides,
  3. current request `runtime_overrides`,
  4. current request `raw_body_overrides`,
  5. backend injected messages.
- Implement `replace_messages` injection mode first.
- Preserve unknown/custom JSON fields.
- Archive final request JSON before provider call.
- Archive response body or error.
- Add profile test endpoint:
  - `POST /api/llm/request-profiles/:id/test`.

### Frontend Todos

- Add LLM settings pane.
- Add provider editor.
- Add request profile editor.
- Add JSON body editor for `base_body`.
- Add JSON editor for `headers`.
- Add test call UI.
- Add final request JSON preview.

### Tests

- Provider CRUD tests.
- Request profile CRUD tests.
- JSON merge order tests.
- Unknown provider params preserved.
- Missing `messages` gets injected.
- Existing `messages` replaced in `replace_messages` mode.
- Final request archived.

### Human Checkpoint

- A human can handwrite provider request JSON.
- The app shows exactly what JSON would be sent.
- Unknown/custom fields are not stripped.
- Failed provider calls are understandable.

## Batch 9: Real LLM Chat Integration

### Goal

Connect chat panes to real OpenAI-compatible streaming calls through the request
profile system.

### Backend Todos

- Implement `llm_gateway`.
- Implement streaming provider request using reqwest.
- Support OpenAI-compatible streaming deltas.
- Convert provider stream into internal events:
  - `llm.request.created`,
  - `llm.delta`,
  - `llm.done`,
  - `llm.error`.
- Store final assistant message.
- Store final response body or normalized stream summary.
- Implement timeout and provider error mapping:
  - `provider_error`,
  - `provider_timeout`,
  - `provider_bad_response`,
  - `llm_request_build_failed`.

### Frontend Todos

- Let `ChatPane` select LLM request profile.
- Let message send include:
  - `llm_request_profile_id`,
  - `runtime_overrides`,
  - `raw_body_overrides`.
- Add `LlmRequestInspector`.
- Show final request JSON for each assistant response.

### Tests

- Mock provider streaming integration.
- Real provider call can be manually tested.
- Final request JSON matches selected profile and overrides.
- Provider errors persist to `message_llm_calls`.
- Frontend shows stream and final archived message.

### Human Checkpoint

- A human can configure a provider and chat with a real model.
- A human can inspect the exact JSON sent to the provider.
- Chat math rendering still works with real model output.

## Batch 10: Prompt Blocks

### Goal

Add independent reusable text blocks that can be injected into LLM requests
without forcing the user to put everything in one preset.

### Database Todos

- Create `prompt_blocks`:

```text
prompt_blocks
- id uuid primary key
- name text not null
- description text nullable
- content text not null
- block_type text not null
- enabled boolean not null default true
- sort_order int not null default 0
- created_at timestamptz not null
- updated_at timestamptz not null
```

### Backend Todos

- Implement prompt block CRUD.
- Implement prompt injection service.
- Support selected prompt block ids in chat send request.
- Default injection order:
  1. selected prompt blocks by `sort_order`,
  2. conversation context,
  3. current user message.
- Add archived metadata showing which prompt blocks were used.

### Frontend Todos

- Add `PromptBlocksPane` or workspace item pane for prompt block management.
- Add prompt block selector in `ChatPane`.
- Support multi-select.
- Support preview.
- Support temporary per-message selection.

### Tests

- Prompt block CRUD.
- Multi-block injection order.
- Disabled blocks are not injected.
- Selected block ids are archived.
- Final request JSON includes expected injected text.

### Human Checkpoint

- A human can create separate reusable text blocks.
- A human can combine several blocks for one message.
- The final request JSON makes the injection transparent.

## Batch 11: User-defined Agent Configs

### Goal

Implement configurable agents without hard-coded agent types.

### Database Todos

- Create `agent_configs`:

```text
agent_configs
- id uuid primary key
- workspace_item_id uuid nullable references workspace_items(id)
- name text not null
- description text nullable
- enabled boolean not null default true
- llm_request_profile_id uuid references llm_request_profiles(id)
- system_prompt text nullable
- selected_prompt_block_ids uuid[] not null default '{}'
- tool_permissions jsonb not null default '{}'
- runtime_config jsonb not null default '{}'
- created_at timestamptz not null
- updated_at timestamptz not null
```

- Create `agent_runs`:

```text
agent_runs
- id uuid primary key
- agent_config_id uuid references agent_configs(id)
- conversation_id uuid nullable references conversations(id)
- status text not null
- input jsonb not null
- output jsonb nullable
- error text nullable
- started_at timestamptz nullable
- finished_at timestamptz nullable
- created_at timestamptz not null
```

### Backend Todos

- Implement agent config CRUD.
- Create agent config workspace items.
- Implement agent run service:
  - create run,
  - execute one LLM request using agent config,
  - persist run status,
  - persist events.
- Agent config determines:
  - request profile,
  - system prompt,
  - selected prompt blocks,
  - runtime config,
  - allowed tools.
- Do not branch logic based on agent name/type.

### Frontend Todos

- Add `AgentConfigPane`.
- Edit:
  - name,
  - description,
  - request profile,
  - system prompt,
  - default prompt blocks,
  - runtime config JSON,
  - tool permissions JSON.
- Add agent selector in `ChatPane`.
- Add agent run panel.

### Tests

- Agent config CRUD.
- Agent workspace item opens config pane.
- Agent run persists status transitions.
- Agent uses configured request profile.
- Agent uses configured prompt blocks.
- No hard-coded agent type behavior exists.

### Human Checkpoint

- A human can create a named agent.
- The agent's behavior is visibly controlled by its configuration.
- Running an agent creates inspectable events and run records.

## Batch 12: Tool Registry And Conversation/Note Tools

### Goal

Add a permission-checked tool system and implement safe tools for reading
conversations and modifying notes.

### Backend Todos

- Implement tool registry abstraction:

```text
tool.name
tool.description
tool.input_schema
tool.output_schema
tool.permission_key
tool.execute()
```

- Implement tools:
  - `conversation.read_messages`,
  - `conversation.summarize_context`,
  - `note.list`,
  - `note.read`,
  - `note.create`,
  - `note.update_document`,
  - `note.create_revision`.
- Check `agent_configs.tool_permissions` before every tool call.
- Persist tool call events:
  - `agent.tool_call`,
  - `agent.tool_result`,
  - `agent.tool_error`.
- For note updates by agent, create revisions with `created_by = agent`.

### Frontend Todos

- Add tool permission editor.
- Add agent run event timeline showing tool calls.
- Add visual indication when an agent modifies a note.

### Tests

- Tool permission allowed path.
- Tool permission denied path.
- Agent note update creates revision.
- Tool call events persist.
- Tool error does not crash entire app.

### Human Checkpoint

- A human can see what tools an agent is allowed to use.
- A human can inspect tool calls after an agent run.
- Agent-created note changes are reversible through revisions.

## Batch 13: Files And Attachments

### Goal

Add file sources, document metadata, conversation attachments, and file reading
tools. This batch does not implement vector search.

### Database Todos

- Create `file_sources`:

```text
file_sources
- id uuid primary key
- workspace_item_id uuid nullable references workspace_items(id)
- name text not null
- source_type text not null
- original_path text nullable
- storage_policy text not null
- enabled boolean not null default true
- metadata jsonb not null default '{}'
- created_at timestamptz not null
- updated_at timestamptz not null
```

- Create `documents`:

```text
documents
- id uuid primary key
- source_id uuid references file_sources(id)
- title text not null
- mime_type text nullable
- original_path text nullable
- stored_path text nullable
- hash text nullable
- parse_status text not null default 'pending'
- parse_error text nullable
- extracted_text text nullable
- metadata jsonb not null default '{}'
- created_at timestamptz not null
- updated_at timestamptz not null
```

- Create `attachments`:

```text
attachments
- id uuid primary key
- conversation_id uuid nullable references conversations(id)
- message_id uuid nullable references messages(id)
- filename text not null
- mime_type text nullable
- original_path text nullable
- stored_path text nullable
- hash text nullable
- metadata jsonb not null default '{}'
- created_at timestamptz not null
```

### Backend Todos

- Add file source CRUD.
- Support `reference` and `copy` storage policy.
- Parse text-like files into `documents.extracted_text`.
- Save unsupported files with `parse_status = unsupported`.
- Implement tools:
  - `file.list`,
  - `file.read`.
- Support attachments in chat message send request.

### Frontend Todos

- Add file pane.
- Add file source create UI.
- Show parse status.
- Show extracted text preview.
- Attach files to chat messages.

### Tests

- Reference file source.
- Copy file source.
- Text file parse.
- Unsupported file status.
- Attachment saved with message.
- Agent file read respects permissions.

### Human Checkpoint

- A human can add a local file or folder source.
- A human can inspect extracted text.
- A chat can carry attachments.
- Agents can read files only when permitted.

## Batch 14: Scheduler

### Goal

Add Rust-process-bound scheduled tasks. Tasks run only while the backend process
is alive; the frontend only displays and triggers them.

### Database Todos

- Create `scheduled_tasks`:

```text
scheduled_tasks
- id uuid primary key
- workspace_item_id uuid nullable references workspace_items(id)
- name text not null
- enabled boolean not null default true
- task_type text not null
- agent_config_id uuid nullable references agent_configs(id)
- schedule_config jsonb not null
- payload jsonb not null default '{}'
- last_run_at timestamptz nullable
- next_run_at timestamptz nullable
- created_at timestamptz not null
- updated_at timestamptz not null
```

- Create `task_runs`:

```text
task_runs
- id uuid primary key
- task_id uuid references scheduled_tasks(id)
- agent_run_id uuid nullable references agent_runs(id)
- status text not null
- output jsonb nullable
- error text nullable
- started_at timestamptz not null
- finished_at timestamptz nullable
```

### Backend Todos

- Implement scheduler service.
- Load enabled tasks on startup.
- Support interval schedule:

```json
{ "kind": "interval", "seconds": 86400 }
```

- Optionally support cron if dependency is added.
- Implement task CRUD.
- Implement manual task run.
- Task type `agent_run` invokes configured agent.
- Persist task run status and errors.

### Frontend Todos

- Add `TaskPane`.
- Show task list.
- Create/edit task with JSON schedule config.
- Bind task to agent config.
- Manual run button.
- Show latest task runs.

### Tests

- Task CRUD.
- Manual task run.
- Interval next run calculation.
- Failed agent task records error.
- Scheduler does not run disabled tasks.

### Human Checkpoint

- A human can create a scheduled task.
- A human can manually run it.
- A human can close the frontend while backend continues to run tasks.
- Stopping backend stops scheduling.

## Batch 15: File Write Plan, Audit, And Rollback

### Goal

Allow agents to propose and apply file writes through an auditable, reversible
pipeline.

### Database Todos

- Create `file_write_audits`:

```text
file_write_audits
- id uuid primary key
- agent_run_id uuid nullable references agent_runs(id)
- path text not null
- operation text not null
- before_hash text nullable
- after_hash text nullable
- before_snapshot_path text nullable
- after_snapshot_path text nullable
- diff text nullable
- created_at timestamptz not null
```

### Backend Todos

- Implement write plan API:
  - proposed operations,
  - target paths,
  - before/after content,
  - unified diff.
- Implement apply write:
  - read existing file,
  - calculate before hash,
  - save before snapshot,
  - write new content,
  - calculate after hash,
  - save audit.
- Implement rollback:
  - restore from snapshot,
  - delete created file when rolling back create,
  - restore deleted file when rolling back delete.
- Implement tools:
  - `file.preview_write`,
  - `file.apply_write`,
  - `file.rollback_write`.

### Frontend Todos

- Add file write plan preview.
- Show diff before apply.
- Show audit list.
- Add rollback button.
- Show agent run that caused a write.

### Tests

- Preview write creates no file changes.
- Apply update writes file and audit.
- Apply create writes file and audit.
- Rollback update restores content.
- Rollback create removes file.
- Permission denied prevents tool execution.

### Human Checkpoint

- A human can inspect a proposed file write before applying it.
- Applied writes are auditable.
- Rollback works on real files.
- This batch should receive extra review before broader agent write access.

## Batch 16: Polishing, Consistency, And End-to-End Acceptance

### Goal

Make the whole product coherent enough for daily experimental use.

### Backend Todos

- Normalize error codes across all routes.
- Add request logging with sensitive fields redacted.
- Add consistent timestamps.
- Add pagination where needed:
  - workspace items,
  - messages,
  - events,
  - runs,
  - audits.
- Add database indexes for:
  - workspace parent/sort,
  - conversation messages,
  - message events,
  - agent runs,
  - task runs.
- Add backup/export endpoint for major user data if practical.

### Frontend Todos

- Improve workspace navigation.
- Add loading and error states everywhere.
- Add empty states.
- Add dirty-state warnings for unsaved notes/config JSON.
- Add final request JSON inspector polish.
- Add event timelines for chat/agent/task.
- Make math styling consistent across notes and chat.

### Tests

- Full end-to-end happy path:
  1. create note,
  2. type formula,
  3. create chat,
  4. call LLM,
  5. inspect request JSON,
  6. create prompt blocks,
  7. create agent,
  8. agent reads conversation,
  9. agent modifies note,
  10. restore revision.
- Error path tests:
  - bad provider key,
  - invalid JSON profile,
  - provider timeout,
  - invalid formula,
  - denied tool permission,
  - failed file write.

### Human Checkpoint

- The app can be used for a real study session.
- The workspace model feels consistent.
- Notes and chat both render formulas acceptably.
- LLM request transparency works.
- Agent actions are inspectable.
- The project is ready for a second planning pass.

## Deferred Scope

These are intentionally not part of the batches above.

### Access Control

- Admin password.
- API tokens.
- Local no-auth / remote auth split.
- Multi-user accounts.
- Permission isolation.

### Vector Retrieval

- pgvector.
- Embedding provider.
- Document chunks.
- Hybrid search.
- RAG context builder.
- Folder indexing tasks.

### Full-text Search

- PostgreSQL full-text search.
- Chinese tokenization.
- File name/path/body ranking.
- Conversation search.

### Remote Deployment Security

- Reverse proxy auth.
- VPN.
- IP allowlist.
- Cloudflare Access.
- Built-in auth.

### Advanced Automated File Write Policy

- Whether every write requires confirmation.
- Which paths are forbidden.
- Whether delete is allowed.
- Whether batch modification is allowed.
- Whether agent may auto-apply writes.
