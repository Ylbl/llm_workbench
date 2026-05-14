<script setup lang="ts">
import { DropdownMenuItem } from 'reka-ui'

defineProps<{
  disabled?: boolean
}>()
</script>

<template>
  <DropdownMenuItem class="wb-menu-item" :disabled="disabled">
    <slot />
  </DropdownMenuItem>
</template>
