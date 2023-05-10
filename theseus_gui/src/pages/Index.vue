<script setup>
import { ref, shallowRef, onUnmounted } from 'vue'
import { ofetch } from 'ofetch'
import { useRoute } from 'vue-router'
import RowDisplay from '@/components/RowDisplay.vue'
import { list } from '@/helpers/profile.js'
import { profile_listener } from '@/helpers/events'
import { useBreadcrumbs, useNotifications } from '@/store/state'

const featuredModpacks = ref({})
const featuredMods = ref({})
const filter = ref('')

const route = useRoute()
const breadcrumbs = useBreadcrumbs()
const notificationStore = useNotifications()

breadcrumbs.setRootContext({ name: 'Home', link: route.path })

const recentInstances = shallowRef()

const getInstances = async () => {
  filter.value = ''

  try {
    const profiles = await list()
    recentInstances.value = Object.values(profiles)

    const excludeIds = recentInstances.value.map((i) => i.metadata?.linked_data?.project_id)
    excludeIds.forEach((id, index) => {
      filter.value += `NOT"project_id"="${id}"`
      if (index < excludeIds.length - 1) filter.value += ' AND '
    })
  } catch (err) {
    notificationStore.addTauriErrorNotif(err)
  }
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

await Promise.all([getFeaturedModpacks(), getFeaturedMods()]).catch((err) =>
  notificationStore.addApiErrorNotif(err)
)

const unlisten = await profile_listener(async (e) => {
  if (e.event === 'edited') {
    await getInstances()
    await Promise.all([getFeaturedModpacks(), getFeaturedMods()]).catch((err) =>
      notificationStore.addApiErrorNotif(err)
    )
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
