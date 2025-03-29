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
import { useTheming } from '@/store/state.js'
import { HeadingLink } from '@modrinth/ui'
import { GAME_MODES, useWorldsMultiInstance } from '@/composables/worlds.ts'
import WorldItem from '@/components/ui/world/WorldItem.vue'

const featuredModpacks = ref({})
const featuredMods = ref({})
const filter = ref('')

const route = useRoute()
const theme = useTheming()
const breadcrumbs = useBreadcrumbs()

breadcrumbs.setRootContext({ name: 'Home', link: route.path })

const recentInstances = ref([])
const instances = ref([])

const offline = ref(!navigator.onLine)
window.addEventListener('offline', () => {
  offline.value = true
})
window.addEventListener('online', () => {
  offline.value = false
})

const getInstances = async () => {
  instances.value = await list().catch(handleError)

  recentInstances.value = instances.value
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
  for (const instance of instances.value) {
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

const TWO_WEEKS_AGO = dayjs().subtract(14, 'day')

function worldCondition(world) {
  return world.last_played && dayjs(world.last_played).isAfter(TWO_WEEKS_AGO)
}

const worldInstances = computed(() => {
  return instances.value
    .filter((instance) => instance.last_played)
    .slice()
    .sort((a, b) => dayjs(b.last_played).diff(dayjs(a.last_played)))
})

const { worlds, serverMetadata } = await useWorldsMultiInstance(
  worldInstances,
  () => {},
  worldCondition,
)

onUnmounted(() => {
  unlistenProfile()
})
</script>

<template>
  <div class="p-6 flex flex-col gap-2">
    <h1 v-if="recentInstances" class="m-0 text-2xl">Welcome back!</h1>
    <h1 v-else class="m-0 text-2xl">Welcome to Modrinth App!</h1>
    <div class="flex flex-col gap-2">
      <HeadingLink v-if="theme.featureFlags['worlds_tab']" to="/worlds" class="mt-1">
        Jump back in
      </HeadingLink>
      <span
        v-else
        class="flex mt-1 mb-3 leading-none items-center gap-1 text-primary text-lg font-bold"
      >
        Jump back in
      </span>
      <div
        class="flex flex-col w-full supports-[grid-template-columns:subgrid]:grid supports-[grid-template-columns:subgrid]:grid-cols-[auto_minmax(0,3fr)_minmax(0,4fr)_auto] gap-2"
      >
        <WorldItem
          v-for="world in worlds.filter((x) => x.last_played)"
          :key="world.instancePath + (world.type === 'singleplayer' ? world.path : world.address)"
          :world="world"
          :refreshing="
            world.type === 'server'
              ? serverMetadata[world.instancePath].refreshing.includes(world.address)
              : undefined
          "
          :supports-quick-play="world.supportsQuickPlay"
          :server-status="
            world.type === 'server'
              ? serverMetadata[world.instancePath].status[world.address]
              : undefined
          "
          :rendered-motd="
            world.type === 'server'
              ? serverMetadata[world.instancePath].motd[world.address]
              : undefined
          "
          :game-mode="world.type === 'singleplayer' ? GAME_MODES[world.game_mode] : undefined"
          :instance-path="world.instancePath"
          :instance-name="world.instanceName"
          :instance-icon="world.instanceIcon"
          @refresh="() => (world.type === 'server' ? world.refresh(world.address) : {})"
          @play="() => world.play(world)"
        />
      </div>
    </div>
    <RowDisplay
      v-if="total > 0"
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
