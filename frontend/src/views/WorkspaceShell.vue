<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useHealthStore } from '../stores/health'
import { useWorkspaceStore } from '../stores/workspace'
import type { WorkspaceItem } from '../api/workspace'
import SettingsPane from './SettingsPane.vue'
import NoteEditorPane from '../components/NoteEditorPane.vue'

const health = useHealthStore()
const workspace = useWorkspaceStore()
const route = useRoute()
const router = useRouter()

const isSettingsRoute = computed(() => route.name === 'settings')
const isWorkspaceRoute = computed(
  () => route.name === 'workspace' || route.name === 'workspace-item',
)

const showCreatePanel = ref(false)
const newItemType = ref('note')
const newItemTitle = ref('')
const newParentId = ref<string | null>(null)

const itemTypes = ['note', 'chat', 'agent_config', 'file', 'task', 'settings_view']
const itemTypeLabels: Record<string, string> = {
  note: 'Note',
  chat: 'Chat',
  agent_config: 'Agent',
  file: 'File',
  task: 'Task',
  settings_view: 'Settings',
}

const placeholderText: Record<string, string> = {
  note: 'Note Editor (coming in Batch 4)',
  chat: 'Chat Pane (coming in Batch 6)',
  agent_config: 'Agent Config (coming in Batch 11)',
  file: 'File Pane (coming in Batch 13)',
  task: 'Task Pane (coming in Batch 14)',
  settings_view: 'Settings View',
}

const backendStatus = computed(() => {
  if (health.isLoading && !health.data) return 'checking'
  return health.data?.status ?? 'offline'
})

const databaseStatus = computed(() => health.data?.database.status ?? 'not_configured')

const statusMessage = computed(() => {
  if (health.error) return health.error
  if (health.isLoading) return 'Checking backend health'
  if (health.data) return `Backend ${health.data.status} at ${health.data.app.host}:${health.data.app.port}`
  return 'Backend health not checked'
})

const title = computed(() => {
  if (isSettingsRoute.value) return 'Settings'
  return 'Workspace'
})

const kicker = computed(() => {
  if (isSettingsRoute.value) return 'Batch 1'
  return 'Batch 2'
})

const rootItems = computed(() =>
  workspace.items
    .filter((item) => !item.parent_id)
    .sort((a, b) => a.sort_order - b.sort_order),
)

const selectedItem = computed(() => workspace.selectedItem)

function getChildren(parentId: string): WorkspaceItem[] {
  return workspace.items
    .filter((item) => item.parent_id === parentId)
    .sort((a, b) => a.sort_order - b.sort_order)
}

function handleSelect(id: string) {
  workspace.selectItem(id)
  router.push(`/workspace/${id}`)
}

async function handleDelete(id: string) {
  await workspace.deleteItem(id)
}

function openCreate(parentId: string | null = null) {
  newParentId.value = parentId
  newItemTitle.value = ''
  newItemType.value = 'note'
  showCreatePanel.value = true
}

function cancelCreate() {
  showCreatePanel.value = false
  newParentId.value = null
}

async function confirmCreate() {
  if (!newItemTitle.value.trim()) return
  const item = await workspace.createItem({
    item_type: newItemType.value,
    title: newItemTitle.value.trim(),
    parent_id: newParentId.value,
  })
  if (item) {
    cancelCreate()
    handleSelect(item.id)
  }
}

onMounted(() => {
  void health.refresh()
  void workspace.loadItems()
})

watch(
  () => route.params.itemId,
  (itemId) => {
    if (itemId && typeof itemId === 'string' && isWorkspaceRoute.value) {
      workspace.selectItem(itemId)
    } else if (!itemId && route.name === 'workspace') {
      workspace.selectItem(null)
    }
  },
  { immediate: true },
)
</script>

