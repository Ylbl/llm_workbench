<script setup lang="ts">
withDefaults(
  defineProps<{
    variant?: 'primary' | 'secondary' | 'danger' | 'ghost'
    size?: 'default' | 'compact'
    type?: 'button' | 'submit' | 'reset'
  }>(),
  {
    variant: 'secondary',
    size: 'default',
    type: 'button',
  },
)
</script>

<template>
  <button class="wb-button" :data-variant="variant" :data-size="size" :type="type">
    <slot />
  </button>
</template>
