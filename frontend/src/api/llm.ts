export interface LlmProvider {
  id: string
  name: string
  base_url: string
  api_key: string | null
  default_model: string | null
  enabled: boolean
  created_at: string
  updated_at: string
}

export interface LlmRequestProfile {
  id: string
  name: string
  provider_id: string | null
  endpoint_path: string
  method: string
  base_body: Record<string, unknown>
  headers: Record<string, unknown>
  message_injection_mode: string
  enabled: boolean
  created_at: string
  updated_at: string
}

export interface PromptBlock {
  id: string
  name: string
  description: string | null
  content: string
  block_type: string
  enabled: boolean
  sort_order: number
  created_at: string
  updated_at: string
}

async function api<T>(url: string, options?: RequestInit): Promise<T> {
  const res = await fetch(url, { headers: { Accept: 'application/json', 'Content-Type': 'application/json' }, ...options })
  if (!res.ok) throw new Error(`${url} failed: ${res.status}`)
  return res.json()
}

export const llmApi = {
  providers: {
    list: () => api<LlmProvider[]>('/api/llm/providers'),
    create: (data: Partial<LlmProvider>) => api<LlmProvider>('/api/llm/providers', { method: 'POST', body: JSON.stringify(data) }),
    get: (id: string) => api<LlmProvider>(`/api/llm/providers/${id}`),
    update: (id: string, data: Partial<LlmProvider>) => api<LlmProvider>(`/api/llm/providers/${id}`, { method: 'PATCH', body: JSON.stringify(data) }),
    delete: (id: string) => api<void>(`/api/llm/providers/${id}`, { method: 'DELETE' }),
  },
  profiles: {
    list: () => api<LlmRequestProfile[]>('/api/llm/request-profiles'),
    create: (data: Partial<LlmRequestProfile>) => api<LlmRequestProfile>('/api/llm/request-profiles', { method: 'POST', body: JSON.stringify(data) }),
    get: (id: string) => api<LlmRequestProfile>(`/api/llm/request-profiles/${id}`),
    update: (id: string, data: Partial<LlmRequestProfile>) => api<LlmRequestProfile>(`/api/llm/request-profiles/${id}`, { method: 'PATCH', body: JSON.stringify(data) }),
    delete: (id: string) => api<void>(`/api/llm/request-profiles/${id}`, { method: 'DELETE' }),
    test: (id: string) => api<{ status: number | string; request: unknown; response?: unknown; error?: string }>(`/api/llm/request-profiles/${id}/test`, { method: 'POST' }),
  },
  promptBlocks: {
    list: () => api<PromptBlock[]>('/api/prompt-blocks'),
    create: (data: Partial<PromptBlock>) => api<PromptBlock>('/api/prompt-blocks', { method: 'POST', body: JSON.stringify(data) }),
    update: (id: string, data: Partial<PromptBlock>) => api<PromptBlock>(`/api/prompt-blocks/${id}`, { method: 'PATCH', body: JSON.stringify(data) }),
    delete: (id: string) => api<void>(`/api/prompt-blocks/${id}`, { method: 'DELETE' }),
  },
}
