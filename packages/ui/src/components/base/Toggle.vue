<template>
  <input
    :id="id"
    type="checkbox"
    class="switch stylized-toggle"
    :disabled="disabled"
    :checked="checked"
    @change="checked = !checked"
  />
</template>

<script setup lang="ts">
defineProps<{
  id?: string
  disabled?: boolean
}>()

const checked = defineModel<boolean>()
</script>
