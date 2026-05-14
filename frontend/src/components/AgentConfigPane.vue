<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import type { WorkspaceItem } from '../api/workspace'
import {
  createConversation,
  fetchConversations,
  fetchMessages,
  appendMessage,
  type Message,
} from '../api/chat'
import { llmApi, type LlmRequestProfile, type PromptBlock } from '../api/llm'
import { createAgent, fetchAgents, updateAgent, type CreateAgentRequest, type UpdateAgentRequest } from '../api/agents'
import type { AgentConfig } from '../api/agents'
import MixedContentRenderer from './MixedContentRenderer.vue'

const props = defineProps<{
  workspaceItem: WorkspaceItem
}>()

// --- Chat state ---
const conversationId = ref<string | null>(null)
const messages = ref<Message[]>([])
const composerText = ref('')
const isLoading = ref(false)
const isSending = ref(false)
const error = ref<string | null>(null)
const scrollRef = ref<HTMLElement | null>(null)
const streamingContent = ref('')
const isStreaming = ref(false)
let eventSource: EventSource | null = null

// --- Agent config state ---
const showSettings = ref(false)
const agentId = ref<string | null>(null)
const agentConfig = ref({
  name: '',
  description: '' as string | null,
  enabled: true,
  llm_request_profile_id: null as string | null,
  system_prompt: '' as string | null,
  selected_prompt_block_ids: [] as string[],
  tool_permissions: '{}',
  runtime_config: '{}',
})
const configSaving = ref(false)
const configError = ref<string | null>(null)
const profiles = ref<LlmRequestProfile[]>([])
const blocks = ref<PromptBlock[]>([])

// --- Init ---
async function init() {
  isLoading.value = true
  error.value = null
  try {
    let agentsList: AgentConfig[] = []
    try {
      agentsList = await fetchAgents()
    } catch { /* ignore */ }

    const [p, b, conversations] = await Promise.all([
      llmApi.profiles.list().catch(() => [] as LlmRequestProfile[]),
      llmApi.promptBlocks.list().catch(() => [] as PromptBlock[]),
      fetchConversations(),
    ])
    profiles.value = p
    blocks.value = b

    const existing = agentsList.find(a => a.workspace_item_id === props.workspaceItem.id)
    if (existing) {
      agentId.value = existing.id
      agentConfig.value = {
        name: existing.name,
        description: existing.description,
        enabled: existing.enabled,
        llm_request_profile_id: existing.llm_request_profile_id,
        system_prompt: existing.system_prompt,
        selected_prompt_block_ids: existing.selected_prompt_block_ids || [],
        tool_permissions: JSON.stringify(existing.tool_permissions || {}, null, 2),
        runtime_config: JSON.stringify(existing.runtime_config || {}, null, 2),
      }
    } else {
      agentConfig.value.name = props.workspaceItem.title
    }

    const conv = conversations.find(c => c.workspace_item_id === props.workspaceItem.id)
    if (conv) {
      conversationId.value = conv.id
      await loadMessages(conv.id)
    } else {
      const newConv = await createConversation(props.workspaceItem.title, props.workspaceItem.id)
      conversationId.value = newConv.id
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : '初始化失败'
  } finally {
    isLoading.value = false
  }
}

async function loadMessages(convId: string) {
  try {
    messages.value = await fetchMessages(convId)
    await scrollToBottom()
  } catch (e) {
    error.value = e instanceof Error ? e.message : '加载消息失败'
  }
}

// --- Send ---
async function sendMessage() {
  const text = composerText.value.trim()
  if (!text || !conversationId.value || isSending.value) return

  isSending.value = true
  composerText.value = ''
  error.value = null
  streamingContent.value = ''

  try {
    const userMsg = await appendMessage(conversationId.value, 'user', text)
    messages.value.push(userMsg)
    await scrollToBottom()

    if (agentConfig.value.llm_request_profile_id) {
      sendRealStream()
    } else {
      isStreaming.value = true
      connectSSE(conversationId.value)
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : '发送失败'
    isSending.value = false
  }
}

async function sendRealStream() {
  if (!conversationId.value || !agentConfig.value.llm_request_profile_id) return
  isStreaming.value = true

  try {
    const res = await fetch(`/api/conversations/${conversationId.value}/stream`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        request_profile_id: agentConfig.value.llm_request_profile_id,
        prompt_block_ids: agentConfig.value.selected_prompt_block_ids,
        system_prompt: agentConfig.value.system_prompt,
      }),
    })

    if (!res.ok) {
      const errText = await res.text()
      error.value = `流失败 (${res.status}): ${errText.slice(0, 200)}`
      isStreaming.value = false
      isSending.value = false
      return
    }

    const reader = res.body?.getReader()
    if (!reader) { error.value = '无响应体'; isStreaming.value = false; isSending.value = false; return }

    const decoder = new TextDecoder()
    let buffer = ''

    while (true) {
      const { done, value } = await reader.read()
      if (done) break
      buffer += decoder.decode(value, { stream: true })

      while (buffer.includes('\n\n')) {
        const idx = buffer.indexOf('\n\n')
        const eventBlock = buffer.slice(0, idx)
        buffer = buffer.slice(idx + 2)

        const lines = eventBlock.split('\n')
        let eventType = ''
        let eventData = ''

        for (const line of lines) {
          if (line.startsWith('event: ')) eventType = line.slice(7).trim()
          else if (line.startsWith('data: ')) eventData = line.slice(6)
        }

        if (eventType && eventData) {
          try {
            const data = JSON.parse(eventData)
            if (eventType === 'llm.delta' && data.delta) {
              streamingContent.value += data.delta
              await scrollToBottom()
            } else if (eventType === 'llm.done') {
              isStreaming.value = false
              isSending.value = false
              streamingContent.value = ''
              await loadMessages(conversationId.value!)
              return
            }
          } catch { /* ignore */ }
        }
      }
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : '流错误'
  } finally {
    isStreaming.value = false
    isSending.value = false
  }
}

