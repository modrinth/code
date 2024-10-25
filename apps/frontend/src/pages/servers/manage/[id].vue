<template>
  <div class="contents">
    <div
      v-if="serverData"
      data-pyro-server-manager-root
      class="experimental-styles-within relative mx-auto box-border flex min-h-screen w-full min-w-0 max-w-[1280px] flex-col gap-6 px-3 transition-all duration-300"
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
            <h1 class="m-0 w-full flex-shrink truncate text-4xl font-bold text-contrast">
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

          <div class="flex min-w-0 flex-row items-center gap-4 text-secondary">
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
          v-if="error"
          class="mx-auto mb-4 flex justify-between gap-2 rounded-2xl border-2 border-solid border-red bg-bg-red p-4 font-semibold text-contrast"
        >
          <div class="flex flex-row gap-4">
            <IssuesIcon class="h-8 w-8 text-red" />
            <div class="flex flex-col gap-2 leading-[150%]">
              <div class="flex gap-2 text-xl font-bold">{{ errorTitle }}</div>
              <div
                v-if="errorTitle.toLocaleLowerCase() === 'installation error'"
                class="font-normal"
              >
                <div
                  v-if="
                    errorMessage.toLocaleLowerCase() === 'the specified version may be incorrect'
                  "
                >
                  An invalid loader or Minecraft version was specified and could not be installed.
                  <ul class="m-0 mt-4 p-0 pl-4">
                    <li>
                      If this version of Minecraft was released recently, please check if Modrinth
                      Servers supports it.
                    </li>
                    <li>
                      If you've installed a modpack, it may have been packaged incorrectly or may
                      not be compatible with the loader.
                    </li>
                    <li>
                      Your server may need to be reinstalled with a valid mod loader and version.
                      You can change the loader by clicking the "Change Loader" button.
                    </li>
                    <li>
                      If you're stuck, please contact Modrinth support with the information below:
                    </li>
                  </ul>
                  <ButtonStyled>
                    <button class="mt-2" @click="copyServerDebugInfo">
                      <CopyIcon v-if="!copied" />
                      <CheckIcon v-else />
                      Copy Debug Info
                    </button>
                  </ButtonStyled>
                </div>
                <div v-if="errorMessage.toLocaleLowerCase() === 'internal error'">
                  An internal error occurred while installing your server. Don't fret — try
                  reinstalling your server, and if the problem persists, please contact Modrinth
                  support with your server's debug information.
                </div>

                <div v-if="errorTitle === 'Installation error'" class="mt-2 flex flex-row gap-4">
                  <ButtonStyled>
                    <button @click="copyServerDebugInfo">
                      <CopyIcon v-if="!copied" />
                      <CheckIcon v-else />
                      Copy Debug Info
                    </button>
                  </ButtonStyled>
                  <ButtonStyled color="red" type="standard">
                    <NuxtLink
                      class="whitespace-pre"
                      :to="`/servers/manage/${serverId}/options/loader`"
                    >
                      <RightArrowIcon />
                      Change Loader
                    </NuxtLink>
                  </ButtonStyled>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div
          v-if="!isConnected && !isReconnecting && !isLoading"
          data-pyro-server-ws-error
          class="mb-4 flex w-full flex-row items-center gap-4 rounded-2xl bg-bg-red p-4 text-contrast"
        >
          <IssuesIcon class="size-5 text-red" />
          Something went wrong...
        </div>

        <div
          v-if="isReconnecting"
          data-pyro-server-ws-reconnecting
          class="mb-4 flex w-full flex-row items-center gap-4 rounded-2xl bg-bg-orange p-4 text-contrast"
        >
          <UiServersPanelSpinner />
          Hang on, we're reconnecting to your server.
        </div>

        <div
          v-if="serverData.status === 'installing'"
          data-pyro-server-installing
          class="mb-4 flex w-full flex-row items-center gap-4 rounded-2xl bg-bg-orange p-4 text-contrast"
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
import { CopyIcon, IssuesIcon, LeftArrowIcon, RightArrowIcon, CheckIcon } from "@modrinth/assets";
import DOMPurify from "dompurify";
import { ButtonStyled } from "@modrinth/ui";
import type { ServerState, Stats, WSEvent, WSInstallationResultEvent } from "~/types/servers";

const socket = ref<WebSocket | null>(null);
const isReconnecting = ref(false);
const isLoading = ref(true);
const reconnectInterval = ref<ReturnType<typeof setInterval> | null>(null);
const isMounted = ref(true);

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
const firstConnect = ref(true);
const copied = ref(false);

