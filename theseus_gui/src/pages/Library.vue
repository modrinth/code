<script setup>
import { onUnmounted, shallowRef } from 'vue'
import GridDisplay from '@/components/GridDisplay.vue'
import { list } from '@/helpers/profile.js'
import { useRoute } from 'vue-router'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { profile_listener } from '@/helpers/events.js'
import { handleError } from '@/store/notifications.js'

const route = useRoute()
const breadcrumbs = useBreadcrumbs()

breadcrumbs.setRootContext({ name: 'Library', link: route.path })

const profiles = await list(true).catch(handleError)
const instances = shallowRef(Object.values(profiles))

const unlisten = await profile_listener(async () => {
  const profiles = await list(true).catch(handleError)
  instances.value = Object.values(profiles)
})
onUnmounted(() => unlisten())
</script>

<template>
  <GridDisplay
    v-if="instances.length > 0"
    label="Instances"
    :instances="instances"
    class="display"
  />
</template>

<style lang="scss" scoped>
.display {
  background-color: rgb(30, 31, 34);
  min-height: 100%;
}
</style>
