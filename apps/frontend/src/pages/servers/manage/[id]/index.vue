<template>
  <div
    v-if="isConnected && !isWSAuthIncorrect"
    class="relative flex select-none flex-col gap-6"
    data-pyro-server-manager-root
  >
    <UiServersServerStats :data="stats" />
    <div
      class="relative flex h-[600px] w-full flex-col gap-3 overflow-hidden rounded-2xl border border-divider bg-bg-raised p-8 transition-all duration-300 ease-in-out"
    >
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-4">
          <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">Console</h2>
          <UiServersPanelServerStatus :state="serverPowerState" />
        </div>
        <div class="ml-auto mr-2 flex gap-2">
          <UiServersPanelCopyIP :ip="serverIP" :port="serverPort" :subdomain="serverDomain" />
        </div>
        <UiServersPanelServerActionButton
          :is-online="isServerRunning"
          :is-actioning="isActioning"
          @action="sendPowerAction"
        />
      </div>
      <UiServersPanelTerminal :console-output="consoleOutput" :full-screen="fullScreen">
        <div class="relative w-full px-4 pt-4">
          <ul
            v-if="suggestions.length"
            id="command-suggestions"
            class="z-20 mt-1 max-h-60 w-full list-none overflow-auto rounded-md border border-divider bg-bg-raised p-0 shadow-lg"
            role="listbox"
          >
            <li
              v-for="(suggestion, index) in suggestions"
              :id="'suggestion-' + index"
              :key="index"
              role="option"
              :aria-selected="index === selectedSuggestionIndex"
              :class="[
                'cursor-pointer px-4 py-2',
                index === selectedSuggestionIndex ? 'bg-bg-raised' : 'bg-bg',
              ]"
              @click="selectSuggestion(index)"
            >
              {{ suggestion }}
            </li>
          </ul>
          <div class="relative">
            <span
              v-if="bestSuggestion"
              class="pointer-events-none absolute left-[1.7rem] top-[52%] z-20 -translate-y-1/2 transform select-none text-gray-400"
            >
              {{ ".".repeat(commandInput.length - 1) }}{{ bestSuggestion }}
              <button
                class="text z-50 cursor-pointer border-none bg-transparent text-sm focus:outline-none"
                aria-label="Accept suggestion"
                @click="acceptSuggestion"
              >
                TAB
              </button>
            </span>

            <input
              v-if="isServerRunning"
              v-model="commandInput"
              type="text"
              placeholder="Send a command"
              class="z-50 w-full rounded-md px-4 pt-4 focus:border-none [&&]:border-[1px] [&&]:border-solid [&&]:border-bg-raised [&&]:bg-bg"
              aria-autocomplete="list"
              aria-controls="command-suggestions"
              :aria-activedescendant="'suggestion-' + selectedSuggestionIndex"
              @keydown.tab.prevent="acceptSuggestion"
              @keydown.down.prevent="selectNextSuggestion"
              @keydown.up.prevent="selectPrevSuggestion"
              @keydown.enter.prevent="sendCommand"
            />
            <input
              v-else
              disabled
              type="text"
              placeholder="Send a command"
              class="z-50 w-full rounded-md px-4 pt-4 focus:border-none [&&]:border-[1px] [&&]:border-solid [&&]:border-bg-raised [&&]:bg-bg"
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
import type { ServerState, Stats, WSAuth, WSEvent } from "~/types/servers";

const DYNAMIC_ARG = Symbol("DYNAMIC_ARG");

