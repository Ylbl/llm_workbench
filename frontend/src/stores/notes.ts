import { defineStore } from 'pinia'
import {
  fetchNotes,
  fetchNote,
  createNote,
  updateNote,
  deleteNote,
  fetchNoteRevisions,
  restoreNoteRevision,
  type Note,
  type NoteRevision,
  type CreateNoteRequest,
  type UpdateNoteRequest,
} from '../api/notes'

interface NotesState {
  notes: Note[]
  revisions: NoteRevision[]
  currentNoteId: string | null
  isLoading: boolean
  isSaving: boolean
  isLoadingRevisions: boolean
  saveStatus: 'idle' | 'saving' | 'saved' | 'error'
  error: string | null
}

export const useNotesStore = defineStore('notes', {
  state: (): NotesState => ({
    notes: [],
    revisions: [],
    currentNoteId: null,
    isLoading: false,
    isSaving: false,
    isLoadingRevisions: false,
    saveStatus: 'idle',
    error: null,
  }),

  getters: {
    currentNote(state): Note | null {
      if (!state.currentNoteId) return null
      return state.notes.find((n) => n.id === state.currentNoteId) ?? null
    },
  },

  actions: {
    async loadNotes() {
      this.isLoading = true
      this.error = null
      try {
        this.notes = await fetchNotes()
      } catch (e) {
        this.error = e instanceof Error ? e.message : '加载笔记失败'
      } finally {
        this.isLoading = false
      }
    },

    async create(req: CreateNoteRequest): Promise<Note | null> {
      this.isSaving = true
      this.error = null
      try {
        const note = await createNote(req)
        this.notes.unshift(note)
        return note
      } catch (e) {
        this.error = e instanceof Error ? e.message : '创建笔记失败'
        return null
      } finally {
        this.isSaving = false
      }
    },

    async loadNote(id: string): Promise<Note | null> {
      this.isLoading = true
      this.error = null
      try {
        const note = await fetchNote(id)
        const idx = this.notes.findIndex((n) => n.id === id)
        if (idx >= 0) this.notes[idx] = note
        else this.notes.push(note)
        this.currentNoteId = id
        return note
      } catch (e) {
        this.error = e instanceof Error ? e.message : '加载单条笔记失败'
        return null
      } finally {
        this.isLoading = false
      }
    },

    async saveNote(
      id: string,
      req: UpdateNoteRequest,
    ): Promise<Note | null> {
      this.isSaving = true
      this.saveStatus = 'saving'
      this.error = null
      try {
        const note = await updateNote(id, req)
        const idx = this.notes.findIndex((n) => n.id === id)
        if (idx >= 0) this.notes[idx] = note
        this.saveStatus = 'saved'
        return note
      } catch (e) {
        this.error = e instanceof Error ? e.message : '保存笔记失败'
        this.saveStatus = 'error'
        return null
      } finally {
        this.isSaving = false
      }
    },

    async removeNote(id: string): Promise<boolean> {
      this.isSaving = true
      try {
        await deleteNote(id)
        this.notes = this.notes.filter((n) => n.id !== id)
        if (this.currentNoteId === id) this.currentNoteId = null
        return true
      } catch (e) {
        this.error = e instanceof Error ? e.message : '删除笔记失败'
        return false
      } finally {
        this.isSaving = false
      }
    },

    async loadRevisions(noteId: string) {
      this.isLoadingRevisions = true
      try {
        this.revisions = await fetchNoteRevisions(noteId)
      } catch (e) {
        this.error = e instanceof Error ? e.message : '加载历史版本失败'
      } finally {
        this.isLoadingRevisions = false
      }
    },

    async restoreRevision(
      noteId: string,
      revisionId: string,
    ): Promise<Note | null> {
      this.isSaving = true
      try {
        const note = await restoreNoteRevision(noteId, revisionId)
        const idx = this.notes.findIndex((n) => n.id === noteId)
        if (idx >= 0) this.notes[idx] = note
        this.saveStatus = 'saved'
        return note
      } catch (e) {
        this.error = e instanceof Error ? e.message : '恢复历史版本失败'
        return null
      } finally {
        this.isSaving = false
      }
    },
  },
})