function connectSSE(convId: string) {
  disconnectSSE()
  eventSource = new EventSource(`/api/events?conversation_id=${convId}`)

  eventSource.addEventListener('llm.delta', (event: MessageEvent) => {
    try {
      const data = JSON.parse(event.data) as { delta: string }
      streamingContent.value += data.delta
      void scrollToBottom()
    } catch { /* ignore */ }
  })

  eventSource.addEventListener('llm.done', async (event: MessageEvent) => {
    disconnectSSE()
    isStreaming.value = false
    try {
      const data = JSON.parse(event.data) as { message_id: string }
      if (data.message_id) await loadMessages(conversationId.value!)
    } catch { /* ignore */ }
    isSending.value = false
    streamingContent.value = ''
  })

  eventSource.onerror = () => {
    disconnectSSE()
    if (isStreaming.value && streamingContent.value) {
      isStreaming.value = false
      loadMessages(conversationId.value!).finally(() => { isSending.value = false; streamingContent.value = '' })
    } else {
      isSending.value = false
    }
  }
}

function disconnectSSE() {
  if (eventSource) { eventSource.close(); eventSource = null }
}

async function scrollToBottom() {
  await nextTick()
  requestAnimationFrame(() => {
    if (scrollRef.value) scrollRef.value.scrollTop = scrollRef.value.scrollHeight
  })
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    void sendMessage()
  }
}

// --- Config save ---
async function saveConfig() {
  configSaving.value = true
  configError.value = null
  try {
    let tp = {}; let rc = {}
    try { tp = JSON.parse(agentConfig.value.tool_permissions) } catch { throw new Error('工具权限 JSON 格式无效') }
    try { rc = JSON.parse(agentConfig.value.runtime_config) } catch { throw new Error('运行时配置 JSON 格式无效') }
    let saved: { id: string }
    if (agentId.value) {
      const updateData: UpdateAgentRequest = {
        name: agentConfig.value.name,
        description: agentConfig.value.description,
        enabled: agentConfig.value.enabled,
        llm_request_profile_id: agentConfig.value.llm_request_profile_id,
        system_prompt: agentConfig.value.system_prompt,
        selected_prompt_block_ids: agentConfig.value.selected_prompt_block_ids,
        tool_permissions: tp,
        runtime_config: rc,
      }
      saved = await updateAgent(agentId.value, updateData)
    } else {
      const createData: CreateAgentRequest = {
        name: agentConfig.value.name,
        description: agentConfig.value.description,
        llm_request_profile_id: agentConfig.value.llm_request_profile_id,
        system_prompt: agentConfig.value.system_prompt,
        selected_prompt_block_ids: agentConfig.value.selected_prompt_block_ids,
        tool_permissions: tp,
        runtime_config: rc,
      }
      saved = await createAgent({ ...createData, workspace_item_id: props.workspaceItem.id })
    }
    agentId.value = saved.id
    showSettings.value = false
  } catch (e) {
    configError.value = e instanceof Error ? e.message : '保存失败'
  } finally {
    configSaving.value = false
  }
}