<template>
  <div class="app-shell">
    <aside class="workspace-sidebar">
      <div class="brand-block">
        <span class="brand-mark" aria-hidden="true"></span>
        <span class="brand-name">LLM Workbench</span>
      </div>

      <nav class="sidebar-nav" aria-label="Workspace">
        <RouterLink to="/workspace" class="sidebar-link">Workspace</RouterLink>
        <RouterLink to="/settings" class="sidebar-link">Settings</RouterLink>
      </nav>

      <div class="sidebar-section" aria-label="Workspace items">
        <div class="sidebar-section-header">
          <span class="sidebar-section-label">Items</span>
          <button
            class="sidebar-add-button"
            type="button"
            title="Add item"
            @click="openCreate(null)"
          >
            +
          </button>
        </div>

        <div v-if="showCreatePanel" class="sidebar-create-panel">
          <select v-model="newItemType" class="sidebar-create-select">
            <option v-for="t in itemTypes" :key="t" :value="t">{{ t }}</option>
          </select>
          <input
            v-model="newItemTitle"
            class="sidebar-create-input"
            type="text"
            placeholder="Title"
            @keyup.enter="confirmCreate()"
          />
          <div class="sidebar-create-actions">
            <button class="sidebar-create-ok" type="button" @click="confirmCreate()">OK</button>
            <button class="sidebar-create-cancel" type="button" @click="cancelCreate()">Cancel</button>
          </div>
        </div>

        <div v-if="rootItems.length === 0 && !workspace.isLoading" class="sidebar-empty muted-label">
          No items yet
        </div>

        <template v-for="item in rootItems" :key="item.id">
          <div
            class="workspace-item"
            :class="{
              'is-active': workspace.selectedItemId === item.id,
              'is-loading': workspace.isLoading,
            }"
            role="treeitem"
            tabindex="0"
            @click="handleSelect(item.id)"
            @keydown.enter="handleSelect(item.id)"
            @keydown.space.prevent="handleSelect(item.id)"
          >
            <span class="wi-type">{{ itemTypeLabels[item.item_type] ?? item.item_type }}</span>
            <span class="wi-title">{{ item.title }}</span>
          </div>

          <template v-if="getChildren(item.id).length > 0">
            <div
              v-for="child in getChildren(item.id)"
              :key="child.id"
              class="workspace-item child-item"
              :class="{ 'is-active': workspace.selectedItemId === child.id }"
              role="treeitem"
              tabindex="0"
              @click="handleSelect(child.id)"
              @keydown.enter="handleSelect(child.id)"
              @keydown.space.prevent="handleSelect(child.id)"
            >
              <span class="wi-type">{{ itemTypeLabels[child.item_type] ?? child.item_type }}</span>
              <span class="wi-title">{{ child.title }}</span>
              <button
                class="wi-delete"
                type="button"
                title="Delete"
                @click.stop="handleDelete(child.id)"
              >
                &times;
              </button>
            </div>
          </template>
        </template>
      </div>
    </aside>

    <main class="main-content">
      <header class="topbar">
        <div>
          <p class="kicker">{{ kicker }}</p>
          <h1>{{ title }}</h1>
        </div>

        <button
          class="refresh-button"
          type="button"
          :disabled="health.isLoading"
          @click="health.refresh()"
        >
          {{ health.isLoading ? 'Checking' : 'Refresh' }}
        </button>
      </header>

      <section class="status-panel" aria-labelledby="system-status-title">
        <div class="section-title-row">
          <h2 id="system-status-title">System Status</h2>
          <span
            class="status-pill"
            :class="{ 'is-ok': backendStatus === 'ok', 'is-error': health.error }"
          >
            {{ backendStatus }}
          </span>
        </div>

        <dl class="status-grid">
          <div>
            <dt>Service</dt>
            <dd>{{ health.data?.service ?? 'llm_workbench' }}</dd>
          </div>
          <div>
            <dt>Version</dt>
            <dd>{{ health.data?.version ?? '0.1.0' }}</dd>
          </div>
          <div>
            <dt>Database</dt>
            <dd>{{ databaseStatus }}</dd>
          </div>
          <div>
            <dt>Data Dir</dt>
            <dd>{{ health.data?.app.app_data_dir ?? './data' }}</dd>
          </div>
        </dl>
      </section>

      <SettingsPane v-if="isSettingsRoute" />

      <section v-else-if="isWorkspaceRoute" class="workspace-stage" aria-labelledby="ws-content-title">
        <div v-if="!selectedItem" class="empty-state">
          Select or create a workspace item
        </div>

        <div v-else>
          <template v-if="selectedItem.item_type === 'note'">
            <NoteEditorPane :workspaceItem="selectedItem" />
          </template>
          <template v-else>
            <div class="section-title-row">
              <div>
                <h2 id="ws-content-title">{{ itemTypeLabels[selectedItem.item_type] ?? selectedItem.item_type }}</h2>
                <p class="section-subtitle">{{ selectedItem.title }}</p>
              </div>
              <button
                class="secondary-button"
                type="button"
                @click="handleDelete(selectedItem.id)"
              >
                Delete
              </button>
            </div>

            <div class="placeholder-pane">
              <p>{{ placeholderText[selectedItem.item_type] ?? 'Unknown item type' }}</p>
              <dl class="item-meta">
                <dt>ID</dt>
                <dd>{{ selectedItem.id }}</dd>
                <dt>Type</dt>
                <dd>{{ selectedItem.item_type }}</dd>
                <dt>Parent</dt>
                <dd>{{ selectedItem.parent_id ?? '(root)' }}</dd>
                <dt>Sort Order</dt>
                <dd>{{ selectedItem.sort_order }}</dd>
                <dt>Created</dt>
                <dd>{{ new Date(selectedItem.created_at).toLocaleString() }}</dd>
                <dt>Updated</dt>
                <dd>{{ new Date(selectedItem.updated_at).toLocaleString() }}</dd>
              </dl>
            </div>
          </template>
        </div>
      </section>

      <footer class="status-region" role="status" aria-live="polite">
        <span :class="{ 'text-error': health.error }">{{ statusMessage }}</span>
        <span v-if="health.lastCheckedAt">Last check {{ health.lastCheckedAt }}</span>
      </footer>
    </main>
  </div>
