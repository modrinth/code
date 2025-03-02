<template>
  <Logs :full-screen="fullScreen">
    <div data-pyro-servers-component="overview-terminal" class="relative w-full px-4 pt-4">
      <ul
        v-if="suggestions.length"
        id="command-suggestions"
        ref="suggestionsList"
        class="mt-1 max-h-60 w-full list-none overflow-auto rounded-md border border-divider bg-bg-raised p-0 shadow-lg"
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
          @mousemove="() => (selectedSuggestionIndex = index)"
        >
          {{ suggestion }}
        </li>
      </ul>
      <div class="relative flex items-center">
        <span
          v-if="bestSuggestion"
          class="pointer-events-none absolute left-[26px] transform select-none text-gray-400"
        >
          <span class="ml-[23.5px] whitespace-pre">{{ " ".repeat(commandInput.length - 1) }}</span>
          <span> {{ bestSuggestion }} </span>
          <button
            class="text pointer-events-auto ml-2 cursor-pointer rounded-md border-none bg-white text-sm focus:outline-none dark:bg-highlight"
            aria-label="Accept suggestion"
            style="transform: translateY(-1px)"
            @click="acceptSuggestion"
          >
            TAB
          </button>
        </span>
        <div class="pointer-events-none absolute left-0 top-0 flex h-full w-full items-center">
          <TerminalSquareIcon class="ml-3 h-5 w-5" />
        </div>
        <input
          v-if="isServerRunning"
          v-model="commandInput"
          type="text"
          placeholder="Send a command"
          class="w-full rounded-md !pl-10 pt-4 focus:border-none [&&]:border-[1px] [&&]:border-solid [&&]:border-bg-raised [&&]:bg-bg"
          aria-autocomplete="list"
          aria-controls="command-suggestions"
          spellcheck="false"
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
          class="w-full rounded-md !pl-10 focus:border-none [&&]:border-[1px] [&&]:border-solid [&&]:border-bg-raised [&&]:bg-bg"
        />
      </div>
    </div>
  </Logs>
</template>

<script setup lang="ts">
import { TerminalSquareIcon } from "@modrinth/assets";
import Logs from "./Logs.vue";

const props = defineProps<{
  socket: WebSocket | null;
  isServerRunning: boolean;
  fullScreen: boolean;
}>();

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

const commandInput = ref("");
const suggestions = ref<string[]>([]);
const selectedSuggestionIndex = ref(0);

const suggestionsList = ref<HTMLUListElement | null>(null);

const bestSuggestion = computed(() => {
  if (!suggestions.value.length) return "";
  const inputTokens = commandInput.value.trim().split(/\s+/);
  let lastInputToken = inputTokens[inputTokens.length - 1] || "";
  if (inputTokens.length - 1 === 0 && lastInputToken.startsWith("/")) {
    lastInputToken = lastInputToken.slice(1);
  }
  const selectedSuggestion = suggestions.value[selectedSuggestionIndex.value];
  const suggestionTokens = selectedSuggestion.split(/\s+/);
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
      .filter((k) => k !== lastToken.trim())
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

const sendCommand = () => {
  const cmd = commandInput.value.trim();
  if (!props.socket || !cmd) return;
  try {
    sendConsoleCommand(cmd);
    commandInput.value = "";
    suggestions.value = [];
    selectedSuggestionIndex.value = 0;
  } catch (error) {
    console.error("Error sending command:", error);
  }
};

const sendConsoleCommand = (cmd: string) => {
  try {
    props.socket?.send(JSON.stringify({ event: "command", cmd }));
  } catch (error) {
    console.error("Error sending command:", error);
  }
};

watch(
  () => selectedSuggestionIndex.value,
  (newVal) => {
    if (suggestionsList.value) {
      const selectedSuggestion = suggestionsList.value.querySelector(`#suggestion-${newVal}`);
      if (selectedSuggestion) {
        selectedSuggestion.scrollIntoView({ block: "nearest" });
      }
    }
  },
);

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

const selectNextSuggestion = () => {
  if (suggestions.value.length === 0) return;
  selectedSuggestionIndex.value = (selectedSuggestionIndex.value + 1) % suggestions.value.length;
};

const selectPrevSuggestion = () => {
  if (suggestions.value.length === 0) return;
  selectedSuggestionIndex.value =
    (selectedSuggestionIndex.value - 1 + suggestions.value.length) % suggestions.value.length;
};

const acceptSuggestion = () => {
  if (suggestions.value.filter((s) => s !== "<arg>").length === 0) return;
  const selected = suggestions.value[selectedSuggestionIndex.value];
  const currentTokens = commandInput.value.trim().split(" ");
  const suggestionTokens = selected.split(/\s+/).filter(Boolean);

  // check if last current token is in command tree if so just add to the end
  if (currentTokens[currentTokens.length - 1].toLowerCase() === suggestionTokens[0].toLowerCase()) {
    /* empty */
  } else {
    const offset =
      currentTokens.length - 1 === 0 && currentTokens[0].trim().startsWith("/") ? 1 : 0;
    commandInput.value =
      commandInput.value +
      suggestionTokens[suggestionTokens.length - 1].substring(
        currentTokens[currentTokens.length - 1].length - offset,
      ) +
      " ";
    suggestions.value = getSuggestions(commandInput.value);
    selectedSuggestionIndex.value = 0;
  }
};

const selectSuggestion = (index: number) => {
  selectedSuggestionIndex.value = index;
  acceptSuggestion();
};
</script>
