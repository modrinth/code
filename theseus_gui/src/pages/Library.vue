<script setup>
import { shallowRef } from 'vue'
import GridDisplay from '@/components/GridDisplay.vue'
import { list } from '@/helpers/profile.js'

const profiles = await list()
const instances = shallowRef(
  Object.values(profiles).filter((prof) => !prof.metadata.linked_project_id)
)
const modpacks = shallowRef(
  Object.values(profiles).filter((prof) => prof.metadata.linked_project_id)
)
</script>

<template>
  <div>
    <GridDisplay label="Instances" :instances="instances" />
    <GridDisplay label="Modpacks" :instances="modpacks" />
  </div>
</template>

<style lang="scss" scoped></style>
