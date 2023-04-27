<script setup>
import { ref, shallowRef } from 'vue'
import { ofetch } from 'ofetch'
import { useRoute } from 'vue-router'
import RowDisplay from '@/components/RowDisplay.vue'
import { list } from '@/helpers/profile.js'
import { loading_listener } from '@/helpers/events'
import { useBreadcrumbs } from '@/store/breadcrumbs'

const featuredModpacks = ref({})
const featuredMods = ref({})
const filter = ref('')

const route = useRoute()
const breadcrumbs = useBreadcrumbs()

breadcrumbs.setRootContext({ name: 'Home', link: route.path })

const recentInstances = shallowRef()

const getInstances = async () => {
  filter.value = ''
  const profiles = await list()
  recentInstances.value = Object.values(profiles)

  const excludeIds = recentInstances.value.map((i) => i.metadata.linked_project_id)
  excludeIds.forEach((id, index) => {
    filter.value += `NOT"project_id"="${id}"`
    if (index < excludeIds.length - 1) filter.value += ' AND '
  })
}

const getFeaturedModpacks = async () => {
  const response = await ofetch(
    `https://api.modrinth.com/v2/search?facets=[["project_type:modpack"]]&limit=10&index=follows&filters=${filter.value}`
  )
  featuredModpacks.value = response.hits
}
const getFeaturedMods = async () => {
  const response = await ofetch(
    `https://api.modrinth.com/v2/search?facets=[["project_type:mod"]]&limit=10&index=follows&filters=${filter.value}`
  )
  featuredMods.value = response.hits
}

await getInstances()
await Promise.all([getFeaturedModpacks(), getFeaturedMods()])

// If a modpack is finished installing, refresh our instances list, the featured modpacks & featured mods
await loading_listener(async (e) => {
  // Null check is a current bug. Events API sometimes send back down a NULL when the task completes
  if (
    e.message === 'Downloading modpack...' &&
    (e.fraction === 1 || e.fraction === null) &&
    e.event.PackDownload
  ) {
    await getInstances()
    await Promise.all([getFeaturedModpacks(), getFeaturedMods()])
  }
})
</script>

<template>
  <div class="page-container">
    <RowDisplay label="Jump back in" :instances="recentInstances" :can-paginate="false" />
    <RowDisplay label="Popular packs" :instances="featuredModpacks" :can-paginate="true" />
    <RowDisplay label="Popular mods" :instances="featuredMods" :can-paginate="true" />
  </div>
</template>

<style lang="scss" scoped>
.page-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 100%;
}
</style>
