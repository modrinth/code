<template>
  <div class="contents">
    <div
      v-if="error"
      class="mx-auto mb-4 flex h-full w-full max-w-[1280px] items-center justify-between gap-2 rounded-xl border-2 border-solid border-red bg-bg-red p-4 font-semibold text-contrast"
    >
      <div class="flex flex-row items-center gap-2">
        <IssuesIcon class="h-8 w-8 text-red" />
        <div class="flex gap-2">{{ errorTitle }} - {{ errorMessage }}</div>
      </div>

      <div v-if="errorTitle === 'Installation error'">
        <ButtonStyled color="red" type="standard">
          <NuxtLink :to="`/servers/manage/${serverId}/options/loader`"> Change Loader </NuxtLink>
        </ButtonStyled>
      </div>
    </div>
    <div
      v-if="serverData"
      data-pyro-server-manager-root
      class="relative mx-auto box-border flex min-h-screen w-full min-w-0 max-w-[1280px] flex-col gap-6 px-3 transition-all duration-300"
    >
      <div class="flex w-full min-w-0 select-none flex-row items-center gap-6 pt-4">
        <UiServersServerIcon :image="serverData.image" />
        <div class="flex min-w-0 flex-1 flex-col gap-2">
          <div class="flex shrink-0 flex-row items-center gap-1">
            <NuxtLink to="/servers/manage" class="breadcrumb goto-link flex w-fit items-center">
              <LeftArrowIcon />
              All servers
            </NuxtLink>
          </div>
          <div class="flex w-full flex-row items-center gap-4">
            <h1
              class="m-0 w-full flex-shrink truncate text-4xl font-bold text-[var(--color-contrast)]"
            >
              {{ serverData.name }}
            </h1>
            <div
              v-if="isConnected && serverData.status !== 'installing'"
              data-pyro-server-action-buttons
              class="server-action-buttons-anim flex w-fit flex-shrink-0"
            >
              <UiServersPanelServerActionButton
                class="flex-shrink-0"
                :is-online="isServerRunning"
                :is-actioning="isActioning"
                :disabled="isActioning || !!error"
                @action="sendPowerAction"
              />
            </div>
            <ButtonStyled
              v-else-if="serverData.status === 'installing'"
              type="standard"
              color="brand"
            >
              <button disabled class="flex-shrink-0">
                <UiServersPanelSpinner class="size-5" /> Installing...
              </button>
            </ButtonStyled>
          </div>

          <div class="flex min-w-0 flex-row items-center gap-4 text-[var(--color-text-secondary)]">
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
            <UiServersServerUptimeLabel v-if="uptimeSeconds" :uptime-seconds="uptimeSeconds" />
          </div>
        </div>
      </div>

      <div
        data-pyro-navigation
        class="isolate flex w-full select-none flex-col justify-between gap-4 md:flex-row md:items-center"
      >
        <UiNavTabs :links="navLinks" />
      </div>

      <div data-pyro-mount class="h-full w-full flex-1">
        <div
          v-if="!isConnected && !isReconnecting && !isLoading"
          data-pyro-server-ws-error
          class="mb-4 flex w-full flex-row items-center gap-4 rounded-xl bg-bg-red p-4 text-contrast"
        >
          <IssuesIcon class="size-5 text-red" />
          Something went wrong...
        </div>

        <div
          v-if="isReconnecting"
          data-pyro-server-ws-reconnecting
          class="mb-4 flex w-full flex-row items-center gap-4 rounded-xl bg-bg-orange p-4 text-contrast"
        >
          <UiServersPanelSpinner />
          Hang on, we're reconnecting to your server.
        </div>

        <div
          v-if="serverData.status === 'installing'"
          data-pyro-server-installing
          class="mb-4 flex w-full flex-row items-center gap-4 rounded-xl bg-bg-orange p-4 text-contrast"
        >
          <UiServersPanelSpinner />
          We're preparing your server, this may take a few minutes.
        </div>

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
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { IssuesIcon, LeftArrowIcon } from "@modrinth/assets";
import DOMPurify from "dompurify";
import { ButtonStyled } from "@modrinth/ui";
import type { ServerState, Stats, WSEvent, WSInstallationResultEvent } from "~/types/servers";

