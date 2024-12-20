<script setup>
import { ref, onUnmounted, computed } from 'vue'
import { useRoute } from 'vue-router'
import RowDisplay from '@/components/RowDisplay.vue'
import { list } from '@/helpers/profile.js'
import { profile_listener } from '@/helpers/events'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { handleError } from '@/store/notifications.js'
import dayjs from 'dayjs'
import { get_search_results } from '@/helpers/cache.js'

const featuredModpacks = ref({})
const featuredMods = ref({})
const filter = ref('')

const route = useRoute()
const breadcrumbs = useBreadcrumbs()

breadcrumbs.setRootContext({ name: 'Home', link: route.path })

const recentInstances = ref([])

const offline = ref(!navigator.onLine)
window.addEventListener('offline', () => {
  offline.value = true
})
window.addEventListener('online', () => {
  offline.value = false
})

const getInstances = async () => {
  const profiles = await list().catch(handleError)

  recentInstances.value = profiles
    .filter((x) => x.last_played)
    .sort((a, b) => {
      const dateA = dayjs(a.last_played)
      const dateB = dayjs(b.last_played)

      if (dateA.isSame(dateB)) {
        return a.name.localeCompare(b.name)
      }

      return dateB - dateA
    })

  const filters = []
  for (const instance of profiles) {
    if (instance.linked_data && instance.linked_data.project_id) {
      filters.push(`NOT"project_id"="${instance.linked_data.project_id}"`)
    }
  }
  filter.value = filters.join(' AND ')
}

const getFeaturedModpacks = async () => {
  const response = await get_search_results(
    `?facets=[["project_type:modpack"]]&limit=10&index=follows&filters=${filter.value}`,
  )

  if (response) {
    featuredModpacks.value = response.result.hits
  } else {
    featuredModpacks.value = []
  }
}
const getFeaturedMods = async () => {
  const response = await get_search_results('?facets=[["project_type:mod"]]&limit=10&index=follows')

  if (response) {
    featuredMods.value = response.result.hits
  } else {
    featuredModpacks.value = []
  }
}

await getInstances()

await Promise.all([getFeaturedModpacks(), getFeaturedMods()])

const unlistenProfile = await profile_listener(async (e) => {
  await getInstances()

  if (e.event === 'added' || e.event === 'created' || e.event === 'removed') {
    await Promise.all([getFeaturedModpacks(), getFeaturedMods()])
  }
})

// computed sums of recentInstances, featuredModpacks, featuredMods, treating them as arrays if they are not
const total = computed(() => {
  return (
    (recentInstances.value?.length ?? 0) +
    (featuredModpacks.value?.length ?? 0) +
    (featuredMods.value?.length ?? 0)
  )
})

onUnmounted(() => {
  unlistenProfile()
})
</script>

<template>
  <div class="p-6 flex flex-col gap-2">
    <h1 v-if="recentInstances" class="m-0 text-2xl">Welcome back!</h1>
    <h1 v-else class="m-0 text-2xl">Welcome to Modrinth App!</h1>
    <RowDisplay
      v-if="total > 0"
      :instances="[
        {
          label: 'Recently played',
          route: '/library',
          instances: recentInstances,
          instance: true,
          downloaded: true,
          compact: true,
        },
        {
          label: 'Discover a modpack',
          route: '/browse/modpack',
          instances: featuredModpacks,
          downloaded: false,
        },
        {
          label: 'Discover mods',
          route: '/browse/mod',
          instances: featuredMods,
          downloaded: false,
        },
      ]"
      :can-paginate="true"
    />
  </div>
</template>
