export interface Note {
  id: string
  workspace_item_id: string
  title: string
  document_json: Record<string, unknown>
  plain_text: string
  format: string
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export interface NoteRevision {
  id: string
  note_id: string
  document_json: Record<string, unknown>
  plain_text: string
  reason: string | null
  created_by: string
  agent_run_id: string | null
  created_at: string
}

export interface CreateNoteRequest {
  title: string
  parent_id?: string | null
  sort_order?: number
  document_json?: Record<string, unknown>
  metadata?: Record<string, unknown>
}

export interface UpdateNoteRequest {
  title?: string
  document_json?: Record<string, unknown>
  metadata?: Record<string, unknown>
}

export async function fetchNotes(): Promise<Note[]> {
  const response = await fetch('/api/notes', {
    headers: { Accept: 'application/json' },
  })
  if (!response.ok) throw new Error(`Notes fetch failed: ${response.status}`)
  return response.json()
}

export async function createNote(req: CreateNoteRequest): Promise<Note> {
  const response = await fetch('/api/notes', {
    method: 'POST',
    headers: { Accept: 'application/json', 'Content-Type': 'application/json' },
    body: JSON.stringify(req),
  })
  if (!response.ok) throw new Error(`Note create failed: ${response.status}`)
  return response.json()
}

export async function fetchNote(id: string): Promise<Note> {
  const response = await fetch(`/api/notes/${id}`, {
    headers: { Accept: 'application/json' },
  })
  if (!response.ok) throw new Error(`Note fetch failed: ${response.status}`)
  return response.json()
}

export async function updateNote(id: string, req: UpdateNoteRequest): Promise<Note> {
  const response = await fetch(`/api/notes/${id}`, {
    method: 'PATCH',
    headers: { Accept: 'application/json', 'Content-Type': 'application/json' },
    body: JSON.stringify(req),
  })
  if (!response.ok) throw new Error(`Note update failed: ${response.status}`)
  return response.json()
}

export async function deleteNote(id: string): Promise<void> {
  const response = await fetch(`/api/notes/${id}`, {
    method: 'DELETE',
    headers: { Accept: 'application/json' },
  })
  if (!response.ok) throw new Error(`Note delete failed: ${response.status}`)
}

export async function fetchNoteRevisions(
  noteId: string,
): Promise<NoteRevision[]> {
  const response = await fetch(`/api/notes/${noteId}/revisions`, {
    headers: { Accept: 'application/json' },
  })
  if (!response.ok) throw new Error(`Revisions fetch failed: ${response.status}`)
  const payload = (await response.json()) as { revisions: NoteRevision[] }
  return payload.revisions
}

export async function restoreNoteRevision(
  noteId: string,
  revisionId: string,
): Promise<Note> {
  const response = await fetch(
    `/api/notes/${noteId}/revisions/${revisionId}/restore`,
    { method: 'POST', headers: { Accept: 'application/json' } },
  )
  if (!response.ok) throw new Error(`Revision restore failed: ${response.status}`)
  return response.json()
}
