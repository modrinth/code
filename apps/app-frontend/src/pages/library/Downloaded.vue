<script setup lang="ts">
import GridDisplay from '@/components/GridDisplay.vue'
import type { Profile } from '@/helpers/lib-types'

defineProps<{
  instances: Profile[]
}>()
</script>
<template>
  <GridDisplay
    v-if="instances.length > 0"
    label="Instances"
    :instances="instances.filter((i) => i.linked_data)"
  />
</template>
