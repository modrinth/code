<template>
  <div class="contents">
    <UiServersPyroError v-if="error" :title="errorTitle" :message="errorMessage" />
    <div
      v-if="serverData && serverData.status !== 'installing'"
      data-pyro-server-manager-root
      class="mx-auto flex min-h-screen w-full max-w-[1280px] flex-col gap-6 px-3 transition-all duration-300"
      :class="error ? 'pointer-events-none select-none blur-md' : ''"
    >
      <div class="flex flex-row items-center gap-6 pt-4">
        <UiServersServerIcon :image="serverData.image" />
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
            <!-- only show power action button when websocket is fully connected -->
            <div
              v-if="isConnected"
              data-pyro-server-action-buttons
              class="server-action-buttons-anim"
            >
              <UiServersPanelServerActionButton
                class="flex-shrink-0"
                :is-online="isServerRunning"
                :is-actioning="isActioning"
                @action="sendPowerAction"
              />
            </div>
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
          :server="server"
          @reinstall="onReinstall"
        />
      </div>

      <UiServersPoweredByPyro />
    </div>

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
import type { ServerState, Stats, WSEvent, WSInstallationResultEvent } from "~/types/servers";

const socket = ref<WebSocket | null>(null);

const route = useNativeRoute();
const serverId = route.params.id as string;
const server = await usePyroServer(serverId, [
  "general",
  "mods",
  "backups",
  "network",
  "startup",
  "ws",
  "fs",
]);

const errorTitle = ref("Error");
const errorMessage = ref("An unexpected error occurred.");
const serverData = computed(() => server.general);
const error = ref<Error | null>(null);
const isConnected = ref(false);
const isWSAuthIncorrect = ref(false);
const consoleOutput = ref<string[]>([]);
const cpuData = ref<number[]>([]);
const ramData = ref<number[]>([]);
const isActioning = ref(false);
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

const connectWebSocket = () => {
  try {
    const wsAuth = computed(() => server.ws);
    socket.value = new WebSocket(`wss://${wsAuth.value?.url}`);

    socket.value.onopen = () => {
      socket.value?.send(JSON.stringify({ event: "auth", jwt: wsAuth.value?.token }));
    };

    socket.value.onmessage = (event) => {
      const data: WSEvent = JSON.parse(event.data);
      handleWebSocketMessage(data);
    };

    socket.value.onclose = () => {
      consoleOutput.value.push("\nWS connection closed");
      isConnected.value = false;
    };

    socket.value.onerror = (error) => {
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
      consoleOutput.value.push(...data.message.split("\n").filter((l) => l.trim()));
      break;
    case "stats":
      updateStats(data);
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
    case "installation-result":
      handleInstallationResult(data);
      break;
    case "auth-ok":
      break;
    default:
      console.warn("Unhandled WebSocket event:", data);
  }
};

const handleInstallationResult = (data: WSInstallationResultEvent) => {
  switch (data.result) {
    case "ok":
      if (!serverData.value) break;
      serverData.value.status = "available";
      break;
    case "err":
      console.log("failed to install");
      console.log(data);
      errorTitle.value = "Installation error";
      errorMessage.value = data.reason ?? "Unknown error";
      error.value = new Error(data.reason ?? "Unknown error");
      break;
  }
};

const onReinstall = () => {
  console.log("QAHHHHHHHHHHHHHHH");
  if (!serverData.value) return;
  serverData.value.status = "installing";
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

const reauthenticate = () => {
  try {
    const wsAuth = computed(() => server.ws);
    socket.value?.send(JSON.stringify({ event: "auth", jwt: wsAuth.value?.token }));
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
    await server.general?.power(actionName);
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
  addNotification({
    group: "server",
    title,
    text,
    type: "success",
  });
};

const notifyError = (title: string, text: string) => {
  addNotification({
    group: "server",
    title,
    text,
    type: "error",
  });
};

let intervalId: ReturnType<typeof setInterval> | null = null;

const startPolling = () => {
  intervalId = setInterval(async () => {
    await server.refresh();
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
});

onUnmounted(() => {
  stopPolling();
  socket.value?.close();
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

@keyframes server-action-buttons-anim {
  0% {
    opacity: 0;
    transform: translateX(1rem);
  }
  100% {
    opacity: 1;
    transform: none;
  }
}

.server-action-buttons-anim {
  animation: server-action-buttons-anim 0.2s ease-out;
}
</style>
