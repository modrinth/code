<template>
  <div
    v-if="currentConnection?.connectionState === 'connected'"
    data-pyro-server-manager-root
    class="flex flex-col gap-6"
  >
    <transition name="fade-slide">
      <ServerStats v-if="!fullScreen" :data="currentConnection.stats" />
    </transition>
    <div
      class="relative flex w-full flex-col gap-3 overflow-hidden rounded-2xl bg-bg-raised p-8 transition-[height] duration-500 ease-in-out"
      :style="consoleStyle"
    >
      <div class="experimental-styles-within flex flex-row items-center justify-between">
        <div class="flex flex-row items-center gap-4">
          <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">Console</h2>
          <PanelServerStatus :state="currentConnection.serverPowerState" />
        </div>
        <PanelServerActionButton
          :is-online="currentConnection.serverPowerState === 'running'"
          :is-actioning="currentConnection.isActioning"
          @action="(action) => sendPowerAction(serverId, action)"
        />
      </div>

      <PanelTerminal
        :console-output="currentConnection.consoleOutput"
        :full-screen="fullScreen"
        @toggle-full-screen="toggleFullScreen"
      />
    </div>
  </div>
  <PanelOverviewLoading v-else-if="currentConnection?.connectionState === 'connecting'" />
  <PyroError
    v-else-if="currentConnection?.connectionState === 'auth-failed'"
    title="WebSocket authentication failed"
    message="Indicative of a server misconfiguration. Please report this to support."
  />
  <PyroError
    v-else-if="currentConnection?.connectionState === 'error'"
    :title="currentConnection.errorTitle"
    :message="currentConnection.errorMessage"
  />
</template>

<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue'
import { useRoute } from 'vue-router'
import { storeToRefs } from 'pinia'
import { useWebSocketStore } from '@/store/websocket'
import type { Server } from '@/types/servers'
import ServerStats from '@/components/ui/servers/ServerStats.vue'
import PanelServerStatus from '@/components/ui/servers/PanelServerStatus.vue'
import PanelServerActionButton from '@/components/ui/servers/PanelServerActionButton.vue'
import PanelTerminal from '@/components/ui/servers/PanelTerminal.vue'
import PanelOverviewLoading from '@/components/ui/servers/PanelOverviewLoading.vue'
import PyroError from '@/components/ui/servers/PyroError.vue'
import type { Credentials } from '@/store/credentials'

const webSocketStore = useWebSocketStore()
const route = useRoute()

const { connections } = storeToRefs(webSocketStore)

const fullScreen = ref(false)
const consoleStyle = ref({ height: '600px', marginTop: '0px' })

const props = defineProps<{
  server: Server
  credentials: Credentials
}>()

const serverId = computed(() => route.params.id as string)

const currentConnection = computed(() => connections.value[serverId.value])

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

const sendPowerAction = async (serverId: string, action: 'restart' | 'start' | 'stop' | 'kill') => {
  await webSocketStore.sendPowerAction(serverId, action)
}

onMounted(() => {
  webSocketStore.connect(serverId.value, props.credentials.session)
})

watch(serverId, async (newId, oldId) => {
  if (newId !== oldId) {
    if (!connections.value[newId]) {
      await webSocketStore.connect(newId, props.credentials.session)
    }
  }
})

window.addEventListener('online', () => {
  if (currentConnection.value?.connectionState !== 'connected') {
    webSocketStore.connect(serverId.value, props.credentials.session)
  }
})

window.addEventListener('offline', () => {
  if (currentConnection.value) {
    currentConnection.value.connectionState = 'error'
    currentConnection.value.errorTitle = 'Network disconnected'
    currentConnection.value.errorMessage =
      'Your internet connection appears to be offline. Please check your connection and try again.'
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
