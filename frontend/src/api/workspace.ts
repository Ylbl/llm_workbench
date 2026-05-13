export interface WorkspaceItem {
  id: string
  item_type: string
  title: string
  parent_id: string | null
  sort_order: number
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export interface CreateWorkspaceItemRequest {
  item_type: string
  title: string
  parent_id?: string | null
  sort_order?: number
  metadata?: Record<string, unknown>
}

export interface UpdateWorkspaceItemRequest {
  title?: string
  parent_id?: string | null
  sort_order?: number
  metadata?: Record<string, unknown>
}

export async function fetchWorkspaceItems(): Promise<WorkspaceItem[]> {
  const response = await fetch('/api/workspace/items', {
    headers: { Accept: 'application/json' },
  })

  if (!response.ok) {
    throw new Error(`Workspace items fetch failed with HTTP ${response.status}`)
  }

  return response.json() as Promise<WorkspaceItem[]>
}

export async function createWorkspaceItem(
  req: CreateWorkspaceItemRequest,
): Promise<WorkspaceItem> {
  const response = await fetch('/api/workspace/items', {
    method: 'POST',
    headers: { Accept: 'application/json', 'Content-Type': 'application/json' },
    body: JSON.stringify(req),
  })

  if (!response.ok) {
    throw new Error(`Workspace item create failed with HTTP ${response.status}`)
  }

  return response.json() as Promise<WorkspaceItem>
}

export async function fetchWorkspaceItem(id: string): Promise<WorkspaceItem> {
  const response = await fetch(`/api/workspace/items/${id}`, {
    headers: { Accept: 'application/json' },
  })

  if (!response.ok) {
    throw new Error(`Workspace item fetch failed with HTTP ${response.status}`)
  }

  return response.json() as Promise<WorkspaceItem>
}

export async function updateWorkspaceItem(
  id: string,
  req: UpdateWorkspaceItemRequest,
): Promise<WorkspaceItem> {
  const response = await fetch(`/api/workspace/items/${id}`, {
    method: 'PATCH',
    headers: { Accept: 'application/json', 'Content-Type': 'application/json' },
    body: JSON.stringify(req),
  })

  if (!response.ok) {
    throw new Error(`Workspace item update failed with HTTP ${response.status}`)
  }

  return response.json() as Promise<WorkspaceItem>
}

export async function deleteWorkspaceItem(id: string): Promise<void> {
  const response = await fetch(`/api/workspace/items/${id}`, {
    method: 'DELETE',
    headers: { Accept: 'application/json' },
  })

  if (!response.ok) {
    throw new Error(`Workspace item delete failed with HTTP ${response.status}`)
  }
}
