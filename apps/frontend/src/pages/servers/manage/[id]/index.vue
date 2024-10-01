<template>
  <div v-if="isConnected && !isWSAuthIncorrect" class="relative flex flex-col gap-6" data-pyro-server-manager-root>
    <div :class="[
      'duration-400 absolute left-0 right-0 top-0 w-full transition-all',
      fullScreen ? '-translate-y-4 scale-95 opacity-0' : 'opacity-100',
    ]">
      <UiServersServerStats :data="stats" />
    </div>
    <div :class="[
      'relative flex w-full flex-col gap-3 overflow-hidden rounded-2xl border border-divider bg-bg-raised p-8 transition-all duration-300 ease-in-out',
      fullScreen ? 'mt-0 h-[85vh]' : 'mt-[254px] h-[600px]',
    ]">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-4">
          <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">Console</h2>
          <UiServersPanelServerStatus :state="serverPowerState" />
        </div>
        <div class="ml-auto mr-2 flex gap-2">
          <UiServersPanelCopyIP :ip="serverIP" :port="serverPort" :subdomain="serverDomain" />
        </div>
        <UiServersPanelServerActionButton :is-online="isServerRunning" :is-actioning="isActioning"
          @action="sendPowerAction" />
      </div>
      <UiServersPanelTerminal :console-output="consoleOutput" :full-screen="fullScreen"
        @toggle-full-screen="toggleFullScreen">
        <div class="relative w-full px-2.5 pt-2">
          <ul v-if="suggestions.length" id="command-suggestions"
            class="z-20 mt-1 max-h-60 w-full list-none overflow-auto rounded-md border border-divider bg-bg-raised p-0 shadow-lg"
            role="listbox">
            <li v-for="(suggestion, index) in suggestions" :key="index" :id="'suggestion-' + index" role="option"
              :aria-selected="index === selectedSuggestionIndex" :class="[
                'cursor-pointer px-4 py-2',
                index === selectedSuggestionIndex ? 'bg-gray-200' : 'bg-white',
              ]" @click="selectSuggestion(index)">
              {{ suggestion }}
            </li>
          </ul>
          <div class="relative">
            <span v-if="bestSuggestion"
              class="pointer-events-none absolute left-4 top-1/2 z-20 -translate-y-1/2 transform select-none text-gray-400">
              {{ bestSuggestion }}
            </span>

            <input v-model="commandInput" type="text" placeholder="Send a command"
              class="z-50 w-full rounded-md border border-gray-300 bg-white p-2 pl-16 focus:outline-none focus:ring-2 focus:ring-blue-500"
              @keydown.tab.prevent="acceptSuggestion" @keydown.down.prevent="selectNextSuggestion"
              @keydown.up.prevent="selectPrevSuggestion" @keydown.enter.prevent="sendCommand" aria-autocomplete="list"
              aria-controls="command-suggestions" :aria-activedescendant="'suggestion-' + selectedSuggestionIndex" />

            <button
              class="text-blue-500 absolute left-4 top-1/2 z-10 -translate-y-1/2 transform cursor-pointer border-none bg-transparent focus:outline-none"
              @click="focusInput" aria-label="Focus input">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24"
                stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M12 5l7 7-7 7" />
              </svg>
            </button>

            <button
              class="text-blue-500 absolute right-3 top-1/2 z-10 -translate-y-1/2 transform cursor-pointer border-none bg-transparent focus:outline-none"
              @click="acceptSuggestion" aria-label="Accept suggestion">
              TAB
            </button>
          </div>
        </div>
      </UiServersPanelTerminal>
    </div>
  </div>
  <UiServersPanelOverviewLoading v-else-if="!isConnected && !isWSAuthIncorrect" />
  <UiServersPyroError v-else-if="isWSAuthIncorrect" title="WebSocket authentication failed"
    message="Indicative of a server misconfiguration. Please report this to support." />
  <UiServersPyroError v-else title="An error occurred"
    message="Something went wrong while attempting to connect to your server. Your data is safe, and we're working to resolve the issue." />
</template>

<script setup lang="ts">
import {
  ref,
  computed,
  watch,
  onMounted,
  onBeforeUnmount,
} from 'vue';
import { useRoute, useHead } from 'vue-router';
import type {
  ServerState,
  Stats,
  WSAuth,
  WSEvent,
} from '~/types/servers';
import UiServersServerStats from '~/components/UiServersServerStats.vue';
import UiServersPanelServerStatus from '~/components/UiServersPanelServerStatus.vue';
import UiServersPanelCopyIP from '~/components/UiServersPanelCopyIP.vue';
import UiServersPanelServerActionButton from '~/components/UiServersPanelServerActionButton.vue';
import UiServersPanelTerminal from '~/components/UiServersPanelTerminal.vue';
import UiServersPanelOverviewLoading from '~/components/UiServersPanelOverviewLoading.vue';
import UiServersPyroError from '~/components/UiServersPyroError.vue';
import { useServerStore } from '~/stores/serverStore';
import { useNuxtApp } from '#app';

