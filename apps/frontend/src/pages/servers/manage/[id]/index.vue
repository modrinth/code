<template>
  <div
    v-if="isConnected && !isWSAuthIncorrect"
    class="relative flex flex-col gap-6"
    data-pyro-server-manager-root
  >
    <div
      :class="[
        'duration-400 absolute left-0 right-0 top-0 w-full transition-all',
        fullScreen ? '-translate-y-4 scale-95 opacity-0' : 'opacity-100',
      ]"
    >
      <UiServersServerStats :data="stats" />
    </div>
    <div
      :class="[
        'relative flex w-full flex-col gap-3 overflow-hidden rounded-2xl border border-divider bg-bg-raised p-8 transition-all duration-300 ease-in-out',
        fullScreen ? 'mt-0 h-[85vh]' : 'mt-[254px] h-[600px]',
      ]"
    >
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-4">
          <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">Console</h2>
          <UiServersPanelServerStatus :state="serverPowerState" />
        </div>
        <div class="ml-auto mr-2 flex gap-2">
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
          <ul
            v-if="topThreeSuggestions.length > 0"
            id="command-suggestions"
            class="z-20 mt-1 max-h-60 w-full list-none overflow-auto rounded-md border border-divider bg-bg-raised p-0 shadow-lg"
          >
            <li
              v-for="(suggestion, index) in topThreeSuggestions"
              :key="index"
              :id="'suggestion-' + index"
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
              class="pointer-events-none absolute left-[1.7rem] top-[48%] z-20 -translate-y-1/2 transform select-none text-gray-400"
            >
              {{ ".".repeat(commandInput.length - 1) }}{{ bestSuggestion }}
            </span>

            <input
              v-model="commandInput"
              type="text"
              placeholder="Send a command"
              class="z-50 w-full rounded-md p-2 pt-4 focus:border-none [&&]:border-[1px] [&&]:border-solid [&&]:border-bg-raised [&&]:bg-bg"
              @keydown.tab.prevent="acceptSuggestion"
              @keydown.down.prevent="selectNextSuggestion"
              @keydown.up.prevent="selectPrevSuggestion"
              @keydown.enter.prevent="sendCommand"
              aria-autocomplete="list"
              aria-controls="command-suggestions"
              aria-activedescendant="suggestion-{{ selectedSuggestionIndex }}"
            />

            <button
              class="text-blue-500 absolute right-3 top-1/2 z-10 -translate-y-1/2 transform cursor-pointer border-none bg-transparent focus:outline-none"
              @click="acceptSuggestion"
              aria-label="Accept suggestion"
            >
              TAB
            </button>
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
const topThreeSuggestions = computed(() => filteredSuggestions.value.slice(0, 3).reverse());

