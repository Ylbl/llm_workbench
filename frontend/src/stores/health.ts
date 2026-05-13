import { defineStore } from 'pinia'
import { fetchHealth, type HealthResponse } from '../api/health'

interface HealthState {
  data: HealthResponse | null
  isLoading: boolean
  error: string | null
  lastCheckedAt: string | null
}

export const useHealthStore = defineStore('health', {
  state: (): HealthState => ({
    data: null,
    isLoading: false,
    error: null,
    lastCheckedAt: null,
  }),

  actions: {
    async refresh() {
      this.isLoading = true
      this.error = null

      try {
        this.data = await fetchHealth()
        this.lastCheckedAt = new Date().toLocaleTimeString()
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Health check failed'
      } finally {
        this.isLoading = false
      }
    },
  },
})
