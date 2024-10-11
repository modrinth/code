<template>
  <div class="contents">
    <UiServersPanelLoading v-if="initialLoading" class="h-screen" />
    <div
      v-else-if="serverData && serverData.status !== 'installing'"
      data-pyro-server-manager-root
      class="mx-auto flex min-h-screen w-full max-w-[1280px] flex-col gap-6 px-3"
    >
      <div class="flex flex-row items-center gap-6 pt-4">
        <UiServersServerIcon :server-id="serverId" />
        <div class="flex w-full flex-col gap-2">
          <div class="flex shrink-0 flex-row items-center gap-1">
            <NuxtLink to="/servers/manage" class="breadcrumb goto-link flex w-fit items-center">
              <LeftArrowIcon />
              All servers
            </NuxtLink>
          </div>
          <div class="flex flex-row items-center gap-4">
            <h1 class="m-0 flex-grow text-4xl font-bold text-[var(--color-contrast)]">
              {{ serverData.name }}
            </h1>
            <UiServersPanelServerActionButton
              class="flex-shrink-0"
              :is-online="isServerRunning"
              :is-actioning="isActioning"
              @action="sendPowerAction"
            />
          </div>

          <div class="flex flex-row items-center gap-4 text-[var(--color-text-secondary)]">
            <UiServersServerGameLabel
              v-if="showGameLabel"
              :game="serverData.game!"
              :mc-version="serverData.mc_version ?? ''"
            />
            <UiServersServerLoaderLabel
              v-if="showLoaderLabel"
              :loader="serverData.loader!"
              :loader-version="serverData.loader_version ?? ''"
            />
            <UiServersServerSubdomainLabel
              v-if="serverData.net.domain"
              :subdomain="serverData.net.domain"
            />
          </div>
        </div>
      </div>

      <div class="flex w-full flex-col justify-between gap-4 md:flex-row md:items-center">
        <UiNavTabs :links="navLinks" />
      </div>

      <div data-pyro-mount class="h-full w-full flex-1">
        <NuxtPage
          :route="route"
          :is-connected="isConnected"
          :is-ws-auth-incorrect="isWSAuthIncorrect"
          :is-server-running="isServerRunning"
          :stats="stats"
          :server-power-state="serverPowerState"
          :console-output="consoleOutput"
          :socket="socket"
        />
      </div>

      <UiServersPoweredByPyro />
    </div>

    <UiServersPyroError v-else-if="error" :title="errorTitle" :message="errorMessage" />
    <div v-else class="flex h-screen flex-col items-center justify-center">
      <p class="text-lg font-bold">Get ready! We're preparing your server for you.</p>
      <div class="h-1.5 w-full max-w-lg overflow-hidden rounded-xl bg-brand-highlight">
        <div class="progress left-right h-full w-full bg-brand"></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { LeftArrowIcon } from "@modrinth/assets";
import type { Server, ServerState, Stats, WSAuth, WSEvent } from "~/types/servers";
import type { NuxtPage } from "#build/components";

let socket: WebSocket | null = null;

const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();

const errorTitle = ref("Error");
const errorMessage = ref("An unexpected error occurred.");
const initialLoading = ref(true);
const serverData = ref<Server | null>(null);
const error = ref<Error | null>(null);
const isConnected = ref(false);
const isWSAuthIncorrect = ref(false);
const consoleOutput = ref<string[]>([]);
const cpuData = ref<number[]>([]);
const ramData = ref<number[]>([]);
const isActioning = ref(false);
const app = useNuxtApp();
const isServerRunning = computed(() => serverPowerState.value === "running");
const serverPowerState = ref<ServerState>("stopped");

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

const fetchServerData = async () => {
  try {
    await serverStore.fetchServerData(serverId);
    serverData.value = serverStore.serverData[serverId];
    initialLoading.value = false;
    error.value = null;
  } catch (err) {
    initialLoading.value = false;
    error.value = err as Error;
    if (err instanceof PyroFetchError) {
      switch (err.statusCode) {
        case 400:
          errorTitle.value = "Oh no, a pop-up";
          errorMessage.value = "Request was malformed.";
          break;
        case 401:
        case 404:
          errorTitle.value = "Server Not Found";
          errorMessage.value = "The server you are looking for does not exist.";
          break;
        default:
          errorTitle.value = "Error";
          errorMessage.value = `An error occurred: ${err.message}`;
      }
    } else {
      errorTitle.value = "Unexpected Error";
      errorMessage.value = "An unexpected error occurred while fetching server data.";
    }
  }
};

const showGameLabel = computed(() => !!serverData.value?.game);
const showLoaderLabel = computed(() => !!serverData.value?.loader);

