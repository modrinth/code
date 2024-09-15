<template>
  <div
    v-if="isConnected && !isWSAuthIncorrect"
    data-pyro-server-manager-root
    class="flex flex-col gap-6"
  >
    <transition name="fade-slide">
      <ServerStats v-if="!fullScreen" :data="stats" />
    </transition>
    <div
      class="relative flex w-full flex-col gap-3 overflow-hidden rounded-2xl bg-bg-raised p-8 transition-[height] duration-500 ease-in-out"
      :style="consoleStyle"
    >
      <div class="experimental-styles-within flex flex-row items-center justify-between">
        <div class="flex flex-row items-center gap-4">
          <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">Console</h2>
          <PanelServerStatus :state="serverPowerState" />
        </div>
        <PanelServerActionButton
          :is-online="serverPowerState === 'running'"
          :is-actioning="isActioning"
          @action="sendPowerAction"
        />
      </div>

      <PanelTerminal
        :console-output="consoleOutput"
        :full-screen="fullScreen"
        @toggle-full-screen="toggleFullScreen"
      />
    </div>
  </div>
  <PanelOverviewLoading v-else-if="!isConnected && !isWSAuthIncorrect" />
  <PyroError
    v-else-if="isWSAuthIncorrect"
    title="WebSocket authentication failed"
    message="Indicative of a server misconfiguration. Please report this to support."
  />
  <PyroError
    v-else
    title="An error occurred"
    message="Something went wrong while attempting to connect to your server. Your data is safe, and we're working to resolve the issue."
  />
</template>
<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { useRoute } from 'vue-router'
import WebSocket from '@tauri-apps/plugin-websocket'
import { useServerStore } from '@/store/servers'
import type { Server, ServerState, Stats, WSAuth, WSEvent } from '@/types/servers'
import ServerStats from '@/components/ui/servers/ServerStats.vue'
import PanelServerStatus from '@/components/ui/servers/PanelServerStatus.vue'
import PanelServerActionButton from '@/components/ui/servers/PanelServerActionButton.vue'
import PanelTerminal from '@/components/ui/servers/PanelTerminal.vue'
import PanelOverviewLoading from '@/components/ui/servers/PanelOverviewLoading.vue'
import PyroError from '@/components/ui/servers/PyroError.vue'
import type { Credentials } from './Index.vue'

const serverStore = useServerStore()
const route = useRoute()

const fullScreen = ref(false)
const consoleStyle = ref({ height: '600px', marginTop: '0px' })
const isConnected = ref(false)
const isWSAuthIncorrect = ref(false)
const consoleOutput = ref<string[]>([])
const cpuData = ref<number[]>([])
const ramData = ref<number[]>([])
const isActioning = ref(false)
const serverPowerState = ref<ServerState>('stopped')

const props = defineProps<{
  server: Server
  credentials: Credentials
}>()

const stats = ref<Stats>({
  current: {
    cpu_percent: 0,
    ram_usage_bytes: 0,
    ram_total_bytes: 1,
    storage_usage_bytes: 0,
    storage_total_bytes: 0,
  },
  past: {
    cpu_percent: 0,
    ram_usage_bytes: 0,
    ram_total_bytes: 1,
    storage_usage_bytes: 0,
    storage_total_bytes: 0,
  },
  graph: {
    cpu: [],
    ram: [],
  },
})

const serverId = route.params.id as string

let socket: WebSocket | null = null

const session = ref(props.credentials.session)
const wsAuth = ref<WSAuth | null>(null)
const webSocketMessages = ref<string[]>([])

const toggleFullScreen = () => {
  fullScreen.value = !fullScreen.value
  if (fullScreen.value) {
    consoleStyle.value.height = '90vh'
    animateMarginTop()
  } else {
    consoleStyle.value.height = '600px'
    consoleStyle.value.marginTop = '0px'
  }
}

const animateMarginTop = () => {
  setTimeout(() => {
    let mt = 254
    const interval = setInterval(() => {
      mt -= 10
      consoleStyle.value.marginTop = `${mt}px`
      if (mt <= 0 || !fullScreen.value) clearInterval(interval)
    }, 10)
  }, 500)
}

const sendPowerAction = async (action: 'restart' | 'start' | 'stop' | 'kill') => {
  const actionName = action.charAt(0).toUpperCase() + action.slice(1)
  console.log(`${actionName}ing server`)

  try {
    isActioning.value = true
    await serverStore.sendPowerAction(serverId, actionName)
  } catch (error) {
    console.error(`Error ${actionName}ing server:`, error)
  } finally {
    isActioning.value = false
  }
}

const connectWebSocket = async () => {
  console.log(session.value)
  wsAuth.value = (await serverStore.requestWebsocket(session.value, serverId)) as WSAuth
  console.log(wsAuth.value)
  socket = await WebSocket.connect(`wss://${wsAuth.value.url}`)

  socket.addListener((msg) => handleWebSocketMessage(msg as unknown as WSEvent))

  await socket.send(JSON.stringify({ event: 'auth', jwt: wsAuth.value.token }))
}
const handleWebSocketMessage = (msg: WSEvent) => {
  try {
    const data = typeof msg === 'string' ? JSON.parse(msg) : msg
    console.log('Received WebSocket message:', data)

    const parsedData = JSON.parse(data.data)

    webSocketMessages.value.push(JSON.stringify(parsedData))

    switch (parsedData.event) {
      case 'log':
        consoleOutput.value.push(parsedData.message)
        break
      case 'stats':
        updateStats(parsedData as unknown as Stats['current'])
        break
      case 'auth-expiring':
        reauth()
        break
      case 'power-state':
        updatePowerState(parsedData.state)
        break
      case 'auth-incorrect':
        isWSAuthIncorrect.value = true
        break
    }
  } catch (error) {
    console.error('Failed to parse WebSocket message:', msg, error)
  }
}

const updatePowerState = (state: ServerState) => {
  serverPowerState.value = state
}

const updateStats = (data: Stats['current']) => {
  isConnected.value = true
  stats.value = {
    current: data,
    past: stats.value.current,
    graph: {
      cpu: updateDataArray(cpuData.value, Math.round(data.cpu_percent * 100) / 100),
      ram: updateDataArray(
        ramData.value,
        Math.floor((data.ram_usage_bytes / data.ram_total_bytes) * 100),
      ),
    },
  }
}

const updateDataArray = (arr: number[], newValue: number) => {
  arr.push(newValue)
  if (arr.length > 10) arr.shift()
  return [...arr]
}

const reauth = async () => {
  wsAuth.value = (await serverStore.requestWebsocket(props.credentials.session, serverId)) as WSAuth
  await socket?.send(JSON.stringify({ event: 'auth', jwt: wsAuth.value.token }))
}

onMounted(connectWebSocket)
onBeforeUnmount(async () => {
  if (socket) {
    await socket.disconnect()
  }
})
</script>

<style scoped>
.fade-slide-enter-active,
.fade-slide-leave-active {
  transition:
    opacity 0.5s ease,
    transform 0.5s ease;
}
.fade-slide-enter-from,
.fade-slide-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}

.console {
  transition:
    height 0.5s ease,
    margin-top 0.5s ease;
}
</style>