// Symbols
const DYNAMIC_ARG = Symbol('DYNAMIC_ARG');

// Command Tree Type
interface CommandNode {
  [key: string]: CommandNode | string | null | symbol;
}

// Command Tree Definition
const commandTree: CommandNode = {
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

// Reactive References
const serverStore = useServerStore();
const app = useNuxtApp();
const fullScreen = ref(false);
const isConnected = ref(false);
const isWSAuthIncorrect = ref(false);
const consoleOutput = ref<string[]>([]);
const cpuData = ref<number[]>([]);
const ramData = ref<number[]>([]);
const isActioning = ref(false);
const serverPowerState = ref<ServerState>('stopped');
const commandInput = ref('');
const suggestions = ref<string[]>([]);
const selectedSuggestionIndex = ref(0);

// WebSocket Reference
let socket: WebSocket | null = null;

// Route and Server ID
const route = useRoute();
const serverId = route.params.id as string;

// Computed Properties for Server Data
const serverData = computed(() => serverStore.serverData[serverId]);
const serverIP = computed(() => serverData.value?.net.ip ?? '');
const serverPort = computed(() => serverData.value?.net.port ?? '');
const serverDomain = computed(() => serverData.value?.net.domain ?? '');
const isServerRunning = computed(() => serverPowerState.value === 'running');

// Head Configuration
useHead({
  title: `Overview - ${serverData.value?.name ?? 'Server'} - Modrinth`,
});

// Stats Initialization
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

// Computed for Best Suggestion
const bestSuggestion = computed(() => {
  if (!suggestions.value.length) return '';
  const inputTokens = commandInput.value.trim().split(/\s+/);
  const lastInputToken = inputTokens[inputTokens.length - 1] || '';
  const firstSuggestion = suggestions.value[0];
  const suggestionTokens = firstSuggestion.split(/\s+/);
  const lastSuggestionToken = suggestionTokens[suggestionTokens.length - 1] || '';
  if (lastSuggestionToken.toLowerCase().startsWith(lastInputToken.toLowerCase())) {
    return lastSuggestionToken.slice(lastInputToken.length);
  }
  return '';
});

// Improved Command Tree Navigation
const getSuggestions = (input: string): string[] => {
  const trimmedInput = input.trim();
  const inputWithoutSlash = trimmedInput.startsWith('/')
    ? trimmedInput.slice(1)
    : trimmedInput;
  const tokens = inputWithoutSlash.split(/\s+/);
  let currentLevel: CommandNode | null = commandTree;

  for (let i = 0; i < tokens.length; i++) {
    const token = tokens[i].toLowerCase();
    if (currentLevel?.[token]) {
      currentLevel = currentLevel[token] as CommandNode;
    } else if (currentLevel?.[DYNAMIC_ARG]) {
      currentLevel = currentLevel[DYNAMIC_ARG] as CommandNode;
    } else {
      if (i === tokens.length - 1) {
        break;
      }
      currentLevel = null;
      break;
    }
  }

  if (currentLevel) {
    const lastToken = tokens[tokens.length - 1]?.toLowerCase() || '';
    let possibleKeys = Object.keys(currentLevel);
    if (currentLevel[DYNAMIC_ARG]) {
      possibleKeys.push('<arg>');
    }
    return possibleKeys
      .filter((key) => key === '<arg>' || key.toLowerCase().startsWith(lastToken))
      .map((key) => {
        if (key === '<arg>') {
          return [...tokens.slice(0, -1), '<arg>'].join(' ');
        }
        const newTokens = [...tokens.slice(0, -1), key];
        return newTokens.join(' ');
      });
  }

  return [];
};

// Improved watcher for commandInput
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
  }
);

// Suggestion Selection Methods
const selectNextSuggestion = () => {
  if (suggestions.value.length === 0) return;
  selectedSuggestionIndex.value =
    (selectedSuggestionIndex.value + 1) % suggestions.value.length;
};

const selectPrevSuggestion = () => {
  if (suggestions.value.length === 0) return;
  selectedSuggestionIndex.value =
    (selectedSuggestionIndex.value - 1 + suggestions.value.length) %
    suggestions.value.length;
};

