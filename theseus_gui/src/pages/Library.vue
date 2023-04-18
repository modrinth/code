<script setup>
import { shallowRef, onMounted, ref } from 'vue'
import { AnimatedLogo } from 'omorphia'
import GridDisplay from '@/components/GridDisplay.vue'
import { list } from '@/helpers/profile.js'

const loading = ref(false)
let instances
let modpacks

onMounted(async () => {
  loading.value = true
  const profiles = await list()
  instances = shallowRef(Object.values(profiles).filter((prof) => !prof.metadata.linked_project_id))
  modpacks = shallowRef(Object.values(profiles).filter((prof) => prof.metadata.linked_project_id))

  loading.value = false
})
</script>

<template>
  <AnimatedLogo v-if="loading" class="loader" />
  <div v-else>
    <GridDisplay label="Instances" :instances="instances" />
    <GridDisplay label="Modpacks" :instances="modpacks" />
  </div>
</template>

<style lang="scss" scoped>
.loader {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
}
</style>