</template>

<style scoped>
.sidebar-section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.sidebar-section-label {
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--sidebar-muted);
}

.sidebar-add-button {
  padding: 0 6px;
  border: none;
  background: var(--sidebar-active);
  color: var(--sidebar-muted);
  font-size: 16px;
  line-height: 1.4;
  border-radius: 4px;
  cursor: pointer;
}

.sidebar-add-button:hover {
  color: #ffffff;
}

.sidebar-empty {
  font-size: 13px;
  padding-inline: 10px;
}

.sidebar-create-panel {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 8px;
  background: var(--sidebar-active);
  border-radius: 6px;
}

.sidebar-create-select,
.sidebar-create-input {
  padding: 4px 8px;
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.08);
  color: #ffffff;
  font-size: 13px;
}

.sidebar-create-select option {
  background: var(--sidebar);
  color: #ffffff;
}

.sidebar-create-actions {
  display: flex;
  gap: 4px;
}

.sidebar-create-ok {
  padding: 3px 10px;
  border: none;
  background: var(--accent);
  color: #fff;
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
}

.sidebar-create-cancel {
  padding: 3px 10px;
  border: 1px solid rgba(255, 255, 255, 0.15);
  background: none;
  color: var(--sidebar-muted);
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
}

.wi-type {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  opacity: 0.55;
  min-width: 42px;
}

.wi-title {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.workspace-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.workspace-item.is-active .wi-type {
  opacity: 0.75;
}

.child-item {
  padding-left: 22px;
  font-size: 13px;
}

.child-item .wi-type {
  min-width: 36px;
  font-size: 9px;
}

.wi-delete {
  display: none;
  padding: 0 4px;
  border: none;
  background: none;
  color: var(--sidebar-muted);
  font-size: 14px;
  line-height: 1;
  cursor: pointer;
  border-radius: 2px;
}

.child-item:hover .wi-delete {
  display: inline;
}

.wi-delete:hover {
  background: var(--danger);
  color: #fff;
}

.placeholder-pane {
  padding: 24px;
  background: var(--surface-subtle);
  border-radius: 8px;
  border: 1px dashed var(--border);
  text-align: center;
}

.placeholder-pane p {
  margin: 0 0 16px;
  color: var(--muted);
  font-size: 14px;
}

.item-meta {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 4px 16px;
  text-align: left;
  font-size: 12px;
  color: var(--muted);
  margin: 0 auto;
  max-width: 360px;
}

.item-meta dt {
  font-weight: 600;
}

.item-meta dd {
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
