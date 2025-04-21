<script setup>
import { ref, onUnmounted, computed, nextTick, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import RowDisplay from '@/components/RowDisplay.vue'
import { kill, list } from '@/helpers/profile.js'
import { process_listener, profile_listener } from '@/helpers/events'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { handleError } from '@/store/notifications.js'
import dayjs from 'dayjs'
import { get_search_results } from '@/helpers/cache.js'
import { useTheming } from '@/store/state.js'
import { HeadingLink } from '@modrinth/ui'
import { GAME_MODES } from '@/composables/worlds.ts'
import WorldItem from '@/components/ui/world/WorldItem.vue'
import InstanceItem from '@/components/ui/world/InstanceItem.vue'
import { get_recent_worlds, getWorldIdentifier } from '@/helpers/worlds.ts'
import { get_all } from '@/helpers/process.js'
import { trackEvent } from '@/helpers/analytics.js'

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

const jumpBackInItems = ref([])
const serverMetadata = ref({})

const MIN_JUMP_BACK_IN = 3
const MAX_JUMP_BACK_IN = 6
const TWO_WEEKS_AGO = dayjs().subtract(14, 'day')

populateJumpBackIn()

function populateJumpBackIn() {
  nextTick().then(async () => {
    const worlds = await get_recent_worlds(MAX_JUMP_BACK_IN);

    const worldItems = worlds.map((world) => ({
      type: 'world',
      last_played: world.last_played ? dayjs(world.last_played) : undefined,
      world: world,
      instance: recentInstances.value.find((instance) => instance.path === world.profile),
    }));

    const instanceItems = recentInstances.value.map((instance) => ({
      type: 'instance',
      last_played: instance.last_played ? dayjs(instance.last_played) : undefined,
      instance: instance,
    }));

    const items = [...worldItems, ...instanceItems];
    items.sort((a, b) => dayjs(b.last_played).diff(dayjs(a.last_played)));
    jumpBackInItems.value = items.filter((item, index) => index < MIN_JUMP_BACK_IN || item.last_played.isAfter(TWO_WEEKS_AGO));
  })
}

const unlistenProcesses = await process_listener(async (e) => {
  await checkProcesses()
})

onUnmounted(() => {
  unlistenProcesses()
})

const currentProfile = ref()
const currentWorld = ref()

const runningInstances = ref([])

const checkProcesses = async () => {
  const runningProcesses = await get_all().catch(handleError)

  const runningPaths = runningProcesses.map((x) => x.profile_path)

  const stoppedInstances = runningInstances.value.filter((x) => !runningPaths.includes(x))
  if (stoppedInstances.includes(currentProfile.value)) {
    currentProfile.value = null
    currentWorld.value = null
  }

  runningInstances.value = runningPaths
}

const stopInstance = async (path) => {
  await kill(path).catch(handleError)
  trackEvent('InstanceStop', {
    source: 'HomePage',
  })
}

onMounted(() => {
  checkProcesses()
})

onUnmounted(() => {
  unlistenProfile()
  unlistenProcesses()
})
</script>

<template>
  <div class="p-6 flex flex-col gap-2">
    <h1 v-if="recentInstances" class="m-0 text-2xl">Welcome back!</h1>
    <h1 v-else class="m-0 text-2xl">Welcome to Modrinth App!</h1>
    <div v-if="jumpBackInItems.length > 0" class="flex flex-col gap-2">
      <HeadingLink v-if="theme.featureFlags['worlds_tab']" to="/worlds" class="mt-1">
        Jump back in
      </HeadingLink>
      <span
        v-else
        class="flex mt-1 mb-3 leading-none items-center gap-1 text-primary text-lg font-bold"
      >
        Jump back in
      </span>
      <div class="flex flex-col w-full gap-2">
        <template
          v-for="item in jumpBackInItems"
          :key="`${item.instance.path}-${item.type === 'world' ? getWorldIdentifier(item.world) : 'instance'}`"
        >
          <WorldItem
            v-if="item.type === 'world'"
            :world="item.world"
            :playing-instance="runningInstances.includes(item.instance.path)"
            :playing-world="
              currentProfile === item.instance.path &&
              currentWorld === getWorldIdentifier(item.world)
            "
            :refreshing="
              item.world.type === 'server'
                ? serverMetadata[item.instance.path].refreshing.includes(item.world.address)
                : undefined
            "
            supports-quick-play
            :server-status="
              item.world.type === 'server'
                ? serverMetadata[item.instance.path].status[item.world.address]
                : undefined
            "
            :rendered-motd="
              item.world.type === 'server'
                ? serverMetadata[item.instance.path].motd[item.world.address]
                : undefined
            "
            :current-protocol="item.protocolVersion"
            :game-mode="
              item.world.type === 'singleplayer' ? GAME_MODES[item.world.game_mode] : undefined
            "
            :instance-path="item.instance.path"
            :instance-name="item.instance.name"
            :instance-icon="item.instance.icon_path ?? undefined"
            @refresh="() => (item.world.type === 'server' ? item.refresh(item.world.address) : {})"
            @play="
              () => {
                currentProfile = item.instance.path
                currentWorld = getWorldIdentifier(item.world)
                item.play(item.world)
              }
            "
            @stop="() => stopInstance(item.instance.path)"
          />
          <InstanceItem v-else :instance="item.instance" />
        </template>
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
