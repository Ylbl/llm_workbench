<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useSettingsStore } from '../stores/settings'
import type { SettingsMap } from '../api/settings'
import LlmSettingsPane from './LlmSettingsPane.vue'
import MathDemo from './MathDemo.vue'

type SettingsTab = 'raw' | 'llm' | 'math'

const settings = useSettingsStore()
const rawJson = ref('{}')
const localError = ref<string | null>(null)
const activeTab = ref<SettingsTab>('raw')

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
  <div class="settings-pane" aria-labelledby="settings-title">
    <div class="settings-tabs">
      <button
        :class="{ active: activeTab === 'raw' }"
        @click="activeTab = 'raw'"
      >原始设置</button>
      <button
        :class="{ active: activeTab === 'llm' }"
        @click="activeTab = 'llm'"
      >LLM 设置</button>
      <button
        :class="{ active: activeTab === 'math' }"
        @click="activeTab = 'math'"
      >数学演示</button>
    </div>

    <section v-if="activeTab === 'raw'" class="settings-tab-content">
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

    <section v-else-if="activeTab === 'llm'" class="settings-tab-content">
      <LlmSettingsPane />
    </section>

    <section v-else-if="activeTab === 'math'" class="settings-tab-content">
      <MathDemo />
    </section>
  </div>
</template>

<style scoped>
.settings-pane {
  padding: 20px;
}

.settings-tabs {
  display: flex;
  gap: 0;
  margin-bottom: 20px;
  border-bottom: 1px solid var(--border);
}

.settings-tabs button {
  padding: 8px 20px;
  border: none;
  background: none;
  color: var(--muted);
  font-size: 14px;
  cursor: pointer;
  border-bottom: 2px solid transparent;
}

.settings-tabs button.active {
  color: var(--accent);
  border-bottom-color: var(--accent);
}

.settings-tab-content {
  min-height: 0;
}

.section-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}

.section-subtitle {
  margin: 4px 0 0;
  color: var(--muted);
  font-size: 14px;
}

.settings-actions {
  display: flex;
  gap: 8px;
}

.json-editor {
  width: 100%;
  min-height: 420px;
  padding: 14px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--surface-subtle);
  color: var(--text);
  font-family: var(--font-mono);
  font-size: 14px;
  line-height: 1.6;
  resize: vertical;
}

.json-editor:focus {
  outline: 2px solid var(--focus);
  outline-offset: 2px;
}

.inline-status {
  margin-top: 10px;
  font-size: 13px;
  color: var(--muted);
}

.text-error {
  color: var(--danger);
}

.secondary-button,
.primary-button {
  padding: 6px 16px;
  border-radius: 4px;
  font-size: 14px;
  cursor: pointer;
  border: 1px solid var(--border);
}

.primary-button {
  background: var(--accent);
  color: #fff;
  border-color: var(--accent);
}

.primary-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.secondary-button {
  background: var(--surface);
  color: var(--text);
}

.secondary-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
