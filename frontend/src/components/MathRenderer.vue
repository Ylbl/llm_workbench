<script setup lang="ts">
import { computed } from 'vue'
import { renderFormula } from '../utils/math'

const props = defineProps<{
  latex: string
  displayMode?: boolean
}>()

const html = computed(() => renderFormula(props.latex, props.displayMode ?? false))
</script>

<template>
  <component
    :is="displayMode ? 'div' : 'span'"
    class="math-renderer"
    :class="{ 'math-block': displayMode, 'math-inline': !displayMode }"
    v-html="html"
  />
</template>

<style scoped>
.math-inline {
  display: inline;
}

.math-block {
  display: block;
  margin: 12px 0;
  text-align: center;
}

.math-renderer :deep(.katex) {
  font-size: 1.1em;
}

.math-block :deep(.katex) {
  font-size: 1.21em;
}

.math-renderer :deep(.katex-error) {
  color: var(--danger, #b42318);
  font-size: 0.9em;
}
</style>