const navLinks = [
  { label: "Overview", href: `/servers/manage/${serverId}`, subpages: [] },
  {
    label: "Content",
    href: `/servers/manage/${serverId}/content`,
    subpages: ["mods", "datapacks"],
  },
  { label: "Files", href: `/servers/manage/${serverId}/files`, subpages: [] },
  { label: "Backups", href: `/servers/manage/${serverId}/backups`, subpages: [] },
  {
    label: "Options",
    href: `/servers/manage/${serverId}/options`,
    subpages: ["startup", "network", "properties", "info"],
  },
];

const connectWebSocket = async () => {
  try {
    const wsAuth = (await serverStore.requestWebsocket(serverId)) as WSAuth;
    socket = new WebSocket(`wss://${wsAuth.url}`);

    socket.onopen = () => {
      socket?.send(JSON.stringify({ event: "auth", jwt: wsAuth.token }));
    };

    socket.onmessage = (event) => {
      const data: WSEvent = JSON.parse(event.data);
      handleWebSocketMessage(data);
    };

    socket.onclose = () => {
      consoleOutput.value.push("\nWS connection closed");
      isConnected.value = false;
    };

    socket.onerror = (error) => {
      console.error("WebSocket error:", error);
      isConnected.value = false;
    };
  } catch (error) {
    console.error("Failed to connect WebSocket:", error);
    isConnected.value = false;
  }
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
      reauthenticate();
      break;
    case "power-state":
      updatePowerState(data.state);
      break;
    case "auth-incorrect":
      isWSAuthIncorrect.value = true;
      break;
    default:
      console.warn("Unhandled WebSocket event:", data);
  }
};

const updateStats = (currentStats: Stats["current"]) => {
  isConnected.value = true;
  stats.value = {
    current: currentStats,
    past: { ...stats.value.current },
    graph: {
      cpu: updateGraphData(cpuData.value, currentStats.cpu_percent),
      ram: updateGraphData(
        ramData.value,
        Math.floor((currentStats.ram_usage_bytes / currentStats.ram_total_bytes) * 100),
      ),
    },
  };
};

const updatePowerState = (state: ServerState) => {
  serverPowerState.value = state;
};

const updateGraphData = (dataArray: number[], newValue: number): number[] => {
  const updated = [...dataArray, newValue];
  if (updated.length > 10) updated.shift();
  return updated;
};

const reauthenticate = async () => {
  try {
    const wsAuth = (await serverStore.requestWebsocket(serverId)) as WSAuth;
    socket?.send(JSON.stringify({ event: "auth", jwt: wsAuth.token }));
  } catch (error) {
    console.error("Reauthentication failed:", error);
    isWSAuthIncorrect.value = true;
  }
};

const toAdverb = (word: string) => {
  if (word.endsWith("p")) {
    return word + "ping";
  }
  if (word.endsWith("e")) {
    return word.slice(0, -1) + "ing";
  }
  if (word.endsWith("ie")) {
    return word.slice(0, -2) + "ying";
  }
  return word + "ing";
};

const sendPowerAction = async (action: "restart" | "start" | "stop" | "kill") => {
  const actionName = action.charAt(0).toUpperCase() + action.slice(1);
  try {
    isActioning.value = true;
    await serverStore.sendPowerAction(serverId, actionName);
    notifySuccess(`${toAdverb(actionName)} server`, `This may take a few moments.`);
  } catch (error) {
    console.error(`Error ${toAdverb(actionName)} server:`, error);
    notifyError(
      `Error ${toAdverb(actionName)} server`,
      "An error occurred while performing this action.",
    );
  } finally {
    isActioning.value = false;
  }
};

const notifySuccess = (title: string, text: string) => {
  // @ts-ignore
  app.$notify({
    group: "server",
    title,
    text,
    type: "success",
  });
};

const notifyError = (title: string, text: string) => {
  // @ts-ignore
  app.$notify({
    group: "server",
    title,
    text,
    type: "error",
  });
};

let intervalId: ReturnType<typeof setInterval> | null = null;

const startPolling = () => {
  intervalId = setInterval(async () => {
    await fetchServerData();
  }, 10000);
};

const stopPolling = () => {
  if (intervalId) {
    clearInterval(intervalId);
    intervalId = null;
  }
};

onMounted(() => {
  connectWebSocket();
  fetchServerData();
});

onUnmounted(() => {
  stopPolling();
  socket?.close();
});

watch(
  () => serverData.value?.status,
  (newStatus) => {
    if (newStatus === "installing") {
      startPolling();
    } else {
      stopPolling();
    }
  },
);

definePageMeta({
  middleware: "auth",
});
</script>

<style scoped>
.progress {
  animation: progress 1s infinite linear;
}

.left-right {
  transform-origin: 0% 50%;
}

@keyframes progress {
  0% {
    transform: translateX(0) scaleX(0);
  }
  40% {
    transform: translateX(0) scaleX(0.4);
  }
  100% {
    transform: translateX(100%) scaleX(0.5);
  }
}
</style>
