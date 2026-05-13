import { defineStore } from 'pinia'
import {
  fetchWorkspaceItems,
  createWorkspaceItem,
  updateWorkspaceItem,
  deleteWorkspaceItem,
  type WorkspaceItem,
  type CreateWorkspaceItemRequest,
  type UpdateWorkspaceItemRequest,
} from '../api/workspace'

interface WorkspaceState {
  items: WorkspaceItem[]
  selectedItemId: string | null
  isLoading: boolean
  isSaving: boolean
  error: string | null
}

export const useWorkspaceStore = defineStore('workspace', {
  state: (): WorkspaceState => ({
    items: [],
    selectedItemId: null,
    isLoading: false,
    isSaving: false,
    error: null,
  }),

  getters: {
    selectedItem(state): WorkspaceItem | null {
      if (!state.selectedItemId) return null
      return state.items.find((item) => item.id === state.selectedItemId) ?? null
    },
  },

  actions: {
    async loadItems() {
      this.isLoading = true
      this.error = null

      try {
        this.items = await fetchWorkspaceItems()
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to load workspace items'
      } finally {
        this.isLoading = false
      }
    },

    async createItem(req: CreateWorkspaceItemRequest): Promise<WorkspaceItem | null> {
      this.isSaving = true
      this.error = null

      try {
        const item = await createWorkspaceItem(req)
        this.items.push(item)
        return item
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to create workspace item'
        return null
      } finally {
        this.isSaving = false
      }
    },

    async updateItem(id: string, req: UpdateWorkspaceItemRequest): Promise<WorkspaceItem | null> {
      this.isSaving = true
      this.error = null

      try {
        const item = await updateWorkspaceItem(id, req)
        const index = this.items.findIndex((i) => i.id === id)
        if (index !== -1) {
          this.items[index] = item
        }
        return item
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to update workspace item'
        return null
      } finally {
        this.isSaving = false
      }
    },

    async deleteItem(id: string): Promise<boolean> {
      this.isSaving = true
      this.error = null

      try {
        await deleteWorkspaceItem(id)
        this.items = this.items.filter((i) => i.id !== id)
        if (this.selectedItemId === id) {
          this.selectedItemId = null
        }
        return true
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to delete workspace item'
        return false
      } finally {
        this.isSaving = false
      }
    },

    selectItem(id: string | null) {
      this.selectedItemId = id
    },
  },
})