watch(() => props.workspaceItem.id, () => {
  disconnectSSE()
  void init()
})

onMounted(() => init())
onBeforeUnmount(() => disconnectSSE())
</script>

<template>
  <div class="agent-chat">
    <div v-if="error" class="agent-chat-error">{{ error }} <button @click="error=null">&times;</button></div>

    <!-- Messages -->
    <div ref="scrollRef" class="agent-chat-messages">
      <div v-if="messages.length===0 && !isStreaming && !isLoading" class="agent-chat-empty">
        发送消息开始与代理对话
      </div>

      <div v-for="msg in messages" :key="msg.id" class="chat-message" :class="`chat-message--${msg.role}`">
        <div class="chat-message-role">{{ msg.role === 'user' ? '用户' : msg.role === 'assistant' ? '助手' : msg.role === 'system' ? '系统' : msg.role }}</div>
        <div class="chat-message-content"><MixedContentRenderer :content="msg.content" /></div>
      </div>

      <div v-if="streamingContent" class="chat-message chat-message--assistant">
        <div class="chat-message-role">助手</div>
        <div class="chat-message-content chat-message-streaming">
          <MixedContentRenderer :content="streamingContent" />
          <span class="streaming-cursor">|</span>
        </div>
      </div>
    </div>

    <!-- Config panel (slides up from bottom) -->
    <div v-if="showSettings" class="agent-config-panel">
      <div v-if="configError" class="agent-chat-error">{{ configError }} <button @click="configError=null">&times;</button></div>
      <div class="agent-config-form">
        <label>名称 <input v-model="agentConfig.name" /></label>
        <label>描述 <input v-model="agentConfig.description" /></label>
        <label>请求配置
          <select v-model="agentConfig.llm_request_profile_id">
            <option :value="null">无</option>
            <option v-for="p in profiles" :key="p.id" :value="p.id">{{ p.name }}</option>
          </select>
        </label>
        <label>系统提示词
          <textarea v-model="agentConfig.system_prompt" rows="3" class="agent-config-textarea" />
        </label>
        <label>默认提示块
          <div class="agent-config-blocks">
            <label v-for="b in blocks" :key="b.id" class="agent-block-item">
              <input type="checkbox" :value="b.id" v-model="agentConfig.selected_prompt_block_ids" /> {{ b.name }}
            </label>
          </div>
        </label>
        <label>工具权限 (JSON)
          <textarea v-model="agentConfig.tool_permissions" rows="3" class="agent-config-json" spellcheck="false" />
        </label>
        <label>运行时配置 (JSON)
          <textarea v-model="agentConfig.runtime_config" rows="3" class="agent-config-json" spellcheck="false" />
        </label>
        <div class="agent-config-actions">
          <button class="primary-button" :disabled="configSaving" @click="saveConfig()">{{ configSaving ? '保存中...' : '保存配置' }}</button>
          <button class="secondary-button" @click="showSettings = false">取消</button>
        </div>
      </div>
    </div>

    <!-- Fixed bottom: input + action buttons -->
    <div class="agent-chat-bottom">
      <div class="agent-chat-composer">
        <textarea
          v-model="composerText"
          class="agent-chat-input"
          placeholder="输入消息，Enter 发送"
          rows="1"
          :disabled="isSending"
          @keydown="handleKeydown"
        />
        <button class="agent-chat-send" :disabled="isSending || !composerText.trim()" @click="sendMessage()">发送</button>
      </div>
      <div class="agent-chat-actions">
        <button class="agent-action-btn" :class="{ active: showSettings }" @click="showSettings = !showSettings">设置</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.agent-chat {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

.agent-chat-error {
  padding: 6px 20px;
  background: var(--danger-soft);
  color: var(--danger);
  font-size: 13px;
  flex-shrink: 0;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.agent-chat-messages {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.agent-chat-empty {
  padding: 40px;
  text-align: center;
  color: var(--muted);
  font-size: 14px;
}

.chat-message { max-width: 85%; }
.chat-message--user { align-self: flex-end; }
.chat-message--assistant { align-self: flex-start; }
.chat-message-role {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  color: var(--muted);
  margin-bottom: 4px;
}
.chat-message-content {
  padding: 10px 14px;
  border-radius: 12px;
  font-size: 14px;
  line-height: 1.6;
  word-break: break-word;
}
.chat-message--user .chat-message-content { background: var(--accent); color: #fff; }
.chat-message--assistant .chat-message-content { background: var(--surface-subtle); color: var(--text); }
.streaming-cursor { display: inline; animation: blink 1s step-end infinite; font-weight: 300; }
@keyframes blink { 50% { opacity: 0; } }

/* Config panel */
.agent-config-panel {
  max-height: 50%;
  overflow-y: auto;
  padding: 16px 20px;
  border-top: 1px solid var(--border);
  background: var(--surface);
  flex-shrink: 0;
}

.agent-config-form {
  display: flex;
  flex-direction: column;
  gap: 10px;
  max-width: 640px;
}

.agent-config-form label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 13px;
  color: var(--muted);
}

.agent-config-form input,
.agent-config-form select {
  padding: 6px 10px;
  border: 1px solid var(--border);
  border-radius: 4px;
  font-size: 14px;
}

.agent-config-textarea {
  padding: 8px;
  border: 1px solid var(--border);
  border-radius: 4px;
  font-size: 14px;
  resize: vertical;
}

.agent-config-json {
  padding: 6px 8px;
  border: 1px solid var(--border);
  border-radius: 4px;
  font-family: var(--font-mono);
  font-size: 12px;
  resize: vertical;
}

.agent-config-blocks {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.agent-block-item {
  font-size: 12px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 2px 6px;
  border: 1px solid var(--border);
  border-radius: 4px;
}

.agent-config-actions {
  display: flex;
  gap: 8px;
  margin-top: 4px;
}

.primary-button {
  padding: 6px 16px;
  background: var(--accent);
  color: #fff;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  cursor: pointer;
}

.primary-button:disabled { opacity: 0.6; cursor: not-allowed; }

.secondary-button {
  padding: 6px 16px;
  background: var(--surface);
  color: var(--text);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-size: 14px;
  cursor: pointer;
}

/* Fixed bottom */
.agent-chat-bottom {
  flex-shrink: 0;
  border-top: 1px solid var(--border);
  background: var(--surface);
}

.agent-chat-composer {
  display: flex;
  gap: 6px;
  padding: 6px 12px;
}

.agent-chat-input {
  flex: 1;
  padding: 5px 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  font-size: 13px;
  resize: none;
  outline: none;
  min-height: 28px;
}

.agent-chat-input:focus {
  border-color: var(--accent);
}

.agent-chat-send {
  padding: 4px 14px;
  border: none;
  background: var(--accent);
  color: var(--text);
  border-radius: var(--radius);
  font-size: 13px;
  cursor: pointer;
  white-space: nowrap;
}

.agent-chat-send:disabled { opacity: 0.5; cursor: not-allowed; }

.agent-chat-actions {
  display: flex;
  gap: 4px;
  padding: 0 12px 6px;
}

.agent-action-btn {
  padding: 2px 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-subtle);
  color: var(--muted);
  font-size: 11px;
  cursor: pointer;
}

.agent-action-btn:hover,
.agent-action-btn.active {
  border-color: var(--accent);
  color: var(--accent);
}
</style>
