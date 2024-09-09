<template>
  <div
    v-if="isConnected && !isWSAuthIncorrect"
    data-pyro-server-manager-root
    class="flex flex-col gap-6"
  >
    <transition name="fade-slide">
      <UiServersServerStats v-if="!fullScreen" :data="stats" />
    </transition>
    <div
      class="relative flex w-full flex-col gap-3 overflow-hidden rounded-2xl bg-bg-raised p-8 transition-[height] duration-500 ease-in-out"
      :style="consoleStyle"
    >
      <div class="experimental-styles-within flex flex-row items-center justify-between">
        <div class="flex flex-row items-center gap-4">
          <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">Console</h2>
        </div>
      </div>

      <UiServersPanelTerminal
        :console-output="consoleOutput"
        :full-screen="fullScreen"
        @toggle-full-screen="toggleFullScreen"
      />
    </div>
  </div>
  <UiServersPanelOverviewLoading v-else-if="!isConnected && !isWSAuthIncorrect" />
  <UiServersPyroError
    v-else-if="isWSAuthIncorrect"
    title="WebSocket authentication failed"
    message="Indicative of a server misconfiguration. Please report this to support."
  />
  <UiServersPyroError
    v-else
    title="An error occurred"
    message="Something went wrong while attempting to connect to your server. Your data is safe, and we're working to resolve the issue."
  />
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from "vue";
import type { ServerState, Stats, WSAuth, WSEvent } from "~/types/servers";

const serverStore = useServerStore();

const fullScreen = ref(false);
const consoleStyle = ref({ height: "400px", marginTop: "0px" });
const isConnected = ref(false);
const isWSAuthIncorrect = ref(false);
const consoleOutput = ref<string[]>([]);
const cpuData = ref<number[]>([]);
const ramData = ref<number[]>([]);

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
});

const route = useRoute();
const serverId = route.params.id as string;

let socket: WebSocket | null = null;

const toggleFullScreen = () => {
  fullScreen.value = !fullScreen.value;
  if (fullScreen.value) {
    consoleStyle.value.height = "70vh";
    animateMarginTop();
  } else {
    consoleStyle.value.height = "400px";
    consoleStyle.value.marginTop = "0px";
  }
};

const animateMarginTop = () => {
  setTimeout(() => {
    let mt = 254;
    const interval = setInterval(() => {
      mt -= 10;
      consoleStyle.value.marginTop = `${mt}px`;
      if (mt <= 0 || !fullScreen.value) clearInterval(interval);
    }, 10);
  }, 500);
};

const connectWebSocket = async () => {
  const wsAuth = (await serverStore.requestWebsocket(serverId)) as WSAuth;
  socket = new WebSocket("wss://" + wsAuth.url);

  socket.onopen = () => {
    socket?.send(JSON.stringify({ event: "auth", jwt: wsAuth.token }));
  };

  socket.onmessage = (event) => handleWebSocketMessage(JSON.parse(event.data));

  socket.onclose = () => {
    consoleOutput.value.push("\nWS connection closed");
    isConnected.value = false;
  };

  socket.onerror = (error) => {
    console.error("WebSocket error:", error);
    isConnected.value = false;
  };
};

const handleWebSocketMessage = (data: WSEvent) => {
  switch (data.event) {
    case "log":
      consoleOutput.value.push(data.message);
      break;
    case "stats":
      updateStats(data as unknown as Stats["current"]);
      break;
    case "auth-expiring":
      reauth();
      break;
    case "power-state":
      updatePowerState(data.state);
      break;
    case "auth-incorrect":
      isWSAuthIncorrect.value = true;
      break;
  }
};

const updatePowerState = (state: ServerState) => {
  serverPowerState.value = state;
};

const updateStats = (data: Stats["current"]) => {
  isConnected.value = true;
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
  };
};

const updateDataArray = (arr: number[], newValue: number) => {
  arr.push(newValue);
  if (arr.length > 10) arr.shift();
  return [...arr];
};

const reauth = async () => {
  const wsAuth = (await serverStore.requestWebsocket(serverId)) as WSAuth;
  socket?.send(JSON.stringify({ event: "auth", jwt: wsAuth.token }));
};

onMounted(connectWebSocket);
onBeforeUnmount(() => socket?.close());
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