const commandTree: any = {
  advancement: {
    grant: {
      [DYNAMIC_ARG]: {
        everything: null,
        only: {
          [DYNAMIC_ARG]: null,
        },
        from: {
          [DYNAMIC_ARG]: null,
        },
        through: {
          [DYNAMIC_ARG]: null,
        },
        until: {
          [DYNAMIC_ARG]: null,
        },
      },
    },
    revoke: {
      [DYNAMIC_ARG]: {
        everything: null,
        only: {
          [DYNAMIC_ARG]: null,
        },
        from: {
          [DYNAMIC_ARG]: null,
        },
        through: {
          [DYNAMIC_ARG]: null,
        },
        until: {
          [DYNAMIC_ARG]: null,
        },
      },
    },
  },
  ban: {
    [DYNAMIC_ARG]: {
      [DYNAMIC_ARG]: null,
      duration: {
        [DYNAMIC_ARG]: null,
      },
    },
  },
  ban_ip: null,
  banlist: {
    ips: null,
    players: null,
    all: null,
  },
  bossbar: {
    add: null,
    get: null,
    list: null,
    remove: null,
    set: null,
  },
  clear: {
    [DYNAMIC_ARG]: {
      [DYNAMIC_ARG]: {
        [DYNAMIC_ARG]: null,
        reason: null,
      },
    },
  },
  clone: null,
  data: {
    get: null,
    merge: null,
    modify: null,
    remove: null,
  },
  datapack: {
    disable: null,
    enable: null,
    list: null,
    reload: null,
  },
  debug: {
    start: null,
    stop: null,
    function: null,
    memory: null,
  },
  defaultgamemode: {
    survival: null,
    creative: null,
    adventure: null,
    spectator: null,
  },
  deop: null,
  difficulty: {
    peaceful: null,
    easy: null,
    normal: null,
    hard: null,
  },
  effect: {
    give: {
      [DYNAMIC_ARG]: {
        [DYNAMIC_ARG]: {
          [DYNAMIC_ARG]: {
            [DYNAMIC_ARG]: {
              true: null,
              false: null,
            },
          },
        },
      },
    },
    clear: {
      [DYNAMIC_ARG]: {
        [DYNAMIC_ARG]: null,
      },
    },
  },
  enchant: null,
  execute: null,
  experience: {
    add: null,
    set: null,
    query: null,
  },
  fill: null,
  forceload: {
    add: null,
    remove: null,
    query: null,
  },
  function: null,
  gamemode: {
    survival: {
      [DYNAMIC_ARG]: null,
    },
    creative: {
      [DYNAMIC_ARG]: null,
    },
    adventure: {
      [DYNAMIC_ARG]: null,
    },
    spectator: {
      [DYNAMIC_ARG]: null,
    },
  },
  gamerule: null,
  give: {
    [DYNAMIC_ARG]: {
      [DYNAMIC_ARG]: {
        [DYNAMIC_ARG]: null,
      },
    },
  },
  help: null,
  kick: null,
  kill: {
    [DYNAMIC_ARG]: null,
  },
  list: null,
  locate: {
    biome: null,
    poi: null,
    structure: null,
  },
  loot: {
    give: null,
    insert: null,
    replace: null,
    spawn: null,
  },
  me: null,
  msg: null,
  op: null,
  pardon: null,
  pardon_ip: null,
  particle: null,
  playsound: null,
  recipe: {
    give: null,
    take: null,
  },
  reload: null,
  say: null,
  schedule: {
    function: null,
    clear: null,
  },
  scoreboard: {
    objectives: {
      add: null,
      remove: null,
      setdisplay: null,
      list: null,
      modify: null,
    },
    players: {
      add: null,
      remove: null,
      set: null,
      get: null,
      list: null,
      enable: null,
      operation: null,
      reset: null,
    },
  },
  seed: null,
  setblock: null,
  setidletimeout: null,
  setworldspawn: null,
  spawnpoint: null,
  spectate: null,
  spreadplayers: null,
  stop: null,
  stopsound: null,
  summon: null,
  tag: {
    add: null,
    list: null,
    remove: null,
  },
  team: {
    add: null,
    empty: null,
    join: null,
    leave: null,
    list: null,
    modify: null,
    remove: null,
  },
  teleport: {
    [DYNAMIC_ARG]: {
      [DYNAMIC_ARG]: {
        [DYNAMIC_ARG]: {
          [DYNAMIC_ARG]: null,
        },
      },
    },
  },
  tp: {
    [DYNAMIC_ARG]: {
      [DYNAMIC_ARG]: {
        [DYNAMIC_ARG]: null,
      },
    },
  },
  trigger: null,
  weather: {
    clear: {
      [DYNAMIC_ARG]: null,
    },
    rain: {
      [DYNAMIC_ARG]: null,
    },
    thunder: {
      [DYNAMIC_ARG]: null,
    },
  },
  whitelist: {
    add: null,
    list: null,
    off: null,
    on: null,
    reload: null,
    remove: null,
  },
  worldborder: {
    add: null,
    center: null,
    damage: {
      amount: null,
      buffer: null,
    },
    get: null,
    set: null,
    warning: {
      distance: null,
      time: null,
    },
  },
  xp: null,
};

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
const suggestions = ref<string[]>([]);
const selectedSuggestionIndex = ref(0);

let socket: WebSocket | null = null;

const route = useRoute();
const serverId = route.params.id as string;

const serverData = computed(() => serverStore.serverData[serverId]);
const serverIP = computed(() => serverData.value?.net.ip ?? "");
const serverPort = computed(() => serverData.value?.net.port ?? "");
const serverDomain = computed(() => serverData.value?.net.domain ?? "");
const isServerRunning = computed(() => serverPowerState.value === "running");

useHead({
  title: `Overview - ${serverData.value?.name ?? "Server"} - Modrinth`,
});

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

