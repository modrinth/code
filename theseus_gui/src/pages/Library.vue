<script setup>
import { shallowRef } from 'vue'
import GridDisplay from '@/components/GridDisplay.vue'
import { list } from '@/helpers/profile.js'
import { useRoute } from 'vue-router'
import { useBreadcrumbs, useNotifications } from '@/store/state'

const route = useRoute()
const breadcrumbs = useBreadcrumbs()
const notificationStore = useNotifications()

breadcrumbs.setRootContext({ name: 'Library', link: route.path })

const profiles = await list().catch((err) => notificationStore.addTauriErrorNotif(err))
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
