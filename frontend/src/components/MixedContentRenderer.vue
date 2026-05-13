<script setup lang="ts">
import { computed } from 'vue'
import { parseMixedContent, type MathSegment } from '../utils/math'
import MathRenderer from './MathRenderer.vue'

const props = defineProps<{
  content: string
}>()

const segments = computed<MathSegment[]>(() => parseMixedContent(props.content))
</script>

<template>
  <div class="mixed-content">
    <template v-for="(seg, index) in segments" :key="index">
      <span v-if="seg.type === 'text'" class="text-segment">{{ seg.content }}</span>
      <MathRenderer v-else-if="seg.type === 'inline-math'" :latex="seg.latex" :displayMode="false" />
      <MathRenderer v-else-if="seg.type === 'block-math'" :latex="seg.latex" :displayMode="true" />
    </template>
  </div>
</template>

<style scoped>
.mixed-content {
  line-height: 1.7;
  word-break: break-word;
}

.text-segment {
  white-space: pre-wrap;
}
</style>
