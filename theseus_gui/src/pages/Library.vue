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
const instances = shallowRef(Object.values(profiles))

loading_listener(async (profile) => {
  if (profile.event === 'loaded') {
    const profiles = await list()
    instances.value = Object.values(profiles)
  }
})
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