// Improved acceptSuggestion function
const acceptSuggestion = () => {
  if (!suggestions.value.length) return;
  const selected = suggestions.value[selectedSuggestionIndex.value];
  const currentTokens = commandInput.value.trim().split(/\s+/);
  const suggestionTokens = selected.split(/\s+/);

  currentTokens.pop();
  commandInput.value = [...currentTokens, ...suggestionTokens].join(' ') + ' ';

  suggestions.value = getSuggestions(commandInput.value);
  selectedSuggestionIndex.value = 0;
};

const selectSuggestion = (index: number) => {
  selectedSuggestionIndex.value = index;
  acceptSuggestion();
};

// Command Sending
const sendCommand = async () => {
  const cmd = commandInput.value.trim();
  if (!socket || !cmd) return;
  try {
    await socket.send(JSON.stringify({ event: 'command', cmd }));
    commandInput.value = '';
    suggestions.value = [];
    selectedSuggestionIndex.value = 0;
  } catch (error) {
    console.error('Error sending command:', error);
  }
};

// WebSocket Handling
const connectWebSocket = async () => {
  try {
    const wsAuth = (await serverStore.requestWebsocket(serverId)) as WSAuth;
    socket = new WebSocket(`wss://${wsAuth.url}`);

    socket.onopen = () => {
      socket?.send(JSON.stringify({ event: 'auth', jwt: wsAuth.token }));
    };

    socket.onmessage = (event) => {
      const data: WSEvent = JSON.parse(event.data);
      handleWebSocketMessage(data);
    };

    socket.onclose = () => {
      consoleOutput.value.push('\nWS connection closed');
      isConnected.value = false;
    };

    socket.onerror = (error) => {
      console.error('WebSocket error:', error);
      isConnected.value = false;
    };
  } catch (error) {
    console.error('Failed to connect WebSocket:', error);
    isConnected.value = false;
  }
};

// WebSocket Message Handler
const handleWebSocketMessage = (data: WSEvent) => {
  switch (data.event) {
    case 'log':
      consoleOutput.value.push(data.message);
      break;
    case 'stats':
      updateStats(data as Stats['current']);
      break;
    case 'auth-expiring':
      reauthenticate();
      break;
    case 'power-state':
      updatePowerState(data.state);
      break;
    case 'auth-incorrect':
      isWSAuthIncorrect.value = true;
      break;
    default:
      console.warn('Unhandled WebSocket event:', data);
  }
};

// Update Server Power State
const updatePowerState = (state: ServerState) => {
  serverPowerState.value = state;
};

// Update Stats Data
const updateStats = (currentStats: Stats['current']) => {
  isConnected.value = true;
  stats.value = {
    current: currentStats,
    past: { ...stats.value.current },
    graph: {
      cpu: updateGraphData(cpuData, currentStats.cpu_percent),
      ram: updateGraphData(
        ramData,
        Math.floor((currentStats.ram_usage_bytes / currentStats.ram_total_bytes) * 100)
      ),
    },
  };
};

// Update Graph Data Helper
const updateGraphData = (dataArray: number[], newValue: number): number[] => {
  const updated = [...dataArray, newValue];
  if (updated.length > 10) updated.shift();
  return updated;
};

// Reauthenticate WebSocket
const reauthenticate = async () => {
  try {
    const wsAuth = (await serverStore.requestWebsocket(serverId)) as WSAuth;
    socket?.send(JSON.stringify({ event: 'auth', jwt: wsAuth.token }));
  } catch (error) {
    console.error('Reauthentication failed:', error);
    isWSAuthIncorrect.value = true;
  }
};

// Power Action Handling
const sendPowerAction = async (action: 'restart' | 'start' | 'stop' | 'kill') => {
  const actionName = action.charAt(0).toUpperCase() + action.slice(1);
  try {
    isActioning.value = true;
    await serverStore.sendPowerAction(serverId, actionName);
    notifySuccess(`${actionName}ing server`, `Your server is now ${actionName.toLowerCase()}ing. This may take a few moments.`);
  } catch (error) {
    console.error(`Error ${actionName}ing server:`, error);
    notifyError(`Error ${actionName}ing server`, 'An error occurred while attempting to perform the action.');
  } finally {
    isActioning.value = false;
  }
};

// Notification Helpers
const notifySuccess = (title: string, text: string) => {
  // @ts-ignore
  app.$notify({
    group: 'server',
    title,
    text,
    type: 'success',
  });
};

const notifyError = (title: string, text: string) => {
  // @ts-ignore
  app.$notify({
    group: 'server',
    title,
    text,
    type: 'error',
  });
};

// Focus Input Field
const focusInput = () => {
  const inputElement = document.querySelector<HTMLInputElement>('input[placeholder="Send a command"]');
  inputElement?.focus();
};

// Lifecycle Hooks
onMounted(() => {
  connectWebSocket();
});

onBeforeUnmount(() => {
  socket?.close();
});
</script>