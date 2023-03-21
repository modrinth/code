<script setup>
import { useInstances } from '@/store/state'
import GridDisplay from '@/components/GridDisplay.vue'

const instances = useInstances()
instances.fetchInstances()
</script>

<template>
  <div>
    <GridDisplay label="Instances" :instances="instances.instances" />
    <GridDisplay label="Instances" :instances="instances.instances" />
  </div>
</template>
