<script setup lang="ts">
withDefaults(
  defineProps<{
    active?: boolean
    size?: 'default' | 'compact'
    type?: 'button' | 'submit' | 'reset'
  }>(),
  {
    active: false,
    size: 'default',
    type: 'button',
  },
)
</script>

<template>
  <button class="wb-icon-button" :data-active="active" :data-size="size" :type="type">
    <slot />
  </button>
</template>
