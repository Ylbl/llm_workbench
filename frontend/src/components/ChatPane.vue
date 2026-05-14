<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import type { WorkspaceItem } from '../api/workspace'
import {
  createConversation,
  fetchConversations,
  fetchMessages,
  appendMessage,
  type Message,
} from '../api/chat'
import { llmApi, type LlmRequestProfile, type PromptBlock } from '../api/llm'
import MixedContentRenderer from './MixedContentRenderer.vue'

const props = defineProps<{
  workspaceItem: WorkspaceItem
}>()

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

const profiles = ref<LlmRequestProfile[]>([])
const selectedProfileId = ref<string>('')
const promptBlocks = ref<PromptBlock[]>([])
const selectedBlockIds = ref<string[]>([])
const agents = ref<Array<{id:string;name:string;llm_request_profile_id:string|null;system_prompt:string|null;selected_prompt_block_ids:string[]}>>([])
const selectedAgentId = ref<string>('')
const systemPromptForRequest = ref<string | null>(null)

watch(selectedAgentId, (id) => {
  const agent = agents.value.find(a => a.id === id)
  if (agent) {
    if (agent.llm_request_profile_id) selectedProfileId.value = agent.llm_request_profile_id
    selectedBlockIds.value = [...agent.selected_prompt_block_ids]
    systemPromptForRequest.value = agent.system_prompt
  } else {
    systemPromptForRequest.value = null
  }
})

const conversationTitle = computed(() => props.workspaceItem.title)

async function initChat() {
  isLoading.value = true
  error.value = null
  try {
    const [conversations, p, blocks, agentList] = await Promise.all([
      fetchConversations(),
      llmApi.profiles.list().catch(() => [] as LlmRequestProfile[]),
      llmApi.promptBlocks.list().catch(() => [] as PromptBlock[]),
      fetch('/api/agents', { headers: { Accept: 'application/json' } }).then(r => r.ok ? r.json() : []).catch(() => []),
    ])
    profiles.value = p
    promptBlocks.value = blocks
    agents.value = agentList as typeof agents.value
    const existing = conversations.find((c) => c.workspace_item_id === props.workspaceItem.id)
    if (existing) {
      conversationId.value = existing.id
      await loadMessages(existing.id)
    } else {
      const conv = await createConversation(props.workspaceItem.title)
      conversationId.value = conv.id
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to init chat'
  } finally {
    isLoading.value = false
  }
}

async function loadMessages(convId: string) {
  try {
    messages.value = await fetchMessages(convId)
    await scrollToBottom()
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to load messages'
  }
}

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

    if (selectedProfileId.value) {
      sendRealStream()
    } else {
      isStreaming.value = true
      connectSSE(conversationId.value)
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Send failed'
    isSending.value = false
  }
}

async function sendRealStream() {
  if (!conversationId.value) return
  isStreaming.value = true

  try {
    const res = await fetch(
      `/api/conversations/${conversationId.value}/stream`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          request_profile_id: selectedProfileId.value,
          prompt_block_ids: selectedBlockIds.value,
          system_prompt: systemPromptForRequest.value,
        }),
      },
    )

    if (!res.ok) {
      const errText = await res.text()
      error.value = `Stream failed (${res.status}): ${errText.slice(0, 200)}`
      isStreaming.value = false
      isSending.value = false
      return
    }

    const reader = res.body?.getReader()
    if (!reader) { error.value = 'No response body'; isStreaming.value = false; isSending.value = false; return }

    const decoder = new TextDecoder()
    let buffer = ''

    while (true) {
      const { done, value } = await reader.read()
      if (done) break
      buffer += decoder.decode(value, { stream: true })

      // Parse SSE: events end with \n\n
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
          } catch { /* parse error */ }
        }
      }
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Stream error'
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
      loadMessages(conversationId.value!).finally(() => {
        isSending.value = false
        streamingContent.value = ''
      })
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

watch(() => props.workspaceItem.id, () => {
  disconnectSSE()
  void initChat()
})

onMounted(() => initChat())
onBeforeUnmount(() => disconnectSSE())
</script>

