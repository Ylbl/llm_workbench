<script setup lang="ts">
import { onMounted, ref } from 'vue'
import type { WorkspaceItem } from '../api/workspace'
import { llmApi, type LlmRequestProfile, type PromptBlock } from '../api/llm'

const props = defineProps<{ workspaceItem: WorkspaceItem }>()

const agentId = ref<string | null>(null)
const agent = ref({
  name: '',
  description: '' as string | null,
  enabled: true,
  llm_request_profile_id: null as string | null,
  system_prompt: '' as string | null,
  selected_prompt_block_ids: [] as string[],
  tool_permissions: '{}',
  runtime_config: '{}',
})
const isSaving = ref(false)
const error = ref<string | null>(null)
const profiles = ref<LlmRequestProfile[]>([])
const blocks = ref<PromptBlock[]>([])
const runResult = ref<string | null>(null)

async function load() {
  try {
    let agentsList: Array<{ id: string; workspace_item_id: string } & typeof agent.value> = []
    try {
      const res = await fetch('/api/agents', { headers: { Accept: 'application/json' } })
      agentsList = await res.json()
    } catch { /* keep empty */ }
    
    const [p, b] = await Promise.all([
      llmApi.profiles.list().catch(() => []),
      llmApi.promptBlocks.list().catch(() => []),
    ])
    profiles.value = p; blocks.value = b
    const existing = agentsList.find(a => a.workspace_item_id === props.workspaceItem.id)
    if (existing) {
      agentId.value = existing.id
      agent.value = { ...existing, tool_permissions: JSON.stringify(existing.tool_permissions || {}, null, 2), runtime_config: JSON.stringify(existing.runtime_config || {}, null, 2) }
    }
  } catch (e) { error.value = String(e) }
}

async function save() {
  isSaving.value = true; error.value = null
  try {
    let tp = {}; let rc = {}
    try { tp = JSON.parse(agent.value.tool_permissions) } catch { throw new Error('Invalid tool_permissions JSON') }
    try { rc = JSON.parse(agent.value.runtime_config) } catch { throw new Error('Invalid runtime_config JSON') }
    const data = { ...agent.value, tool_permissions: tp, runtime_config: rc }
    let res: Response
    if (agentId.value) {
      res = await fetch(`/api/agents/${agentId.value}`, { method: 'PATCH', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(data) })
    } else {
      res = await fetch('/api/agents', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(data) })
    }
    if (!res.ok) {
      const errBody = await res.text()
      throw new Error(`Save failed (${res.status}): ${errBody.slice(0, 200)}`)
    }
    const saved = await res.json() as { id: string }
    agentId.value = saved.id
    error.value = null
  } catch (e) { error.value = String(e) } finally { isSaving.value = false }
}

const isRunning = ref(false)

async function run() {
  runResult.value = null; error.value = null
  if (!agentId.value) await save()
  if (!agentId.value) {
    error.value = 'Save agent first'
    return
  }
  isRunning.value = true
  try {
    const res = await fetch(`/api/agents/${agentId.value}/run`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ input: { message: 'Run agent' } }),
    })
    if (!res.ok) {
      const errBody = await res.text()
      throw new Error(`Run failed (${res.status}): ${errBody.slice(0, 200)}`)
    }
    const r = await res.json() as { status: string; output?: { content?: string }; error?: string }
    runResult.value = r.output?.content || r.error || r.status
  } catch (e) { error.value = String(e) } finally { isRunning.value = false }
}

onMounted(() => load())
</script>

<template>
  <div class="agent-pane">
    <div class="agent-header">
      <h3>Agent Config</h3>
      <span class="muted-label">{{ workspaceItem.title }}</span>
      <div style="margin-left:auto;display:flex;gap:8px">
        <button class="primary-button" :disabled="isSaving" @click="save()">Save</button>
        <button class="secondary-button" @click="run()" :disabled="isRunning">{{ isRunning ? 'Running...' : 'Run' }}</button>
      </div>
    </div>
    <div v-if="error" class="chat-error">{{ error }}</div>
    <div v-if="runResult" class="agent-result">
      <strong>Result:</strong>
      <pre>{{ runResult }}</pre>
    </div>
    <div class="agent-form">
      <label>Name <input v-model="agent.name" /></label>
      <label>Description <input v-model="agent.description" /></label>
      <label>Request Profile
        <select v-model="agent.llm_request_profile_id">
          <option :value="null">None</option>
          <option v-for="p in profiles" :key="p.id" :value="p.id">{{ p.name }}</option>
        </select>
      </label>
      <label>System Prompt
        <textarea v-model="agent.system_prompt" rows="4" class="agent-textarea" />
      </label>
      <label>Default Prompt Blocks
        <div class="agent-blocks">
          <label v-for="b in blocks" :key="b.id" class="chat-block-item">
            <input type="checkbox" :value="b.id" v-model="agent.selected_prompt_block_ids" /> {{ b.name }}
          </label>
        </div>
      </label>
      <label>Tool Permissions (JSON)
        <textarea v-model="agent.tool_permissions" rows="4" class="agent-json" spellcheck="false" />
      </label>
      <label>Runtime Config (JSON)
        <textarea v-model="agent.runtime_config" rows="4" class="agent-json" spellcheck="false" />
      </label>
    </div>
  </div>
</template>

<style scoped>
.agent-pane { display: flex; flex-direction: column; gap: 12px; }
.agent-header { display: flex; align-items: center; gap: 12px; padding-bottom: 12px; border-bottom: 1px solid var(--border); }
.agent-header h3 { margin: 0; }
.agent-form { display: flex; flex-direction: column; gap: 12px; max-width: 640px; }
.agent-form label { display: flex; flex-direction: column; gap: 4px; font-size: 13px; color: var(--muted); }
.agent-form input, .agent-form select { padding: 6px 10px; border: 1px solid var(--border); border-radius: 4px; font-size: 14px; }
.agent-textarea { padding: 8px; border: 1px solid var(--border); border-radius: 4px; font-size: 14px; resize: vertical; }
.agent-json { padding: 8px; border: 1px solid var(--border); border-radius: 4px; font-family: monospace; font-size: 13px; resize: vertical; }
.agent-blocks { display: flex; flex-wrap: wrap; gap: 6px; }
.agent-result { padding: 12px; background: var(--surface-subtle); border-radius: 6px; }
.agent-result pre { margin: 8px 0 0; white-space: pre-wrap; font-size: 13px; }
.chat-error { padding: 8px 12px; background: var(--danger-soft); color: var(--danger); border-radius: 6px; font-size: 13px; }
.chat-block-item { font-size: 12px; cursor: pointer; display: flex; align-items: center; gap: 2px; padding: 2px 6px; border: 1px solid var(--border); border-radius: 4px; }
</style>
