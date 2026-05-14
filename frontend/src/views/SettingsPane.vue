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
    return '加载中'
  }

  if (settings.isSaving) {
    return '保存中'
  }

  if (settings.lastSavedAt) {
    return `上次保存: ${settings.lastSavedAt}`
  }

  return '就绪'
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
    localError.value = error instanceof Error ? error.message : 'JSON 格式无效'
    return
  }

  if (!parsed || Array.isArray(parsed) || typeof parsed !== 'object') {
    localError.value = '设置必须是 JSON 对象'
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
        <h2 id="settings-title">设置 JSON</h2>
        <p class="section-subtitle">原始持久化设置</p>
      </div>
      <div class="settings-actions">
        <button class="secondary-button" type="button" :disabled="settings.isLoading" @click="loadSettings">
          重新加载
        </button>
        <button class="primary-button" type="button" :disabled="settings.isSaving" @click="saveSettings">
          {{ settings.isSaving ? '保存中' : '保存' }}
        </button>
      </div>
    </div>

    <textarea
      v-model="rawJson"
      class="json-editor"
      spellcheck="false"
      aria-label="设置 JSON 编辑器"
    ></textarea>

    <div class="inline-status" :class="{ 'text-error': localError || settings.error }" role="status">
      {{ status }}
    </div>
  </section>
</template>
