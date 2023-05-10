<script setup>
import { shallowRef } from 'vue'
import GridDisplay from '@/components/GridDisplay.vue'
import { list } from '@/helpers/profile.js'
import { useRoute } from 'vue-router'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { loading_listener } from '@/helpers/events.js'

const route = useRoute()
const breadcrumbs = useBreadcrumbs()

breadcrumbs.setRootContext({ name: 'Library', link: route.path })

const profiles = await list()
const instances = shallowRef(
  Object.values(profiles).filter((prof) => !prof.metadata.linked_project_id)
)
const modpacks = shallowRef(
  Object.values(profiles).filter((prof) => prof.metadata.linked_project_id)
)

loading_listener(async (profile) => {
  console.log(profile)
  if (profile.event === 'loaded') {
    const profiles = await list()
    instances.value = Object.values(profiles).filter((prof) => !prof.metadata.linked_project_id)
    modpacks.value = Object.values(profiles).filter((prof) => prof.metadata.linked_project_id)
  }
})
</script>

<template>
  <div>
    <GridDisplay label="Instances" :instances="instances" />
    <GridDisplay label="Modpacks" :instances="modpacks" />
  </div>
</template>

<style lang="scss" scoped></style>