const bestSuggestion = computed(() => {
  if (!suggestions.value.length) return "";
  const inputTokens = commandInput.value.trim().split(/\s+/);
  const lastInputToken = inputTokens[inputTokens.length - 1] || "";
  const firstSuggestion = suggestions.value[0];
  const suggestionTokens = firstSuggestion.split(/\s+/);
  const lastSuggestionToken = suggestionTokens[suggestionTokens.length - 1] || "";
  if (lastSuggestionToken.toLowerCase().startsWith(lastInputToken.toLowerCase())) {
    return lastSuggestionToken.slice(lastInputToken.length);
  }
  return "";
});

const getSuggestions = (input: string): string[] => {
  const trimmedInput = input.trim();
  const inputWithoutSlash = trimmedInput.startsWith("/") ? trimmedInput.slice(1) : trimmedInput;
  const tokens = inputWithoutSlash.split(/\s+/);
  let currentLevel: any = commandTree;

  for (let i = 0; i < tokens.length; i++) {
    const token = tokens[i].toLowerCase();
    if (currentLevel?.[token]) {
      currentLevel = currentLevel[token] as any;
    } else if (currentLevel?.[DYNAMIC_ARG]) {
      currentLevel = currentLevel[DYNAMIC_ARG] as any;
    } else {
      if (i === tokens.length - 1) {
        break;
      }
      currentLevel = null;
      break;
    }
  }

  if (currentLevel) {
    const lastToken = tokens[tokens.length - 1]?.toLowerCase() || "";
    const possibleKeys = Object.keys(currentLevel);
    if (currentLevel[DYNAMIC_ARG]) {
      possibleKeys.push("<arg>");
    }
    return possibleKeys
      .filter((key) => key === "<arg>" || key.toLowerCase().startsWith(lastToken))
      .map((key) => {
        if (key === "<arg>") {
          return [...tokens.slice(0, -1), "<arg>"].join(" ");
        }
        const newTokens = [...tokens.slice(0, -1), key];
        return newTokens.join(" ");
      });
  }

  return [];
};

watch(
  () => commandInput.value,
  (newVal) => {
    const trimmed = newVal.trim();
    if (!trimmed) {
      suggestions.value = [];
      return;
    }
    suggestions.value = getSuggestions(newVal);
    selectedSuggestionIndex.value = 0;
  },
);

// Suggestion Selection Methods
const selectNextSuggestion = () => {
  if (suggestions.value.length === 0) return;
  selectedSuggestionIndex.value = (selectedSuggestionIndex.value + 1) % suggestions.value.length;
};

const selectPrevSuggestion = () => {
  if (suggestions.value.length === 0) return;
  selectedSuggestionIndex.value =
    (selectedSuggestionIndex.value - 1 + suggestions.value.length) % suggestions.value.length;
};

// Improved acceptSuggestion function
const acceptSuggestion = () => {
  if (suggestions.value.length === 0) return;
  const selected = suggestions.value[selectedSuggestionIndex.value];
  const currentTokens = commandInput.value.trim().split(" ");
  const suggestionTokens = selected.split(/\s+/).filter(Boolean);
  console.log(currentTokens, suggestionTokens);

  // check if last current token is in command tree if so just add to the end
  if (currentTokens[currentTokens.length - 1].toLowerCase() === suggestionTokens[0].toLowerCase()) {
    /* empty */
  } else {
    commandInput.value =
      commandInput.value +
      suggestionTokens[0].substring(currentTokens[currentTokens.length - 1].length);
    suggestions.value = getSuggestions(commandInput.value);
    selectedSuggestionIndex.value = 0;
  }
};

const selectSuggestion = (index: number) => {
  selectedSuggestionIndex.value = index;
  acceptSuggestion();
};

const sendCommand = async () => {
  const cmd = commandInput.value.trim();
  if (!socket || !cmd) return;
  try {
    await socket.send(JSON.stringify({ event: "command", cmd }));
    commandInput.value = "";
    suggestions.value = [];
    selectedSuggestionIndex.value = 0;
  } catch (error) {
    console.error("Error sending command:", error);
  }
};

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

const updatePowerState = (state: ServerState) => {
  serverPowerState.value = state;
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

const sendPowerAction = async (action: "restart" | "start" | "stop" | "kill") => {
  const actionName = action.charAt(0).toUpperCase() + action.slice(1);
  try {
    isActioning.value = true;
    await serverStore.sendPowerAction(serverId, actionName);
    notifySuccess(`${actionName}ing server`, `This may take a few moments.`);
  } catch (error) {
    console.error(`Error ${actionName}ing server:`, error);
    notifyError(`Error ${actionName}ing server`, "An error occurred while performing this action.");
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

onMounted(() => {
  connectWebSocket();
});

onBeforeUnmount(() => {
  socket?.close();
});
</script>
