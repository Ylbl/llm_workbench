<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useNotesStore } from '../stores/notes'
import { useWorkspaceStore } from '../stores/workspace'
import type { WorkspaceItem } from '../api/workspace'
import TiptapNoteEditor from './TiptapNoteEditor.vue'

const props = defineProps<{
  workspaceItem: WorkspaceItem
}>()

const notesStore = useNotesStore()
const workspaceStore = useWorkspaceStore()

const noteTitle = ref(props.workspaceItem.title)
const showRevisions = ref(false)
const isDirty = ref(false)
const currentJson = ref<Record<string, unknown>>({ type: 'doc', content: [] })
let autoSaveTimer: ReturnType<typeof setTimeout> | null = null

const saveStatusText = computed(() => {
  switch (notesStore.saveStatus) {
    case 'saving': return 'Saving...'
    case 'saved': return 'Saved'
    case 'error': return 'Save failed'
    default: return isDirty.value ? 'Unsaved changes' : ''
  }
})

const note = computed(() => notesStore.currentNote)

async function loadOrCreateNote() {
  const note = await notesStore.loadNote(props.workspaceItem.id)
  if (note) {
    noteTitle.value = note.title
    currentJson.value = note.document_json as Record<string, unknown>
  } else {
    // note doesn't exist yet, create one
    const newNote = await notesStore.create({
      title: props.workspaceItem.title,
    })
    if (newNote) {
      currentJson.value = newNote.document_json as Record<string, unknown>
      noteTitle.value = newNote.title
    }
  }
}

function onEditorUpdate(json: Record<string, unknown>) {
  currentJson.value = json
  isDirty.value = true
  scheduleAutoSave()
}

function scheduleAutoSave() {
  if (autoSaveTimer) clearTimeout(autoSaveTimer)
  autoSaveTimer = setTimeout(() => {
    void doSave()
  }, 1500)
}

async function doSave() {
  if (!note.value) return
  isDirty.value = false
  await notesStore.saveNote(note.value.id, {
    title: noteTitle.value,
    document_json: currentJson.value,
  })
  workspaceStore.updateItem(props.workspaceItem.id, {
    title: noteTitle.value,
  })
}

async function handleSave() {
  if (autoSaveTimer) clearTimeout(autoSaveTimer)
  await doSave()
}

async function loadRevisions() {
  if (!note.value) return
  showRevisions.value = !showRevisions.value
  if (showRevisions.value) {
    await notesStore.loadRevisions(note.value.id)
  }
}

async function handleRestore(revisionId: string) {
  if (!note.value) return
  const restored = await notesStore.restoreRevision(note.value.id, revisionId)
  if (restored) {
    currentJson.value = restored.document_json as Record<string, unknown>
    noteTitle.value = restored.title
    isDirty.value = false
  }
}

function handleKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key === 's') {
    e.preventDefault()
    void handleSave()
  }
}

watch(() => props.workspaceItem.id, () => {
  void loadOrCreateNote()
})

onMounted(() => {
  void loadOrCreateNote()
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
  if (autoSaveTimer) clearTimeout(autoSaveTimer)
})
</script>

