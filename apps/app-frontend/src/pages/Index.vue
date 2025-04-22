<script setup lang="ts">
import { ref, onUnmounted, computed } from 'vue'
import { useRoute } from 'vue-router'
import RowDisplay from '@/components/RowDisplay.vue'
import { list } from '@/helpers/profile.js'
import { profile_listener } from '@/helpers/events'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { handleError } from '@/store/notifications.js'
import dayjs from 'dayjs'
import { get_search_results } from '@/helpers/cache.js'
import type { SearchResult } from '@modrinth/utils'
import RecentWorldsList from '@/components/ui/world/RecentWorldsList.vue'

const route = useRoute()
const breadcrumbs = useBreadcrumbs()

breadcrumbs.setRootContext({ name: 'Home', link: route.path })

const instances = ref<GameInstance[]>([])

const featuredModpacks = ref<SearchResult[]>([])
const featuredMods = ref<SearchResult[]>([])
const installedModpacksFilter = ref('')

const recentInstances = computed(() =>
  instances.value
    .filter((x) => x.last_played)
    .slice()
    .sort((a, b) => dayjs(b.last_played).diff(dayjs(a.last_played))),
)

const hasFeaturedProjects = computed(
  () => (featuredModpacks.value?.length ?? 0) + (featuredMods.value?.length ?? 0) > 0,
)

const offline = ref<boolean>(!navigator.onLine)
window.addEventListener('offline', () => {
  offline.value = true
})
window.addEventListener('online', () => {
  offline.value = false
})

async function fetchInstances() {
  instances.value = await list().catch(handleError)

  const filters = []
  for (const instance of instances.value) {
    if (instance.linked_data && instance.linked_data.project_id) {
      filters.push(`NOT"project_id"="${instance.linked_data.project_id}"`)
    }
  }
  installedModpacksFilter.value = filters.join(' AND ')
}

async function fetchFeaturedModpacks() {
  const response = await get_search_results(
    `?facets=[["project_type:modpack"]]&limit=10&index=follows&filters=${installedModpacksFilter.value}`,
  )

  if (response) {
    featuredModpacks.value = response.result.hits
  } else {
    featuredModpacks.value = []
  }
}

async function fetchFeaturedMods() {
  const response = await get_search_results('?facets=[["project_type:mod"]]&limit=10&index=follows')

  if (response) {
    featuredMods.value = response.result.hits
  } else {
    featuredModpacks.value = []
  }
}

async function refreshFeaturedProjects() {
  await Promise.all([fetchFeaturedModpacks(), fetchFeaturedMods()])
}

await fetchInstances()
await refreshFeaturedProjects()

const unlistenProfile = await profile_listener(async (e) => {
  await fetchInstances()

  if (e.event === 'added' || e.event === 'created' || e.event === 'removed') {
    await refreshFeaturedProjects()
  }
})

onUnmounted(() => {
  unlistenProfile()
})
</script>

<template>
  <div class="p-6 flex flex-col gap-2">
    <h1 v-if="recentInstances" class="m-0 text-2xl">Welcome back!</h1>
    <h1 v-else class="m-0 text-2xl">Welcome to Modrinth App!</h1>
    <RecentWorldsList :recent-instances="recentInstances" />
    <RowDisplay
      v-if="hasFeaturedProjects"
      :instances="[
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
