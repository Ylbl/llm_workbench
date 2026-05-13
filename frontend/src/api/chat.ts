export interface Conversation {
  id: string
  workspace_item_id: string
  title: string
  status: string
  created_at: string
  updated_at: string
}

export interface Message {
  id: string
  conversation_id: string
  parent_message_id: string | null
  role: 'user' | 'assistant' | 'system' | 'tool'
  content: string
  content_json: Record<string, unknown> | null
  created_at: string
}

export interface MessageEvent {
  id: string
  conversation_id: string
  message_id: string | null
  agent_run_id: string | null
  event_type: string
  payload: Record<string, unknown>
  created_at: string
}

export async function fetchConversations(): Promise<Conversation[]> {
  const res = await fetch('/api/conversations', { headers: { Accept: 'application/json' } })
  if (!res.ok) throw new Error(`Conversations fetch failed: ${res.status}`)
  return res.json()
}

export async function createConversation(title: string): Promise<Conversation> {
  const res = await fetch('/api/conversations', {
    method: 'POST',
    headers: { Accept: 'application/json', 'Content-Type': 'application/json' },
    body: JSON.stringify({ title }),
  })
  if (!res.ok) throw new Error(`Conversation create failed: ${res.status}`)
  return res.json()
}

export async function fetchConversation(id: string): Promise<Conversation> {
  const res = await fetch(`/api/conversations/${id}`, { headers: { Accept: 'application/json' } })
  if (!res.ok) throw new Error(`Conversation fetch failed: ${res.status}`)
  return res.json()
}

export async function deleteConversation(id: string): Promise<void> {
  const res = await fetch(`/api/conversations/${id}`, { method: 'DELETE' })
  if (!res.ok) throw new Error(`Conversation delete failed: ${res.status}`)
}

export async function fetchMessages(conversationId: string): Promise<Message[]> {
  const res = await fetch(`/api/conversations/${conversationId}/messages`, {
    headers: { Accept: 'application/json' },
  })
  if (!res.ok) throw new Error(`Messages fetch failed: ${res.status}`)
  const payload = (await res.json()) as { messages: Message[] }
  return payload.messages
}

export async function appendMessage(
  conversationId: string,
  role: string,
  content: string,
): Promise<Message> {
  const res = await fetch(`/api/conversations/${conversationId}/messages`, {
    method: 'POST',
    headers: { Accept: 'application/json', 'Content-Type': 'application/json' },
    body: JSON.stringify({ role, content }),
  })
  if (!res.ok) throw new Error(`Message append failed: ${res.status}`)
  return res.json()
}

export async function fetchEvents(conversationId: string): Promise<MessageEvent[]> {
  const res = await fetch(`/api/conversations/${conversationId}/events`, {
    headers: { Accept: 'application/json' },
  })
  if (!res.ok) throw new Error(`Events fetch failed: ${res.status}`)
  const payload = (await res.json()) as { events: MessageEvent[] }
  return payload.events
}
