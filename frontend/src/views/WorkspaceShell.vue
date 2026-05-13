<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { useHealthStore } from '../stores/health'
import SettingsPane from './SettingsPane.vue'
import WorkspaceHome from './WorkspaceHome.vue'

const health = useHealthStore()
const route = useRoute()

const isSettingsRoute = computed(() => route.name === 'settings')

const backendStatus = computed(() => {
  if (health.isLoading && !health.data) {
    return 'checking'
  }

  return health.data?.status ?? 'offline'
})

const databaseStatus = computed(() => health.data?.database.status ?? 'not_configured')

const statusMessage = computed(() => {
  if (health.error) {
    return health.error
  }

  if (health.isLoading) {
    return 'Checking backend health'
  }

  if (health.data) {
    return `Backend ${health.data.status} at ${health.data.app.host}:${health.data.app.port}`
  }

  return 'Backend health not checked'
})

const title = computed(() => (isSettingsRoute.value ? 'Settings' : 'Development Baseline'))
const kicker = computed(() => (isSettingsRoute.value ? 'Batch 1' : 'Batch 0'))

onMounted(() => {
  void health.refresh()
})
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
        <button class="workspace-item is-active" type="button">Health</button>
        <button class="workspace-item" type="button" disabled>Notes</button>
        <button class="workspace-item" type="button" disabled>Chats</button>
        <button class="workspace-item" type="button" disabled>Agents</button>
      </div>
    </aside>

    <main class="main-content">
      <header class="topbar">
        <div>
          <p class="kicker">{{ kicker }}</p>
          <h1>{{ title }}</h1>
        </div>

        <button class="refresh-button" type="button" :disabled="health.isLoading" @click="health.refresh()">
          {{ health.isLoading ? 'Checking' : 'Refresh' }}
        </button>
      </header>

      <section class="status-panel" aria-labelledby="system-status-title">
        <div class="section-title-row">
          <h2 id="system-status-title">System Status</h2>
          <span class="status-pill" :class="{ 'is-ok': backendStatus === 'ok', 'is-error': health.error }">
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
      <WorkspaceHome v-else />

      <footer class="status-region" role="status" aria-live="polite">
        <span :class="{ 'text-error': health.error }">{{ statusMessage }}</span>
        <span v-if="health.lastCheckedAt">Last check {{ health.lastCheckedAt }}</span>
      </footer>
    </main>
  </div>
</template>
