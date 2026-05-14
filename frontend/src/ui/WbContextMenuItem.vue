<script setup lang="ts">
import { ContextMenuItem } from 'reka-ui'

defineProps<{
  disabled?: boolean
}>()
</script>

<template>
  <ContextMenuItem class="wb-menu-item" :disabled="disabled">
    <slot />
  </ContextMenuItem>
</template>
