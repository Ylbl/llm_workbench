<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useWorkspaceStore } from '../stores/workspace'
import WorkspaceTree from '../components/WorkspaceTree.vue'

const store = useWorkspaceStore()

const showCreateMenu = ref(false)
const createParentId = ref<string | null>(null)
const newItemTitle = ref('')
const newItemType = ref('note')

const itemTypes = ['note', 'chat', 'agent_config', 'file', 'task', 'settings_view']

const placeholderText: Record<string, string> = {
  note: 'Note Editor (coming in Batch 4)',
  chat: 'Chat Pane (coming in Batch 6)',
  agent_config: 'Agent Config (coming in Batch 11)',
  file: 'File Pane (coming in Batch 13)',
  task: 'Task Pane (coming in Batch 14)',
  settings_view: 'Settings View',
}

onMounted(() => {
  void store.loadItems()
})

function openCreateMenu(parentId: string | null = null) {
  createParentId.value = parentId
  newItemTitle.value = ''
  newItemType.value = 'note'
  showCreateMenu.value = true
}

function cancelCreate() {
  showCreateMenu.value = false
  createParentId.value = null
  newItemTitle.value = ''
}

async function confirmCreate() {
  if (!newItemTitle.value.trim()) return

  const item = await store.createItem({
    item_type: newItemType.value,
    title: newItemTitle.value.trim(),
    parent_id: createParentId.value,
  })

  if (item) {
    cancelCreate()
    store.selectItem(item.id)
  }
}

async function handleDelete(id: string) {
  await store.deleteItem(id)
}

function handleSelect(id: string) {
  store.selectItem(id)
}

const selectedItem = computed(() => store.selectedItem)
</script>

<template>
  <section class="workspace-stage" aria-labelledby="workspace-stage-title">
    <div class="section-title-row">
      <h2 id="workspace-stage-title">Workspace</h2>
      <button class="create-button" type="button" @click="openCreateMenu(null)">
        + New
      </button>
    </div>

    <div v-if="showCreateMenu" class="create-panel">
      <select v-model="newItemType" class="create-select">
        <option v-for="t in itemTypes" :key="t" :value="t">{{ t }}</option>
      </select>
      <input
        v-model="newItemTitle"
        class="create-input"
        type="text"
        placeholder="Item title"
        @keyup.enter="confirmCreate()"
      />
      <button class="create-confirm" type="button" @click="confirmCreate()">Create</button>
      <button class="create-cancel" type="button" @click="cancelCreate()">Cancel</button>
    </div>

    <div class="workspace-layout">
      <nav class="workspace-tree-panel" aria-label="Workspace tree">
        <WorkspaceTree
          :items="store.items"
          :selectedId="store.selectedItemId"
          @select="handleSelect"
          @delete="handleDelete"
          @create="(parentId: string | null) => openCreateMenu(parentId)"
        />
      </nav>

      <div class="workspace-content">
        <div v-if="!selectedItem" class="empty-state">
          Select or create a workspace item
        </div>

        <div v-else class="item-pane">
          <div class="item-pane-header">
            <h3>{{ selectedItem.item_type }}</h3>
            <span class="muted-label">{{ selectedItem.title }}</span>
          </div>

          <div class="item-pane-body">
            <div class="placeholder-pane">
              <p>{{ placeholderText[selectedItem.item_type] ?? 'Unknown item type' }}</p>
              <dl class="item-meta">
                <dt>ID</dt>
                <dd>{{ selectedItem.id }}</dd>
                <dt>Type</dt>
                <dd>{{ selectedItem.item_type }}</dd>
                <dt>Created</dt>
                <dd>{{ new Date(selectedItem.created_at).toLocaleString() }}</dd>
                <dt>Updated</dt>
                <dd>{{ new Date(selectedItem.updated_at).toLocaleString() }}</dd>
              </dl>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.create-button {
  padding: 4px 12px;
  border: 1px solid var(--color-accent);
  background: var(--color-accent);
  color: #fff;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
}

.create-button:hover {
  opacity: 0.85;
}

.create-panel {
  display: flex;
  gap: 8px;
  align-items: center;
  padding: 8px 12px;
  background: var(--color-surface);
  border-radius: 6px;
  margin-bottom: 12px;
}

.create-select {
  padding: 4px 8px;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  background: var(--color-bg);
  color: var(--color-text);
  font-size: 13px;
}

.create-input {
  flex: 1;
  padding: 4px 8px;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  background: var(--color-bg);
  color: var(--color-text);
  font-size: 13px;
}

.create-confirm {
  padding: 4px 12px;
  border: none;
  background: var(--color-accent);
  color: #fff;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
}

.create-cancel {
  padding: 4px 12px;
  border: 1px solid var(--color-border);
  background: none;
  color: var(--color-muted);
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
}

.workspace-layout {
  display: grid;
  grid-template-columns: 240px 1fr;
  gap: 16px;
  min-height: 300px;
}

.workspace-tree-panel {
  border-right: 1px solid var(--color-border);
  padding-right: 12px;
  overflow-y: auto;
}

.workspace-content {
  min-height: 200px;
}

.item-pane-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--color-border);
  margin-bottom: 16px;
}

.item-pane-header h3 {
  margin: 0;
  font-size: 14px;
  text-transform: uppercase;
  opacity: 0.6;
}

.item-pane-header .muted-label {
  font-size: 18px;
  font-weight: 600;
}

.placeholder-pane {
  padding: 24px;
  background: var(--color-surface);
  border-radius: 6px;
  text-align: center;
}

.placeholder-pane p {
  margin: 0 0 16px;
  color: var(--color-muted);
  font-size: 14px;
}

.item-meta {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 4px 12px;
  text-align: left;
  font-size: 12px;
  color: var(--color-muted);
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