<template>
  <div class="chat-pane">
    <div class="chat-header">
      <h3 class="chat-title">{{ conversationTitle }}</h3>
      <select v-model="selectedProfileId" class="chat-profile-select">
        <option value="">Mock Stream</option>
        <option v-for="p in profiles" :key="p.id" :value="p.id">{{ p.name }}</option>
      </select>
      <select v-model="selectedAgentId" class="chat-profile-select">
        <option value="">No Agent</option>
        <option v-for="a in agents" :key="a.id" :value="a.id">{{ a.name }}</option>
      </select>
      <div v-if="promptBlocks.length > 0" class="chat-blocks">
        <span class="chat-blocks-label">Blocks:</span>
        <label v-for="b in promptBlocks" :key="b.id" class="chat-block-item">
          <input type="checkbox" :value="b.id" v-model="selectedBlockIds" />
          {{ b.name }}
        </label>
      </div>
      <span v-if="isStreaming" class="chat-status" style="color: var(--accent)">
        {{ selectedProfileId ? 'LLM Streaming...' : 'Streaming...' }}
      </span>
    </div>

    <div v-if="error" class="chat-error">{{ error }} <button @click="error=null">&times;</button></div>

    <div ref="scrollRef" class="chat-messages">
      <div v-if="messages.length===0 && !isStreaming && !isLoading" class="chat-empty">
        Send a message to start.
      </div>

      <div v-for="msg in messages" :key="msg.id" class="chat-message" :class="`chat-message--${msg.role}`">
        <div class="chat-message-role">{{ msg.role }}</div>
        <div class="chat-message-content"><MixedContentRenderer :content="msg.content" /></div>
      </div>

      <div v-if="streamingContent" class="chat-message chat-message--assistant">
        <div class="chat-message-role">assistant</div>
        <div class="chat-message-content chat-message-streaming">
          <MixedContentRenderer :content="streamingContent" />
          <span class="streaming-cursor">|</span>
        </div>
      </div>
    </div>

    <div class="chat-composer">
      <textarea v-model="composerText" class="chat-composer-input" placeholder="Enter to send" rows="2"
        :disabled="isSending" @keydown="handleKeydown" />
      <button class="chat-composer-send" :disabled="isSending || !composerText.trim()" @click="sendMessage()">
        Send
      </button>
    </div>
  </div>
</template>

<style scoped>
.chat-pane { display: flex; flex-direction: column; height: calc(100vh - 280px); min-height: 400px; }
.chat-header { display: flex; align-items: center; gap: 12px; padding: 12px 20px; border-bottom: 1px solid var(--border); background: var(--surface); flex-shrink: 0; }
.chat-title { margin: 0; font-size: 16px; font-weight: 600; }
.chat-profile-select { padding: 4px 8px; border: 1px solid var(--border); border-radius: 4px; background: var(--surface); color: var(--text); font-size: 13px; margin-left: auto; }
.chat-status { font-size: 12px; }
.chat-error { padding: 8px 20px; background: var(--danger-soft); color: var(--danger); font-size: 13px; flex-shrink: 0; display: flex; justify-content: space-between; }
.chat-messages { flex: 1; overflow-y: auto; padding: 16px 20px; display: flex; flex-direction: column; gap: 16px; }
.chat-empty { padding: 40px; text-align: center; color: var(--muted); font-size: 14px; }
.chat-message { max-width: 85%; }
.chat-message--user { align-self: flex-end; }
.chat-message--assistant { align-self: flex-start; }
.chat-message-role { font-size: 11px; font-weight: 600; text-transform: uppercase; color: var(--muted); margin-bottom: 4px; }
.chat-message-content { padding: 10px 14px; border-radius: 12px; font-size: 14px; line-height: 1.6; word-break: break-word; }
.chat-message--user .chat-message-content { background: var(--accent); color: #fff; }
.chat-message--assistant .chat-message-content { background: var(--surface-subtle); color: var(--text); }
.streaming-cursor { display: inline; animation: blink 1s step-end infinite; font-weight: 300; }
@keyframes blink { 50% { opacity: 0; } }
.chat-composer { display: flex; gap: 8px; padding: 12px 20px; border-top: 1px solid var(--border); background: var(--surface); flex-shrink: 0; }
.chat-composer-input { flex: 1; padding: 8px 12px; border: 1px solid var(--border); border-radius: 8px; font-size: 14px; resize: none; outline: none; }
.chat-composer-input:focus { border-color: var(--accent); box-shadow: 0 0 0 2px rgba(15,118,110,0.15); }
.chat-composer-send { padding: 8px 20px; border: none; background: var(--accent); color: #fff; border-radius: 8px; font-size: 14px; font-weight: 600; cursor: pointer; }
.chat-composer-send:disabled { opacity: 0.5; cursor: not-allowed; }
.chat-blocks { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; }
.chat-blocks-label { font-size: 11px; color: var(--muted); font-weight: 600; text-transform: uppercase; }
.chat-block-item { font-size: 12px; cursor: pointer; display: flex; align-items: center; gap: 2px; padding: 2px 6px; border: 1px solid var(--border); border-radius: 4px; }
.chat-block-item:hover { border-color: var(--accent); }
</style>
