<template>
  <div
    v-if="isConnected && !isWSAuthIncorrect"
    data-pyro-server-manager-root
    class="relative flex flex-col gap-6"
  >
    <div
      :class="fullScreen ? '-translate-y-4 scale-95 opacity-0' : 'opacity-100'"
      class="absolute left-0 right-0 top-0 w-full transition-all duration-[400ms]"
    >
      <UiServersServerStats :data="stats" />
    </div>
    <div
      class="relative flex w-full flex-col gap-3 overflow-hidden rounded-2xl border border-divider bg-bg-raised p-8 transition-[height,_margin-top] duration-300 ease-in-out"
      :style="consoleStyle"
    >
      <div class="experimental-styles-within flex flex-row items-center justify-between">
        <div class="flex flex-row items-center gap-4">
          <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">Console</h2>
          <UiServersPanelServerStatus :state="serverPowerState" />
        </div>
        <div class="ml-auto mr-2 flex flex-row gap-2">
          <UiServersPanelCopyIP
            :ip="serverStore.serverData[serverId]?.net.ip"
            :port="serverStore.serverData[serverId]?.net.port"
            :subdomain="serverStore.serverData[serverId]?.net.domain"
          />
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
        <div class="relative w-full px-2.5 pt-2">
          <div class="relative w-full">
            <span
              v-if="suggestion"
              class="pointer-events-none absolute left-[1.2rem] top-[48%] flex -translate-y-1/2 transform items-center text-gray-400"
              :style="suggestionStyle"
            >
              {{ suggestion }}
              <span class="ml-1 text-xs">Tab</span>
            </span>
            <input
              v-model="commandInput"
              type="text"
              placeholder="Send a command"
              class="z-50 w-full rounded-md p-2 pt-4 focus:border-none [&&]:border-[1px] [&&]:border-solid [&&]:border-bg-raised [&&]:bg-bg"
              @keydown.tab.prevent="acceptSuggestion"
              @input="handleInput"
            />
          </div>
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
import { ref, onMounted, onBeforeUnmount, computed, watch } from "vue";
import type { ServerState, Stats, WSAuth, WSEvent } from "~/types/servers";

const serverStore = useServerStore();
const app = useNuxtApp();

const fullScreen = ref(false);
const isConnected = ref(false);
const isWSAuthIncorrect = ref(false);
const consoleOutput = ref<string[]>([]);
const cpuData = ref<number[]>([]);
const ramData = ref<number[]>([]);
const isActioning = ref(false);
const serverPowerState = ref<ServerState>("stopped");
const commandInput = ref("");
const suggestion = ref("");

const suggestionStyle = computed(() => ({
  width: `${commandInput.value.length}ch`,
  whiteSpace: "pre",
}));

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

const consoleStyle = ref({
  marginTop: "254px",
  height: "600px",
  transition: "height 0.3s ease-in-out, margin-top 0.3s ease-in-out",
});

const toggleFullScreen = () => {
  fullScreen.value = !fullScreen.value;
  if (fullScreen.value) {
    consoleStyle.value.height = "85vh";
    consoleStyle.value.marginTop = "0px";
  } else {
    consoleStyle.value.height = "600px";
    consoleStyle.value.marginTop = "254px";
  }
};

const sendPowerAction = async (action: "restart" | "start" | "stop" | "kill") => {
  const actionName = action.charAt(0).toUpperCase() + action.slice(1);
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

const minecraftCommands = [
  "/gamemode",
  "/tp",
  "/give",
  "/ban",
  "/kick",
  "/whitelist",
  "/setworldspawn",
  "/time",
  "/weather",
  "/spawnpoint",
  "/say",
  "/op",
  "/deop",
  "/list",
  "/clear",
  "/msg",
  "/enchant",
  "/effect",
  "/summon",
  "/setblock",
  "/fill",
  "/clone",
  "/scoreboard",
  "/xp",
  "/title",
  "/stopsound",
  "/playsound",
  "/locate",
];

const filteredSuggestion = computed(() => {
  const input = commandInput.value.toLowerCase();
  const inputLength = commandInput.value.length;
  let matchingCommand = "";

  if (inputLength === 0) return "";

  if (commandInput.value.startsWith("/")) {
    matchingCommand = minecraftCommands.find((cmd) => cmd.startsWith(input));
  } else {
    matchingCommand = minecraftCommands.find((cmd) => cmd.startsWith("/" + input));
  }

  if (matchingCommand && matchingCommand !== commandInput.value) {
    const suggestion = matchingCommand.slice(inputLength + (input.startsWith("/") ? 0 : 1));
    return " ".repeat(inputLength) + suggestion;
  }

  return "";
});

watch(filteredSuggestion, (newVal) => {
  suggestion.value = newVal;
});

const handleInput = () => {
  if (commandInput.value.trim() === "") {
    suggestion.value = "";
  }
};

const acceptSuggestion = () => {
  if (suggestion.value) {
    if (commandInput.value.startsWith("/")) {
      commandInput.value += suggestion.value.replaceAll(" ", "");
    } else {
      commandInput.value = commandInput.value + suggestion.value.replaceAll(" ", "");
    }
    suggestion.value = "";
  }
};

const sendCommand = async () => {
  if (!socket || commandInput.value.trim() === "") return;
  await socket.send(JSON.stringify({ event: "command", cmd: commandInput.value }));
  commandInput.value = "";
  suggestion.value = "";
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

onMounted(() => {
  connectWebSocket();
});

onBeforeUnmount(() => {
  socket?.close();
});
</script>
