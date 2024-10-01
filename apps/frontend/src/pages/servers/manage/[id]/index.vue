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
      class="relative flex w-full flex-col gap-3 overflow-hidden rounded-2xl border-[1px] border-solid border-divider bg-bg-raised p-8 transition-[height] duration-500 ease-in-out"
      :style="consoleStyle"
    >
      <div class="experimental-styles-within flex flex-row items-center justify-between">
        <div class="flex flex-row items-center gap-4">
          <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">Console</h2>
          <UiServersPanelServerStatus :state="serverPowerState" />
        </div>
        <UiServersPanelServerActionButton
          :is-online="serverPowerState === 'running'"
          :is-actioning="isActioning"
          @action="sendPowerAction"
        />
      </div>
      <UiServersPanelTerminal
        :console-output="consoleOutput"
        :full-screen="fullScreen"
        @toggle-full-screen="toggleFullScreen"
      >
        <div class="w-full px-2.5 pt-2">
          <input
            v-model="commandInput"
            type="text"
            placeholder="Send a command"
            class="z-50 w-full rounded-md p-2 pt-4 focus:border-none [&&]:border-[1px] [&&]:border-solid [&&]:border-bg-raised [&&]:bg-bg"
            @keyup.enter="sendCommand"
          />
        </div>
      </UiServersPanelTerminal>
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
const app = useNuxtApp();

const fullScreen = ref(false);
const consoleStyle = ref({ height: "600px", marginTop: "0px" });
const isConnected = ref(false);
const isWSAuthIncorrect = ref(false);
const consoleOutput = ref<string[]>([]);
const cpuData = ref<number[]>([]);
const ramData = ref<number[]>([]);
const isActioning = ref(false);
const serverPowerState = ref<ServerState>("stopped");
const commandInput = ref("");

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

useHead({
  title: `Overview - ${serverStore.serverData[serverId]?.name ?? "Server"} - Modrinth`,
});

let socket: WebSocket | null = null;

const toggleFullScreen = () => {
  fullScreen.value = !fullScreen.value;
  if (fullScreen.value) {
    consoleStyle.value.height = "90vh";
    animateMarginTop();
  } else {
    consoleStyle.value.height = "600px";
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

const sendPowerAction = async (action: "restart" | "start" | "stop" | "kill") => {
  const actionName = action.charAt(0).toUpperCase() + action.slice(1);
  // @ts-ignore
  app.$notify({
    group: "server",
    title: `${actionName}ing server`,
    text: `Your server is now ${actionName.toLocaleLowerCase()}ing, this may take a few moments`,
    type: "success",
  });

  try {
    isActioning.value = true;
    await serverStore.sendPowerAction(serverId, actionName);
  } catch (error) {
    console.error(`Error ${actionName}ing server:`, error);
    // @ts-ignore
    app.$notify({
      group: "server",
      title: `Error ${actionName}ing server`,
      text: "An error occurred while attempting to perform the action.",
      type: "error",
    });
  } finally {
    isActioning.value = false;
  }
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

const sendCommand = async () => {
  if (!socket) return;
  console.log("Sending command", commandInput.value);
  await socket.send(JSON.stringify({ event: "command", cmd: commandInput.value }));
  commandInput.value = "";
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
