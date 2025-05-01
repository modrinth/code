<script setup lang="ts">
import {
  type ServerWorld,
  type ServerData,
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
import { watch, onMounted, onUnmounted, ref, computed } from 'vue'
import type { Dayjs } from 'dayjs'
import dayjs from 'dayjs'
import { useTheming } from '@/store/theme.ts'
import { kill, run } from '@/helpers/profile'
import { handleError } from '@/store/notifications'
import { trackEvent } from '@/helpers/analytics'
import { process_listener, profile_listener } from '@/helpers/events'
import { get_all } from '@/helpers/process'
import type { GameInstance } from '@/helpers/types'
import { handleSevereError } from '@/store/error'

const props = defineProps<{
  recentInstances: GameInstance[]
}>()

const theme = useTheming()

const jumpBackInItems = ref<JumpBackInItem[]>([])
const serverData = ref<Record<string, ServerData>>({})
const protocolVersions = ref<Record<string, number | null>>({})

const MIN_JUMP_BACK_IN = 3
const MAX_JUMP_BACK_IN = 6
const TWO_WEEKS_AGO = dayjs().subtract(14, 'day')

type BaseJumpBackInItem = {
  last_played: Dayjs
  instance: GameInstance
}

type InstanceJumpBackInItem = BaseJumpBackInItem & {
  type: 'instance'
}

type WorldJumpBackInItem = BaseJumpBackInItem & {
  type: 'world'
  world: WorldWithProfile
}

type JumpBackInItem = InstanceJumpBackInItem | WorldJumpBackInItem

const showWorlds = computed(() => theme.getFeatureFlag('worlds_in_home'))

watch([() => props.recentInstances, () => showWorlds.value], async () => {
  await populateJumpBackIn().catch(() => {
    console.error('Failed to populate jump back in')
  })
})

await populateJumpBackIn().catch(() => {
  console.error('Failed to populate jump back in')
})

async function populateJumpBackIn() {
  console.info('Repopulating jump back in...')

  const worldItems: WorldJumpBackInItem[] = []

  if (showWorlds.value) {
    const worlds = await get_recent_worlds(MAX_JUMP_BACK_IN, ['normal', 'favorite'])

    worlds.forEach((world) => {
      const instance = props.recentInstances.find((instance) => instance.path === world.profile)

      if (!instance || !world.last_played) {
        return
      }

      worldItems.push({
        type: 'world',
        last_played: dayjs(world.last_played),
        world: world,
        instance: instance,
      })
    })

    const servers: {
      instancePath: string
      address: string
    }[] = worldItems
      .filter((item) => item.world.type === 'server' && item.instance)
      .map((item) => ({
        instancePath: item.instance.path,
        address: (item.world as ServerWorld).address,
      }))

    // fetch protocol versions for all unique MC versions with server worlds
    const uniqueServerInstances = new Set<string>(servers.map((x) => x.instancePath))
    await Promise.all(
      [...uniqueServerInstances].map((path) =>
        get_profile_protocol_version(path)
          .then((protoVer) => (protocolVersions.value[path] = protoVer))
          .catch(() => {
            console.error(`Failed to get profile protocol for: ${path} `)
          }),
      ),
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
    Promise.all(
      servers.map(({ instancePath, address }) =>
        refreshServerData(serverData.value[address], protocolVersions.value[instancePath], address),
      ),
    )
  }

  const instanceItems: InstanceJumpBackInItem[] = []
  for (const instance of props.recentInstances) {
    const worldItem = worldItems.find((item) => item.instance.path === instance.path)
    if ((worldItem && worldItem.last_played.isAfter(TWO_WEEKS_AGO)) || !instance.last_played) {
      continue
    }

    instanceItems.push({
      type: 'instance',
      last_played: dayjs(instance.last_played),
      instance: instance,
    })
  }

  const items: JumpBackInItem[] = [...worldItems, ...instanceItems]
  items.sort((a, b) => dayjs(b.last_played).diff(dayjs(a.last_played)))
  jumpBackInItems.value = items
    .filter((item, index) => index < MIN_JUMP_BACK_IN || item.last_played.isAfter(TWO_WEEKS_AGO))
    .slice(0, MAX_JUMP_BACK_IN)
}

async function refreshServer(address: string, instancePath: string) {
  await refreshServerData(serverData.value[address], protocolVersions.value[instancePath], address)
}

async function joinWorld(world: WorldWithProfile) {
  console.log(`Joining world ${getWorldIdentifier(world)}`)
  if (world.type === 'server') {
    await start_join_server(world.profile, world.address).catch(handleError)
  } else if (world.type === 'singleplayer') {
    await start_join_singleplayer_world(world.profile, world.path).catch(handleError)
  }
}

async function playInstance(instance: GameInstance) {
  await run(instance.path)
    .catch((err) => handleSevereError(err, { profilePath: instance.path }))
    .finally(() => {
      trackEvent('InstancePlay', {
        loader: instance.loader,
        game_version: instance.game_version,
        source: 'WorldItem',
      })
    })
}

async function stopInstance(path: string) {
  await kill(path).catch(handleError)
  trackEvent('InstanceStop', {
    source: 'RecentWorldsList',
  })
}

const currentProfile = ref<string>()
const currentWorld = ref<string>()

const unlistenProcesses = await process_listener(async () => {
  await checkProcesses()
})

const unlistenProfiles = await profile_listener(async () => {
  await populateJumpBackIn().catch(() => {
    console.error('Failed to populate jump back in')
  })
})

const runningInstances = ref<string[]>([])

type ProcessMetadata = {
  uuid: string
  profile_path: string
  start_time: string
}

const checkProcesses = async () => {
  const runningProcesses: ProcessMetadata[] = await get_all().catch(handleError)

  const runningPaths = runningProcesses.map((x) => x.profile_path)

  const stoppedInstances = runningInstances.value.filter((x) => !runningPaths.includes(x))
  if (currentProfile.value && stoppedInstances.includes(currentProfile.value)) {
    currentProfile.value = undefined
    currentWorld.value = undefined
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
    <HeadingLink v-if="theme.getFeatureFlag('worlds_tab')" to="/worlds" class="mt-1">
      Jump back in
    </HeadingLink>
    <span
      v-else
      class="flex mt-1 mb-3 leading-none items-center gap-1 text-primary text-lg font-bold"
    >
      Jump back in
    </span>
    <div class="grid-when-huge flex flex-col w-full gap-2">
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
          :current-protocol="protocolVersions[item.instance.path]"
          :game-mode="
            item.world.type === 'singleplayer' ? GAME_MODES[item.world.game_mode] : undefined
          "
          :instance-path="item.instance.path"
          :instance-name="item.instance.name"
          :instance-icon="item.instance.icon_path"
          @refresh="
            () =>
              item.world.type === 'server'
                ? refreshServer(item.world.address, item.instance.path)
                : {}
          "
          @update="() => populateJumpBackIn()"
          @play="
            () => {
              currentProfile = item.instance.path
              currentWorld = getWorldIdentifier(item.world)
              joinWorld(item.world)
            }
          "
          @play-instance="
            () => {
              currentProfile = item.instance.path
              playInstance(item.instance)
            }
          "
          @stop="() => stopInstance(item.instance.path)"
        />
        <InstanceItem v-else :instance="item.instance" />
      </template>
    </div>
  </div>
</template>
<style scoped lang="scss">
.grid-when-huge {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(670px, 1fr));
}
</style>