const initalConsoleMessage = [
  "   __________________________________________________",
  " /  Welcome to your \x1B[32mModrinth Server\x1B[37m!                  \\",
  "|   Press the green start button to start your server! |",
  " \\____________________________________________________/",
  "\x1B[32m     _    _ \x1B[37m",
  "\x1B[32m    (o)--(o)      \x1B[37m",
  "\x1B[32m   /.______.\\\x1B[37m",
  "\x1B[32m   \\________/     \x1B[37m",
  "\x1B[32m  ./        \\.    \x1B[37m",
  "\x1B[32m ( .        , )\x1B[37m",
  "\x1B[32m  \\ \\_\\\\ //_/ /\x1B[37m",
  "\x1B[32m   ~~  ~~  ~~\x1B[37m",
];

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
  if (!isMounted.value) return;

  try {
    const wsAuth = computed(() => server.ws);
    socket.value = new WebSocket(`wss://${wsAuth.value?.url}`);

    socket.value.onopen = () => {
      if (!isMounted.value) {
        socket.value?.close();
        return;
      }

      consoleOutput.value = [];
      socket.value?.send(JSON.stringify({ event: "auth", jwt: wsAuth.value?.token }));
      isConnected.value = true;
      isReconnecting.value = false;
      isLoading.value = false;

      if (firstConnect.value) {
        for (let i = 0; i < initalConsoleMessage.length; i++) {
          consoleOutput.value.push(initalConsoleMessage[i]);
        }
      }

      firstConnect.value = false;

      if (reconnectInterval.value) {
        if (reconnectInterval.value !== null) {
          clearInterval(reconnectInterval.value);
        }
        reconnectInterval.value = null;
      }
    };

    socket.value.onmessage = (event) => {
      if (isMounted.value) {
        const data: WSEvent = JSON.parse(event.data);
        handleWebSocketMessage(data);
      }
    };

    socket.value.onclose = () => {
      if (isMounted.value) {
        consoleOutput.value.push(
          "\nSomething went wrong with the connection, we're reconnecting...",
        );
        isConnected.value = false;
        scheduleReconnect();
      }
    };

    socket.value.onerror = (error) => {
      if (isMounted.value) {
        console.error("Failed to connect WebSocket:", error);
        isConnected.value = false;
        scheduleReconnect();
      }
    };
  } catch (error) {
    if (isMounted.value) {
      console.error("Failed to connect WebSocket:", error);
      isConnected.value = false;
      scheduleReconnect();
    }
  }
};

const scheduleReconnect = () => {
  if (!isMounted.value) return;

  if (!reconnectInterval.value) {
    isReconnecting.value = true;
    reconnectInterval.value = setInterval(() => {
      if (isMounted.value) {
        console.log("Attempting to reconnect...");
        connectWebSocket();
      } else {
        reconnectInterval.value = null;
      }
    }, 5000);
  }
};

let uptimeIntervalId: ReturnType<typeof setInterval> | null = null;

const startUptimeUpdates = () => {
  uptimeIntervalId = setInterval(() => {
    uptimeSeconds.value += 1;
  }, 1000);
};

const stopUptimeUpdates = () => {
  if (uptimeIntervalId) {
    clearInterval(uptimeIntervalId);
    intervalId = null;
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
      stopUptimeUpdates();
      uptimeSeconds.value = data.uptime;
      startUptimeUpdates();
      break;
    default:
      console.warn("Unhandled WebSocket event:", data);
  }
};

const newLoader = ref<string | null>(null);
const newLoaderVersion = ref<string | null>(null);
const newMCVersion = ref<string | null>(null);

const handleInstallationResult = async (data: WSInstallationResultEvent) => {
  switch (data.result) {
    case "ok":
      await server.refresh();
      if (!serverData.value) break;
      serverData.value.status = "available";
      if (server.general) {
        if (newLoader.value) server.general.loader = newLoader.value;
        if (newLoaderVersion.value) server.general.loader_version = newLoaderVersion.value;
        if (newMCVersion.value) server.general.mc_version = newMCVersion.value;
      }

      error.value = null;
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

  error.value = null;
  errorTitle.value = "Error";
  errorMessage.value = "An unexpected error occurred.";

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
    stopUptimeUpdates();
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

const copyServerDebugInfo = () => {
  const debugInfo = `Server ID: ${serverData.value?.server_id}\nError: ${errorMessage.value}\nKind: ${serverData.value?.upstream?.kind}\nProject ID: ${serverData.value?.upstream?.project_id}\nVersion ID: ${serverData.value?.upstream?.version_id}`;
  navigator.clipboard.writeText(debugInfo);
  copied.value = true;
  setTimeout(() => {
    copied.value = false;
  }, 5000);
};

const cleanup = () => {
  isMounted.value = false;

  stopPolling();
  stopUptimeUpdates();
  if (reconnectInterval.value) {
    clearInterval(reconnectInterval.value);
    reconnectInterval.value = null;
  }

  if (socket.value) {
    socket.value.onopen = null;
    socket.value.onmessage = null;
    socket.value.onclose = null;
    socket.value.onerror = null;

    if (
      socket.value.readyState === WebSocket.OPEN ||
      socket.value.readyState === WebSocket.CONNECTING
    ) {
      socket.value.close();
    }
    socket.value = null;
  }

  isConnected.value = false;
  isReconnecting.value = false;
  isLoading.value = true;

  DOMPurify.removeHook("afterSanitizeAttributes");
};

onMounted(() => {
  isMounted.value = true;
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
  cleanup();
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
