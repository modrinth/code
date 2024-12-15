<script setup>
import GridDisplay from '@/components/GridDisplay.vue'

defineProps({
  instances: {
    type: Array,
    required: true,
  },
})
</script>
<template>
  <GridDisplay
    v-if="instances.length > 0"
    label="Instances"
    :instances="instances.filter((i) => i.linked_data)"
  />
</template>
