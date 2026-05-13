<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useSettingsStore } from '../stores/settings'
import type { SettingsMap } from '../api/settings'

const settings = useSettingsStore()
const rawJson = ref('{}')
const localError = ref<string | null>(null)

const status = computed(() => {
  if (localError.value) {
    return localError.value
  }

  if (settings.error) {
    return settings.error
  }

  if (settings.isLoading) {
    return 'Loading settings'
  }

  if (settings.isSaving) {
    return 'Saving settings'
  }

  if (settings.lastSavedAt) {
    return `Saved at ${settings.lastSavedAt}`
  }

  return 'Settings ready'
})

watch(
  () => settings.values,
  (value) => {
    rawJson.value = JSON.stringify(value, null, 2)
  },
  { deep: true },
)

async function loadSettings() {
  localError.value = null
  await settings.load()
  rawJson.value = JSON.stringify(settings.values, null, 2)
}

async function saveSettings() {
  localError.value = null

  let parsed: unknown
  try {
    parsed = JSON.parse(rawJson.value)
  } catch (error) {
    localError.value = error instanceof Error ? error.message : 'Invalid JSON'
    return
  }

  if (!parsed || Array.isArray(parsed) || typeof parsed !== 'object') {
    localError.value = 'Settings JSON must be an object'
    return
  }

  await settings.save(parsed as SettingsMap)
}

onMounted(() => {
  void loadSettings()
})
</script>

<template>
  <section class="settings-pane" aria-labelledby="settings-title">
    <div class="section-title-row">
      <div>
        <h2 id="settings-title">Settings JSON</h2>
        <p class="section-subtitle">Raw persisted settings object</p>
      </div>
      <div class="settings-actions">
        <button class="secondary-button" type="button" :disabled="settings.isLoading" @click="loadSettings">
          Reload
        </button>
        <button class="primary-button" type="button" :disabled="settings.isSaving" @click="saveSettings">
          {{ settings.isSaving ? 'Saving' : 'Save' }}
        </button>
      </div>
    </div>

    <textarea
      v-model="rawJson"
      class="json-editor"
      spellcheck="false"
      aria-label="Settings JSON editor"
    ></textarea>

    <div class="inline-status" :class="{ 'text-error': localError || settings.error }" role="status">
      {{ status }}
    </div>
  </section>
</template>