const bestSuggestion = computed(() => {
  if (filteredSuggestions.value.length > 0) {
    const inputLength = commandInput.value.length;
    const suggestion = filteredSuggestions.value[0];

    return suggestion.length > inputLength ? suggestion.slice(inputLength) : "";
  }
  return "";
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

const route = useRoute();
const serverId = route.params.id as string;

useHead({
  title: `Overview - ${serverStore.serverData[serverId]?.name ?? "Server"} - Modrinth`,
});

let socket: WebSocket | null = null;
const toggleFullScreen = () => {
  fullScreen.value = !fullScreen.value;
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

interface CommandNode {
  [key: string]: CommandNode | null | string;
}

const DYNAMIC_ARG = Symbol("DYNAMIC_ARG");
const commandTree: CommandNode = {
  advancement: {
    grant: {
      [DYNAMIC_ARG as any]: {
        everything: null,
        only: {
          [DYNAMIC_ARG as any]: null,
        },
        from: {
          [DYNAMIC_ARG as any]: null,
        },
        through: {
          [DYNAMIC_ARG as any]: null,
        },
        until: {
          [DYNAMIC_ARG as any]: null,
        },
      },
    },
    revoke: {
      [DYNAMIC_ARG as any]: {
        everything: null,
        only: {
          [DYNAMIC_ARG as any]: null,
        },
        from: {
          [DYNAMIC_ARG as any]: null,
        },
        through: {
          [DYNAMIC_ARG as any]: null,
        },
        until: {
          [DYNAMIC_ARG as any]: null,
        },
      },
    },
  },
  ban: {
    [DYNAMIC_ARG as any]: {
      [DYNAMIC_ARG as any]: null,
      duration: {
        [DYNAMIC_ARG as any]: null,
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
    [DYNAMIC_ARG as any]: {
      [DYNAMIC_ARG as any]: {
        [DYNAMIC_ARG as any]: null,
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
      [DYNAMIC_ARG as any]: {
        [DYNAMIC_ARG as any]: {
          [DYNAMIC_ARG as any]: {
            [DYNAMIC_ARG as any]: {
              true: null,
              false: null,
            },
          },
        },
      },
    },
    clear: {
      [DYNAMIC_ARG as any]: {
        [DYNAMIC_ARG as any]: null,
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
      [DYNAMIC_ARG as any]: null,
    },
    creative: {
      [DYNAMIC_ARG as any]: null,
    },
    adventure: {
      [DYNAMIC_ARG as any]: null,
    },
    spectator: {
      [DYNAMIC_ARG as any]: null,
    },
  },
  gamerule: null,
  give: {
    [DYNAMIC_ARG as any]: {
      [DYNAMIC_ARG as any]: {
        [DYNAMIC_ARG as any]: null,
      },
    },
  },
  help: null,
  kick: null,
  kill: {
    [DYNAMIC_ARG as any]: null,
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
    [DYNAMIC_ARG as any]: {
      [DYNAMIC_ARG as any]: {
        [DYNAMIC_ARG as any]: {
          [DYNAMIC_ARG as any]: null,
        },
      },
    },
  },
  tell: null,
  tellraw: null,
  time: {
    set: {
      day: null,
      night: null,
      noon: null,
      midnight: null,
      [DYNAMIC_ARG as any]: null,
    },
    add: {
      [DYNAMIC_ARG as any]: null,
    },
    query: {
      daytime: null,
      gametime: null,
      day: null,
    },
  },
  title: {
    clear: null,
    reset: null,
    set: {
      title: null,
      subtitle: null,
      actionbar: null,
    },
    times: null,
  },
  tp: null,
  trigger: null,
  weather: {
    clear: {
      [DYNAMIC_ARG as any]: null,
    },
    rain: {
      [DYNAMIC_ARG as any]: null,
    },
    thunder: {
      [DYNAMIC_ARG as any]: null,
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

const getSuggestions = (input: string): string[] => {
  const trimmedInput = input.trim();
  const hasSlash = trimmedInput.startsWith("/");
  const inputWithoutSlash = hasSlash ? trimmedInput.slice(1) : trimmedInput;
  const tokens = inputWithoutSlash.split(/\s+/);
  let currentLevel: CommandNode | null = commandTree;
  for (let i = 0; i < tokens.length - 1; i++) {
    const token = tokens[i];
    if (currentLevel && currentLevel[token]) {
      currentLevel = currentLevel[token] as CommandNode;
    } else if (currentLevel && currentLevel[DYNAMIC_ARG as any]) {
      currentLevel = currentLevel[DYNAMIC_ARG as any] as CommandNode;
    } else {
      currentLevel = null;
      break;
    }
  }
  if (currentLevel) {
    return Object.keys(currentLevel)
      .filter((key) => key !== DYNAMIC_ARG.toString())
      .map((key) => tokens.slice(0, -1).concat(key).join(" "));
  } else {
    return Object.keys(commandTree);
  }
};

const filteredSuggestions = computed(() => {
  const input = commandInput.value.trim();
  const lastToken = input.split(/\s+/).pop() || "";
  return suggestions.value.filter((suggestion) =>
    suggestion.toLowerCase().startsWith(lastToken.toLowerCase()),
  );
});

watch(
  () => commandInput.value,
  (newVal) => {
    if (newVal.trim() === "") {
      suggestions.value = [];
    } else {
      suggestions.value = getSuggestions(newVal);
      selectedSuggestionIndex.value = 0;
    }
  },
);

const selectNextSuggestion = () => {
  if (selectedSuggestionIndex.value < topThreeSuggestions.value.length - 1) {
    selectedSuggestionIndex.value++;
  } else {
    selectedSuggestionIndex.value = 0;
  }
};

const selectPrevSuggestion = () => {
  if (selectedSuggestionIndex.value > 0) {
    selectedSuggestionIndex.value--;
  } else {
    selectedSuggestionIndex.value = topThreeSuggestions.value.length - 1;
  }
};

const acceptSuggestion = () => {
  if (topThreeSuggestions.value.length > 0) {
    const selectedSuggestion = topThreeSuggestions.value[selectedSuggestionIndex.value];
    // Replace the last token with the suggestion
    const tokens = commandInput.value.trim().split(/\s+/);
    tokens[tokens.length - 1] = selectedSuggestion.split(/\s+/).pop() || "";
    commandInput.value = tokens.join(" ") + " ";
    suggestions.value = getSuggestions(commandInput.value);
    selectedSuggestionIndex.value = 0;
  }
};

const selectSuggestion = (index: number) => {
  selectedSuggestionIndex.value = index;
  acceptSuggestion();
};

const sendCommand = async () => {
  if (!socket || commandInput.value.trim() === "") return;
  await socket.send(JSON.stringify({ event: "command", cmd: commandInput.value }));
  commandInput.value = "";
  suggestions.value = [];
  selectedSuggestionIndex.value = 0;
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
