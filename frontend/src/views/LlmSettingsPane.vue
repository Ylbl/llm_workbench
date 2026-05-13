<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { llmApi, type LlmProvider, type LlmRequestProfile, type PromptBlock } from '../api/llm'

const providers = ref<LlmProvider[]>([])
const profiles = ref<LlmRequestProfile[]>([])
const blocks = ref<PromptBlock[]>([])
const error = ref<string | null>(null)
const testResult = ref<Record<string, unknown> | null>(null)
const activeTab = ref<'providers' | 'profiles' | 'blocks'>('providers')

const editProvider = ref<Partial<LlmProvider> & { id?: string }>({})
const showProviderEditor = ref(false)

const editProfile = ref<Partial<LlmRequestProfile> & { id?: string }>({})
const showProfileEditor = ref(false)
const baseBodyJson = ref('{}')
const headersJson = ref('{}')

const editBlock = ref<Partial<PromptBlock> & { id?: string }>({})
const showBlockEditor = ref(false)

async function load() {
  try {
    const [p, r, b] = await Promise.all([llmApi.providers.list(), llmApi.profiles.list(), llmApi.promptBlocks.list()])
    providers.value = p
    profiles.value = r
    blocks.value = b
  } catch (e) { error.value = String(e) }
}

function openProvider(p?: LlmProvider) {
  editProvider.value = p ? { ...p } : { name: '', base_url: '', api_key: null, default_model: null }
  showProviderEditor.value = true
}

async function saveProvider() {
  try {
    if (editProvider.value.id) {
      await llmApi.providers.update(editProvider.value.id, editProvider.value)
    } else {
      await llmApi.providers.create(editProvider.value)
    }
    showProviderEditor.value = false
    await load()
  } catch (e) { error.value = String(e) }
}

async function deleteProvider(id: string) {
  try { await llmApi.providers.delete(id); await load() } catch (e) { error.value = String(e) }
}

function openProfile(p?: LlmRequestProfile) {
  if (p) {
    editProfile.value = { ...p }
    baseBodyJson.value = JSON.stringify(p.base_body, null, 2)
    headersJson.value = JSON.stringify(p.headers, null, 2)
  } else {
    editProfile.value = { name: '', provider_id: null, endpoint_path: '/chat/completions', method: 'POST', message_injection_mode: 'replace_messages' }
    baseBodyJson.value = '{\n  "model": "gpt-4",\n  "temperature": 0.7\n}'
    headersJson.value = '{}'
  }
  showProfileEditor.value = true
}

async function saveProfile() {
  try {
    let baseBody = {}
    let headers = {}
    try { baseBody = JSON.parse(baseBodyJson.value) } catch { throw new Error('Invalid base_body JSON') }
    try { headers = JSON.parse(headersJson.value) } catch { throw new Error('Invalid headers JSON') }
    const data = { ...editProfile.value, base_body: baseBody, headers }
    if (editProfile.value.id) {
      await llmApi.profiles.update(editProfile.value.id, data)
    } else {
      await llmApi.profiles.create(data)
    }
    showProfileEditor.value = false
    await load()
  } catch (e) { error.value = String(e) }
}

async function deleteProfile(id: string) {
  try { await llmApi.profiles.delete(id); await load() } catch (e) { error.value = String(e) }
}

function openBlock(b?: PromptBlock) {
  editBlock.value = b ? { ...b } : { name: '', content: '', block_type: 'system', enabled: true, sort_order: 0 }
  showBlockEditor.value = true
}
async function saveBlock() {
  try {
    if (editBlock.value.id) {
      await llmApi.promptBlocks.update(editBlock.value.id, editBlock.value)
    } else {
      await llmApi.promptBlocks.create(editBlock.value)
    }
    showBlockEditor.value = false
    await load()
  } catch (e) { error.value = String(e) }
}
async function deleteBlock(id: string) {
  try { await llmApi.promptBlocks.delete(id); await load() } catch (e) { error.value = String(e) }
}

async function testProfile(id: string) {
  try {
    testResult.value = null
    const res = await fetch(`/api/llm/request-profiles/${id}/test`, {
      method: 'POST',
      headers: { Accept: 'application/json' },
    })
    const data = await res.json()
    testResult.value = data as Record<string, unknown>
    if (data.status && data.status !== 'error' && data.response) {
      // show result
    } else if (data.error) {
      error.value = String(data.error)
    }
  } catch (e) { error.value = String(e) }
}

onMounted(() => load())
</script>

