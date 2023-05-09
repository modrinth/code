<script setup>
import GridDisplay from '@/components/GridDisplay.vue'
import { shallowRef } from 'vue'
import { list } from '@/helpers/profile'
import { useRoute } from 'vue-router'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { loading_listener } from '@/helpers/events.js'
import { progress_bars_list } from '@/helpers/state.js'

const route = useRoute()
const breadcrumbs = useBreadcrumbs()

const instances = shallowRef(Object.values(await list()))
const loadingInstances = shallowRef(Object.values(await progress_bars_list()))

breadcrumbs.setRootContext({ name: 'Library', link: route.path })
loading_listener(async (profile) => {
  console.log(profile)
  instances.value = Object.values(await list())
  loadingInstances.value = Object.values(await progress_bars_list())
})
</script>

<template>
  <div>
    <GridDisplay label="Instances" :instances="instances" />
    <GridDisplay label="Modpacks" :instances="instances" />
  </div>
</template>