<template>
  <div class="note-editor-pane">
    <div class="note-toolbar">
      <div class="note-toolbar-left">
        <input
          v-model="noteTitle"
          class="note-title-input"
          type="text"
          placeholder="笔记标题"
          @change="scheduleAutoSave()"
        />
      </div>
      <div class="note-toolbar-right">
        <span
          class="note-save-status"
          :class="{
            'is-saving': notesStore.saveStatus === 'saving',
            'is-saved': notesStore.saveStatus === 'saved',
            'is-error': notesStore.saveStatus === 'error',
            'is-dirty': isDirty && notesStore.saveStatus === 'idle',
          }"
        >
          {{ saveStatusText }}
        </span>
        <button
          class="secondary-button note-save-btn"
          type="button"
          :disabled="notesStore.isSaving || !isDirty"
          @click="handleSave()"
        >
          Save
        </button>
        <button
          v-if="note"
          class="secondary-button"
          type="button"
          @click="loadRevisions()"
        >
          {{ showRevisions ? '隐藏版本' : 'Revisions' }}
        </button>
      </div>
    </div>

    <div v-if="notesStore.error" class="note-error">
      {{ notesStore.error }}
    </div>

    <div v-if="notesStore.isLoading && !note" class="note-loading">
      加载中...
    </div>

    <TiptapNoteEditor
      v-else
      :content="currentJson"
      :editable="true"
      @update="onEditorUpdate"
      @save="handleSave()"
    />

    <div v-if="showRevisions && notesStore.revisions.length > 0" class="note-revisions">
      <h3 class="revisions-title">历史版本</h3>
      <div
        v-for="rev in notesStore.revisions"
        :key="rev.id"
        class="revision-item"
      >
        <div class="revision-meta">
          <span class="revision-date">{{ new Date(rev.created_at).toLocaleString() }}</span>
          <span v-if="rev.reason" class="revision-reason">{{ rev.reason }}</span>
          <span class="revision-by">{{ rev.created_by }}</span>
        </div>
        <div class="revision-content">
          <pre class="revision-plaintext">{{ rev.plain_text.slice(0, 200) }}{{ rev.plain_text.length > 200 ? '...' : '' }}</pre>
        </div>
        <button
          class="secondary-button revision-restore"
          type="button"
          @click="handleRestore(rev.id)"
        >
          Restore
        </button>
      </div>
    </div>
    <div v-else-if="showRevisions" class="note-revisions">
      <p class="muted-label">暂无历史版本</p>
    </div>
  </div>
</template>

<style scoped>
.note-editor-pane {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.note-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 20px;
  border-bottom: 1px solid var(--border);
  background: var(--surface);
}

.note-toolbar-left {
  flex: 1;
  min-width: 0;
}

.note-title-input {
  width: 100%;
  padding: 4px 8px;
  border: 1px solid transparent;
  border-radius: 4px;
  background: none;
  color: var(--text);
  font-size: 18px;
  font-weight: 600;
  outline: none;
}

.note-title-input:hover {
  border-color: var(--border);
}

.note-title-input:focus {
  border-color: var(--accent);
}

.note-toolbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.note-save-status {
  font-size: 12px;
  color: var(--muted);
  min-width: 80px;
  text-align: right;
}

.note-save-status.is-saving {
  color: var(--warning, #b45309);
}

.note-save-status.is-saved {
  color: var(--accent);
  animation: fade-saved 2s ease-out forwards;
}

.note-save-status.is-error {
  color: var(--danger);
}

.note-save-status.is-dirty {
  color: var(--warning, #b45309);
}

@keyframes fade-saved {
  0% { opacity: 1; }
  100% { opacity: 0.3; }
}

.note-save-btn {
  min-width: 64px;
}

.note-error {
  padding: 8px 20px;
  background: var(--danger-soft, #fee4e2);
  color: var(--danger);
  font-size: 13px;
  border-bottom: 1px solid var(--border);
}

.note-loading {
  padding: 40px 20px;
  text-align: center;
  color: var(--muted);
}

.note-revisions {
  border-top: 1px solid var(--border);
  padding: 16px 20px;
  background: var(--surface-subtle);
  max-height: 300px;
  overflow-y: auto;
}

.revisions-title {
  margin: 0 0 12px;
  font-size: 14px;
  color: var(--muted);
}

.revision-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--surface);
  margin-bottom: 8px;
}

.revision-meta {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 160px;
  font-size: 12px;
  color: var(--muted);
}

.revision-date {
  font-weight: 600;
}

.revision-by {
  color: var(--accent);
}

.revision-content {
  flex: 1;
  min-width: 0;
}

.revision-plaintext {
  margin: 0;
  font-size: 12px;
  font-family: var(--font-mono);
  white-space: pre-wrap;
  word-break: break-word;
  color: var(--text);
}

.revision-restore {
  min-width: 72px;
  font-size: 12px;
}
</style>