<template>
  <div class="llm-settings">
    <h2>LLM Settings</h2>

    <div v-if="error" class="llm-error">{{ error }} <button @click="error=null">&times;</button></div>

    <div class="llm-tabs">
      <button :class="{ active: activeTab==='providers' }" @click="activeTab='providers'">Providers</button>
      <button :class="{ active: activeTab==='profiles' }" @click="activeTab='profiles'">Profiles</button>
      <button :class="{ active: activeTab==='blocks' }" @click="activeTab='blocks'">Prompt Blocks</button>
    </div>

    <div v-if="activeTab==='providers'" class="llm-section">
      <button class="llm-add" @click="openProvider()">+ Add Provider</button>
      <div v-for="p in providers" :key="p.id" class="llm-card">
        <div class="llm-card-header">
          <strong>{{ p.name }}</strong>
          <span :class="p.enabled?'enabled':'disabled'">{{ p.enabled ? 'Active' : 'Disabled' }}</span>
        </div>
        <div class="llm-card-meta">{{ p.base_url }}</div>
        <div class="llm-card-meta" v-if="p.default_model">Default: {{ p.default_model }}</div>
        <div class="llm-card-actions">
          <button @click="openProvider(p)">Edit</button>
          <button class="danger" @click="deleteProvider(p.id)">Delete</button>
        </div>
      </div>

      <div v-if="showProviderEditor" class="llm-editor-overlay" @click.self="showProviderEditor=false">
        <div class="llm-editor">
          <h3>{{ editProvider.id ? 'Edit' : 'New' }} Provider</h3>
          <label>Name <input v-model="editProvider.name" /></label>
          <label>Base URL <input v-model="editProvider.base_url" /></label>
          <label>API Key <input v-model="editProvider.api_key" type="password" /></label>
          <label>Default Model <input v-model="editProvider.default_model" /></label>
          <div class="llm-editor-actions">
            <button @click="saveProvider()">Save</button>
            <button @click="showProviderEditor=false">Cancel</button>
          </div>
        </div>
      </div>
    </div>

    <div v-if="activeTab==='profiles'" class="llm-section">
      <button class="llm-add" @click="openProfile()">+ Add Profile</button>
      <div v-for="p in profiles" :key="p.id" class="llm-card">
        <div class="llm-card-header">
          <strong>{{ p.name }}</strong>
          <span>{{ p.method }} {{ p.endpoint_path }}</span>
        </div>
        <pre class="llm-card-json">{{ JSON.stringify(p.base_body, null, 2) }}</pre>
        <div class="llm-card-actions">
          <button @click="openProfile(p)">Edit</button>
          <button class="test" @click="testProfile(p.id)">Test</button>
          <button class="danger" @click="deleteProfile(p.id)">Delete</button>
        </div>

        <div v-if="testResult && testResult.request === JSON.stringify(p.base_body, null, 2).replace(/\s/g, '')" class="llm-test-result" style="display:none"></div>
      </div>

      <div v-if="showProfileEditor" class="llm-editor-overlay" @click.self="showProfileEditor=false">
        <div class="llm-editor llm-editor-wide">
          <h3>{{ editProfile.id ? 'Edit' : 'New' }} Profile</h3>
          <label>Name <input v-model="editProfile.name" /></label>
          <div class="llm-row">
            <label>Method <input v-model="editProfile.method" style="width:80px" /></label>
            <label>Endpoint <input v-model="editProfile.endpoint_path" style="flex:1" /></label>
          </div>
          <label>Provider
            <select v-model="editProfile.provider_id">
              <option :value="null">None</option>
              <option v-for="p in providers" :key="p.id" :value="p.id">{{ p.name }}</option>
            </select>
          </label>
          <label>Message Injection
            <select v-model="editProfile.message_injection_mode">
              <option value="replace_messages">replace_messages</option>
            </select>
          </label>
          <label>Base Body (JSON)</label>
          <textarea v-model="baseBodyJson" class="llm-json-editor" rows="12" spellcheck="false" />
          <label>Headers (JSON)</label>
          <textarea v-model="headersJson" class="llm-json-editor" rows="4" spellcheck="false" />
          <div class="llm-editor-actions">
            <button @click="saveProfile()">Save</button>
            <button @click="showProfileEditor=false">Cancel</button>
          </div>
        </div>
      </div>
    </div>

    <div v-if="activeTab==='blocks'" class="llm-section">
      <button class="llm-add" @click="openBlock()">+ Add Block</button>
      <div v-for="b in blocks" :key="b.id" class="llm-card">
        <div class="llm-card-header">
          <strong>{{ b.name }}</strong>
          <span>{{ b.block_type }} | sort: {{ b.sort_order }}</span>
        </div>
        <pre class="llm-card-json">{{ b.content.slice(0, 300) }}{{ b.content.length > 300 ? '...' : '' }}</pre>
        <div class="llm-card-actions">
          <button @click="openBlock(b)">Edit</button>
          <button class="danger" @click="deleteBlock(b.id)">Delete</button>
        </div>
      </div>
      <div v-if="showBlockEditor" class="llm-editor-overlay" @click.self="showBlockEditor=false">
        <div class="llm-editor llm-editor-wide">
          <h3>{{ editBlock.id ? 'Edit' : 'New' }} Block</h3>
          <label>Name <input v-model="editBlock.name" /></label>
          <div class="llm-row">
            <label>Type <select v-model="editBlock.block_type"><option>system</option><option>user</option><option>assistant</option></select></label>
            <label>Sort <input v-model.number="editBlock.sort_order" type="number" style="width:60px" /></label>
            <label><input type="checkbox" v-model="editBlock.enabled" /> Enabled</label>
          </div>
          <label>Content</label>
          <textarea v-model="editBlock.content" class="llm-json-editor" rows="8" spellcheck="false" />
          <div class="llm-editor-actions">
            <button @click="saveBlock()">Save</button>
            <button @click="showBlockEditor=false">Cancel</button>
          </div>
        </div>
      </div>
    </div>

    <div v-if="testResult" class="llm-test-result">
      <h3>Test Result</h3>
      <div class="llm-test-split">
        <div>
          <strong>Request</strong>
          <pre>{{ JSON.stringify(testResult.request, null, 2) }}</pre>
        </div>
        <div>
          <strong>Response</strong>
          <pre>{{ JSON.stringify(testResult.response || testResult.error, null, 2) }}</pre>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.llm-settings { padding: 20px; }
