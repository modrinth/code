<script setup lang="ts">
import {
  type WorldWithProfile,
  get_recent_worlds,
  getWorldIdentifier,
  get_profile_protocol_version,
  refreshServerData,
  start_join_server,
  start_join_singleplayer_world,
} from '@/helpers/worlds.ts'
import { HeadingLink, GAME_MODES } from '@modrinth/ui'
import WorldItem from '@/components/ui/world/WorldItem.vue'
import InstanceItem from '@/components/ui/world/InstanceItem.vue'
import { watch, onMounted, onUnmounted, ref } from 'vue'
import dayjs from 'dayjs'
import { useTheming } from '@/store/theme'
import { kill } from '@/helpers/profile'
import { handleError } from '@/store/notifications'
import { trackEvent } from '@/helpers/analytics'
import { process_listener, profile_listener } from '@/helpers/events'
import { get_all } from '@/helpers/process'

const props = defineProps<{
  recentInstances: GameInstance[]
}>()

const theme = useTheming()

const jumpBackInItems = ref([])
const serverData = ref({})
const protocolVersions = ref<Record<string, number | null>>({})

const MIN_JUMP_BACK_IN = 3
const MAX_JUMP_BACK_IN = 6
const TWO_WEEKS_AGO = dayjs().subtract(14, 'day')

watch(props.recentInstances, async () => {
  await populateJumpBackIn().catch(() => {
    console.error('Failed to populate jump back in')
  })
})

await populateJumpBackIn().catch(() => {
  console.error('Failed to populate jump back in')
})

async function populateJumpBackIn() {
  console.info('Repopulating jump back in...')
  const worlds = await get_recent_worlds(MAX_JUMP_BACK_IN)

  const worldItems = worlds.map((world) => ({
    type: 'world',
    last_played: world.last_played ? dayjs(world.last_played) : undefined,
    world: world,
    instance: props.recentInstances.find((instance) => instance.path === world.profile),
  }))

  const servers: {
    instancePath: string
    address: string
  }[] = worldItems
    .filter((item) => item.world.type === 'server' && item.instance)
    .map((item) => ({
      instancePath: item.instance.path,
      address: item.world.address,
    }))

  // fetch protocol versions for all unique MC versions with server worlds
  const uniqueServerInstances = new Set<string>(servers.map((x) => x.instancePath))
  await Promise.all(
    [...uniqueServerInstances].map((path) => {
      get_profile_protocol_version(path)
        .then((protoVer) => (protocolVersions.value[path] = protoVer))
        .catch(() => {
          console.error(`Failed to get profile protocol for: ${path} `)
        })
    }),
  )

  // initialize server data
  servers.forEach(({ address }) => {
    if (!serverData.value[address]) {
      serverData.value[address] = {
        refreshing: true,
      }
    }
  })

  // fetch each server's data
  await Promise.all(
    servers.map(({ game_version, address }) =>
      refreshServerData(serverData.value[address], protocolVersions.value[game_version], address),
    ),
  )

  const instanceItems = props.recentInstances
    .filter((instance) => !worldItems.some((item) => item.instance.path === instance.path))
    .map((instance) => ({
      type: 'instance',
      last_played: instance.last_played ? dayjs(instance.last_played) : undefined,
      instance: instance,
    }))

  const items = [...worldItems, ...instanceItems]
  items.sort((a, b) => dayjs(b.last_played).diff(dayjs(a.last_played)))
  jumpBackInItems.value = items.filter(
    (item, index) => index < MIN_JUMP_BACK_IN || item.last_played.isAfter(TWO_WEEKS_AGO),
  )
}

async function joinWorld(world: WorldWithProfile) {
  console.log(`Joining world ${getWorldIdentifier(world)}`)
  if (world.type === 'server') {
    await start_join_server(world.profile, world.address).catch(handleError)
  } else if (world.type === 'singleplayer') {
    await start_join_singleplayer_world(world.profile, world.path).catch(handleError)
  }
}

const stopInstance = async (path) => {
  await kill(path).catch(handleError)
  trackEvent('InstanceStop', {
    source: 'RecentWorldsList',
  })
}

const currentProfile = ref()
const currentWorld = ref()

const unlistenProcesses = await process_listener(async () => {
  await checkProcesses()
})

const unlistenProfiles = await profile_listener(async () => {
  await populateJumpBackIn().catch(() => {
    console.error('Failed to populate jump back in')
  })
})

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

onMounted(() => {
  checkProcesses()
})

onUnmounted(() => {
  unlistenProcesses()
  unlistenProfiles()
})
</script>

<template>
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
            currentProfile === item.instance.path && currentWorld === getWorldIdentifier(item.world)
          "
          :refreshing="
            item.world.type === 'server'
              ? serverData[item.world.address].refreshing && !serverData[item.world.address].status
              : undefined
          "
          supports-quick-play
          :server-status="
            item.world.type === 'server' ? serverData[item.world.address].status : undefined
          "
          :rendered-motd="
            item.world.type === 'server' ? serverData[item.world.address].renderedMotd : undefined
          "
          :current-protocol="protocolVersions[item.instance.game_version]"
          :game-mode="
            item.world.type === 'singleplayer' ? GAME_MODES[item.world.game_mode] : undefined
          "
          :instance-path="item.instance.path"
          :instance-name="item.instance.name"
          :instance-icon="item.instance.icon_path"
          @refresh="() => (item.world.type === 'server' ? item.refresh(item.world.address) : {})"
          @play="
            () => {
              currentProfile = item.instance.path
              currentWorld = getWorldIdentifier(item.world)
              joinWorld(item.world)
            }
          "
          @stop="() => stopInstance(item.instance.path)"
        />
        <InstanceItem v-else :instance="item.instance" />
      </template>
    </div>
  </div>
</template>
