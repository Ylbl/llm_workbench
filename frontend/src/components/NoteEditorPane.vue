<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useNotesStore } from '../stores/notes'
import { useWorkspaceStore } from '../stores/workspace'
import type { WorkspaceItem } from '../api/workspace'
import { fetchNoteByWorkspaceItem } from '../api/notes'
import TiptapNoteEditor from './TiptapNoteEditor.vue'

const props = defineProps<{
  workspaceItem: WorkspaceItem
}>()

const notesStore = useNotesStore()
const workspaceStore = useWorkspaceStore()

const noteTitle = ref(props.workspaceItem.title)
const isDirty = ref(false)
const currentJson = ref<Record<string, unknown>>({ type: 'doc', content: [] })
let autoSaveTimer: ReturnType<typeof setTimeout> | null = null

const note = computed(() => notesStore.currentNote)

async function loadOrCreateNote() {
  try {
    const note = await fetchNoteByWorkspaceItem(props.workspaceItem.id)
    noteTitle.value = note.title
    currentJson.value = note.document_json as Record<string, unknown>
    notesStore.currentNoteId = note.id
    const idx = notesStore.notes.findIndex((n) => n.id === note.id)
    if (idx >= 0) notesStore.notes[idx] = note
    else notesStore.notes.push(note)
  } catch {
    const newNote = await notesStore.create({
      title: props.workspaceItem.title,
      workspace_item_id: props.workspaceItem.id,
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
  </div>
</template>

<style scoped>
.note-editor-pane {
  display: flex;
  flex-direction: column;
  gap: 0;
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
</style>
