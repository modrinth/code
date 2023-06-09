<script setup lang="ts">
import { ref, onUnmounted, shallowRef } from 'vue'
import { useRoute } from 'vue-router'
import RowDisplay from '@/components/RowDisplay.vue'
import { list } from '@/helpers/profile'
import { profile_listener } from '@/helpers/events'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { useFetch } from '@/helpers/fetch'
import { handleError } from '@/store/notifications'
import dayjs from 'dayjs'
import { getBaseUrl } from '@/helpers/utils'

const featuredModpacks = ref({})
const featuredMods = ref({})
const filter = ref('')

const route = useRoute()
const breadcrumbs = useBreadcrumbs()

breadcrumbs.setRootContext({ name: 'Home', link: route.path })

const recentInstances = shallowRef([])

const getInstances = async () => {
  const profiles = await list(true).catch(handleError)
  recentInstances.value = Object.values(profiles).sort((a, b) => {
    return dayjs(b.metadata.last_played ?? 0).diff(dayjs(a.metadata.last_played ?? 0))
  })

  const filters = []
  for (const instance of recentInstances.value) {
    if (instance.metadata.linked_data && instance.metadata.linked_data.project_id) {
      filters.push(`NOT"project_id"="${instance.metadata.linked_data.project_id}"`)
    }
  }
  filter.value = filters.join(' AND ')
}

const getFeaturedModpacks = async () => {
  console.log(filter.value)
  const response = await useFetch(
    `${getBaseUrl()}/v2/search?facets=[["project_type:modpack"]]&limit=10&index=follows&filters=${filter.value}`,
    'featured modpacks'
  )
  featuredModpacks.value = response.hits
}
const getFeaturedMods = async () => {
  const response = await useFetch(
    `${getBaseUrl()}/v2/search?facets=[["project_type:mod"]]&limit=10&index=follows`,
    'featured mods'
  )
  featuredMods.value = response.hits
}

await getInstances()
await Promise.all([getFeaturedModpacks(), getFeaturedMods()])

const unlisten = await profile_listener(async (e) => {
  await getInstances()
  if (e.event === 'created' || e.event === 'removed') {
    await Promise.all([getFeaturedModpacks(), getFeaturedMods()])
  }
})

onUnmounted(() => unlisten())
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