const socket = ref<WebSocket | null>(null);
const isReconnecting = ref(false);
const isLoading = ref(true);
const reconnectInterval = ref<ReturnType<typeof setInterval> | null>(null);

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
const uptimeSeconds = ref(0);

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
      isConnected.value = true;
      isReconnecting.value = false;
      isLoading.value = false;
      consoleOutput.value.push("\nReady! Welcome to your Modrinth Server ༼ つ ◕_◕ ༽つ");
      consoleOutput.value.push("\nPress the green start button to start your server!");
      if (reconnectInterval.value) {
        clearInterval(reconnectInterval.value);
        reconnectInterval.value = null;
      }
    };

    socket.value.onmessage = (event) => {
      const data: WSEvent = JSON.parse(event.data);
      handleWebSocketMessage(data);
    };

    socket.value.onclose = () => {
      consoleOutput.value.push("\nSomething went wrong with the connection, we're reconnecting...");
      isConnected.value = false;
      scheduleReconnect();
    };

    socket.value.onerror = (error) => {
      console.error("WebSocket error:", error);
      isConnected.value = false;
      scheduleReconnect();
    };
  } catch (error) {
    console.error("Failed to connect WebSocket:", error);
    isConnected.value = false;
    scheduleReconnect();
  }
};

const scheduleReconnect = () => {
  if (!reconnectInterval.value) {
    isReconnecting.value = true;
    reconnectInterval.value = setInterval(() => {
      console.log("Attempting to reconnect...");
      connectWebSocket();
    }, 5000);
  }
};

const handleWebSocketMessage = (data: WSEvent) => {
  switch (data.event) {
    case "log":
      // eslint-disable-next-line no-case-declarations
      const log = data.message.split("\n").filter((l) => l.trim());
      consoleOutput.value.push(...log);
      break;
    case "stats":
      updateStats(data);
      break;
    case "auth-expiring":
    case "auth-incorrect":
      reauthenticate();
      break;
    case "power-state":
      updatePowerState(data.state);
      break;
    case "installation-result":
      handleInstallationResult(data);
      break;
    case "new-mod":
      server.refresh(["mods"]);
      console.log("New mod:", data);
      break;
    case "auth-ok":
      break;
    case "uptime":
      uptimeSeconds.value = data.uptime;
      break;
    default:
      console.warn("Unhandled WebSocket event:", data);
  }
};

const newLoader = ref<string | null>(null);
const newLoaderVersion = ref<string | null>(null);
const newMCVersion = ref<string | null>(null);

const handleInstallationResult = (data: WSInstallationResultEvent) => {
  switch (data.result) {
    case "ok":
      if (!serverData.value) break;
      serverData.value.status = "available";
      if (server.general) {
        if (newLoader.value) server.general.loader = newLoader.value;
        if (newLoaderVersion.value) server.general.loader_version = newLoaderVersion.value;
        if (newMCVersion.value) server.general.mc_version = newMCVersion.value;
      }
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

const onReinstall = (potentialArgs: any) => {
  if (!serverData.value) return;
  serverData.value.status = "installing";
  // serverData.value.loader = potentialArgs.loader;
  // serverData.value.loader_version = potentialArgs.lVersion;
  // serverData.value.mc_version = potentialArgs.mVersion;
  // if (potentialArgs?.loader) {
  //   console.log("setting loader to", potentialArgs.loader);
  //   serverData.value.loader = potentialArgs.loader;
  // }
  // if (potentialArgs?.lVersion) {
  //   serverData.value.loader_version = potentialArgs.lVersion;
  // }
  // if (potentialArgs?.mVersion) {
  //   serverData.value.mc_version = potentialArgs.mVersion;
  // }
  if (potentialArgs?.loader) {
    newLoader.value = potentialArgs.loader;
  }
  if (potentialArgs?.lVersion) {
    newLoaderVersion.value = potentialArgs.lVersion;
  }
  if (potentialArgs?.mVersion) {
    newMCVersion.value = potentialArgs.mVersion;
  }

  server.refresh();

  console.log(serverData.value);
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
  if (state === "stopped") {
    uptimeSeconds.value = 0;
  }
};

const updateGraphData = (dataArray: number[], newValue: number): number[] => {
  const updated = [...dataArray, newValue];
  if (updated.length > 10) updated.shift();
  return updated;
};

const reauthenticate = async () => {
  try {
    await server.refresh();
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
  DOMPurify.addHook(
    "afterSanitizeAttributes",
    (node: {
      tagName: string;
      getAttribute: (arg0: string) => any;
      setAttribute: (arg0: string, arg1: string) => void;
    }) => {
      if (node.tagName === "A" && node.getAttribute("target")) {
        node.setAttribute("rel", "noopener noreferrer");
      }
    },
  );
});

onUnmounted(() => {
  stopPolling();
  if (reconnectInterval.value) {
    clearInterval(reconnectInterval.value);
  }
  if (socket.value) {
    socket.value.close();
  }
  DOMPurify.removeHook("afterSanitizeAttributes");
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
