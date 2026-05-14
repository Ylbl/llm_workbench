import { defineStore } from 'pinia'
import { fetchSettings, patchSettings, type SettingsMap } from '../api/settings'

interface SettingsState {
  values: SettingsMap
  isLoading: boolean
  isSaving: boolean
  error: string | null
  lastSavedAt: string | null
}

export const useSettingsStore = defineStore('settings', {
  state: (): SettingsState => ({
    values: {},
    isLoading: false,
    isSaving: false,
    error: null,
    lastSavedAt: null,
  }),

  actions: {
    async load() {
      this.isLoading = true
      this.error = null

      try {
        this.values = await fetchSettings()
      } catch (error) {
        this.error = error instanceof Error ? error.message : '设置加载失败'
      } finally {
        this.isLoading = false
      }
    },

    async save(values: SettingsMap) {
      this.isSaving = true
      this.error = null

      try {
        this.values = await patchSettings(values)
        this.lastSavedAt = new Date().toLocaleTimeString()
      } catch (error) {
        this.error = error instanceof Error ? error.message : '设置保存失败'
        throw error
      } finally {
        this.isSaving = false
      }
    },
  },
})