.llm-error { padding: 8px 12px; background: var(--danger-soft); color: var(--danger); border-radius: 6px; margin-bottom: 12px; display: flex; justify-content: space-between; }
.llm-tabs { display: flex; gap: 0; margin-bottom: 16px; border-bottom: 1px solid var(--border); }
.llm-tabs button { padding: 8px 20px; border: none; background: none; color: var(--muted); font-size: 14px; cursor: pointer; border-bottom: 2px solid transparent; }
.llm-tabs button.active { color: var(--accent); border-bottom-color: var(--accent); }
.llm-section { display: flex; flex-direction: column; gap: 12px; }
.llm-add { padding: 6px 14px; border: 1px solid var(--accent); background: var(--accent); color: #fff; border-radius: 4px; cursor: pointer; align-self: flex-start; }
.llm-card { padding: 14px; border: 1px solid var(--border); border-radius: 8px; background: var(--surface); }
.llm-card-header { display: flex; justify-content: space-between; margin-bottom: 4px; }
.llm-card-meta { font-size: 13px; color: var(--muted); }
.llm-card-json { margin: 8px 0; padding: 8px; background: var(--surface-subtle); border-radius: 4px; font-size: 12px; max-height: 120px; overflow: auto; }
.llm-card-actions { display: flex; gap: 6px; margin-top: 8px; }
.llm-card-actions button { padding: 4px 12px; border: 1px solid var(--border); border-radius: 4px; background: var(--surface); cursor: pointer; font-size: 13px; }
.llm-card-actions button.test { border-color: var(--accent); color: var(--accent); }
.llm-card-actions button.danger { border-color: var(--danger); color: var(--danger); }
.enabled { color: var(--accent); font-weight: 600; font-size: 12px; }
.disabled { color: var(--muted); font-size: 12px; }
.llm-editor-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.3); display: flex; align-items: center; justify-content: center; z-index: 200; }
.llm-editor { background: var(--surface); border-radius: 12px; padding: 24px; width: 480px; max-height: 90vh; overflow-y: auto; display: flex; flex-direction: column; gap: 12px; }
.llm-editor-wide { width: 680px; }
.llm-editor h3 { margin: 0; }
.llm-editor label { display: flex; flex-direction: column; gap: 4px; font-size: 13px; color: var(--muted); }
.llm-editor input, .llm-editor select { padding: 6px 10px; border: 1px solid var(--border); border-radius: 4px; font-size: 14px; }
.llm-row { display: flex; gap: 8px; }
.llm-json-editor { width: 100%; padding: 10px; border: 1px solid var(--border); border-radius: 6px; background: var(--surface-subtle); font-family: monospace; font-size: 13px; resize: vertical; }
.llm-editor-actions { display: flex; gap: 8px; justify-content: flex-end; }
.llm-editor-actions button { padding: 6px 16px; border-radius: 4px; border: 1px solid var(--border); cursor: pointer; }
.llm-editor-actions button:first-child { background: var(--accent); color: #fff; border-color: var(--accent); }
.llm-test-result { margin-top: 16px; padding: 16px; border: 1px solid var(--border); border-radius: 8px; background: var(--surface); }
.llm-test-split { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-top: 8px; }
.llm-test-split pre { padding: 10px; background: var(--surface-subtle); border-radius: 4px; font-size: 12px; max-height: 400px; overflow: auto; white-space: pre-wrap; }
</style>
