<template>
  <div
    v-if="isConnected && !isWsAuthIncorrect"
    class="relative flex select-none flex-col gap-6"
    data-pyro-server-manager-root
  >
    <div
      v-if="inspectingError"
      data-pyro-servers-inspecting-error
      class="flex justify-between rounded-2xl border-2 border-solid border-red bg-bg-red p-4 font-semibold text-contrast"
    >
      <div class="flex w-full justify-between gap-2">
        <div v-if="inspectingError.analysis.problems.length" class="flex flex-row gap-4">
          <IssuesIcon class="hidden h-8 w-8 text-red sm:block" />

          <div class="flex flex-col gap-2">
            <div class="font-semibold">
              {{ serverData?.name }} shut down unexpectedly. We've automatically analyzed the logs
              and found the following problems:
            </div>

            <li
              v-for="problem in inspectingError.analysis.problems"
              :key="problem.message"
              class="list-none"
            >
              <h4 class="m-0 text-sm font-normal sm:text-lg sm:font-semibold">
                {{ problem.message }}
              </h4>
              <ul class="m-0 ml-6">
                <li v-for="solution in problem.solutions" :key="solution.message">
                  <span class="m-0 text-sm font-normal">{{ solution.message }}</span>
                </li>
              </ul>
            </li>
          </div>
        </div>
        <div v-else-if="props.serverPowerState === 'crashed'" class="flex flex-row gap-4">
          <IssuesIcon class="hidden h-8 w-8 text-red sm:block" />

          <div class="flex flex-col gap-2">
            <div class="font-semibold">{{ serverData?.name }} shut down unexpectedly.</div>
            <div class="font-normal">
              <template v-if="props.powerStateDetails?.oom_killed">
                The server stopped because it ran out of memory. There may be a memory leak caused
                by a mod or plugin, or you may need to upgrade your Modrinth Server.
              </template>
              <template v-else-if="props.powerStateDetails?.exit_code !== undefined">
                We could not automatically determine the specific cause of the crash, but your
                server exited with code
                {{ props.powerStateDetails.exit_code }}.
                {{
                  props.powerStateDetails.exit_code === 1
                    ? "There may be a mod or plugin causing the issue, or an issue with your server configuration."
                    : ""
                }}
              </template>
              <template v-else> We could not determine the specific cause of the crash. </template>
              <div class="mt-2">You can try restarting the server.</div>
            </div>
          </div>
        </div>
        <div v-else class="flex flex-row gap-4">
          <IssuesIcon class="hidden h-8 w-8 text-red sm:block" />

          <div class="flex flex-col gap-2">
            <div class="font-semibold">{{ serverData?.name }} shut down unexpectedly.</div>
            <div class="font-normal">
              We could not find any specific problems, but you can try restarting the server.
            </div>
          </div>
        </div>
        <ButtonStyled color="red" @click="clearError">
          <button>
            <XIcon />
          </button>
        </ButtonStyled>
      </div>
    </div>
    <div class="flex flex-col-reverse gap-6 md:flex-col">
      <UiServersServerStats :data="stats" />
      <div
        class="relative flex h-[700px] w-full flex-col gap-3 overflow-hidden rounded-2xl border border-divider bg-bg-raised p-4 transition-all duration-300 ease-in-out md:p-8"
      >
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-4">
            <h2 class="m-0 text-3xl font-extrabold text-contrast">Console</h2>

            <UiServersPanelServerStatus :state="serverPowerState" />
          </div>
        </div>
        <!-- <div class="flex flex-row items-center gap-2 text-sm font-medium">
          <InfoIcon class="hidden sm:block" />
          Click and drag to select lines, then CMD+C to copy
        </div> -->
        <UiServersPanelTerminal :full-screen="fullScreen">
          <div class="relative w-full px-4 pt-4">
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
                <span class="ml-[23.5px] whitespace-pre">{{
                  " ".repeat(commandInput.length - 1)
                }}</span>
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
              <div
                class="pointer-events-none absolute left-0 top-0 flex h-full w-full items-center"
              >
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
        </UiServersPanelTerminal>
      </div>
    </div>
  </div>
  <UiServersOverviewLoading v-else-if="!isConnected && !isWsAuthIncorrect" />
  <div v-else-if="isWsAuthIncorrect" class="flex flex-col">
    <h2>Could not connect to the server.</h2>
    <p>
      An error occurred while attempting to connect to your server. Please try refreshing the page.
      (WebSocket Authentication Failed)
    </p>
  </div>
  <div v-else class="flex flex-col">
    <h2>Could not connect to the server.</h2>
    <p>
      An error occurred while attempting to connect to your server. Please try refreshing the page.
      (No further information)
    </p>
  </div>
</template>

<script setup lang="ts">
import { TerminalSquareIcon, XIcon, IssuesIcon } from "@modrinth/assets";
import { ButtonStyled } from "@modrinth/ui";
import type { ServerState, Stats } from "~/types/servers";
import type { Server } from "~/composables/pyroServers";

type ServerProps = {
  socket: WebSocket | null;
  isConnected: boolean;
  isWsAuthIncorrect: boolean;
  stats: Stats;
  serverPowerState: ServerState;
  powerStateDetails?: {
    oom_killed?: boolean;
    exit_code?: number;
  };
  isServerRunning: boolean;
  server: Server<["general", "content", "backups", "network", "startup", "ws", "fs"]>;
};

const props = defineProps<ServerProps>();

interface ErrorData {
  id: string;
  name: string;
  type: string;
  version: string;
  title: string;
  analysis: {
    problems: Array<{
      message: string;
      counter: number;
      entry: {
        level: number;
        time: string | null;
        prefix: string;
        lines: Array<{ number: number; content: string }>;
      };
      solutions: Array<{ message: string }>;
    }>;
    information: Array<{
      message: string;
      counter: number;
      label: string;
      value: string;
      entry: {
        level: number;
        time: string | null;
        prefix: string;
        lines: Array<{ number: number; content: string }>;
      };
    }>;
  };
}

const inspectingError = ref<ErrorData | null>(null);

const inspectError = async () => {
  try {
    const log = await props.server.fs?.downloadFile("logs/latest.log");
    if (!log) return;

    // @ts-ignore
    const response = await $fetch(`https://api.mclo.gs/1/analyse`, {
      method: "POST",
      headers: {
        "Content-Type": "application/x-www-form-urlencoded",
      },
      body: new URLSearchParams({
        content: log,
      }),
    });

    // @ts-ignore
    if (response && response.analysis && Array.isArray(response.analysis.problems)) {
      inspectingError.value = response as ErrorData;
    } else {
      inspectingError.value = null;
    }
  } catch (error) {
    console.error("Failed to analyze logs:", error);
    inspectingError.value = null;
  }
};

const clearError = () => {
  inspectingError.value = null;
};

watch(
  () => props.serverPowerState,
  (newVal) => {
    if (newVal === "crashed" && !props.powerStateDetails?.oom_killed) {
      inspectError();
    } else {
      clearError();
    }
  },
);

if (props.serverPowerState === "crashed" && !props.powerStateDetails?.oom_killed) {
  inspectError();
}

const socket = ref(props.socket);

watch(props, (newAttrs) => {
  socket.value = newAttrs.socket;
});

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

const fullScreen = ref(false);
const commandInput = ref("");
const suggestions = ref<string[]>([]);
const selectedSuggestionIndex = ref(0);

const serverData = computed(() => props.server.general);
// const serverIP = computed(() => serverData.value?.net.ip ?? "");
// const serverPort = computed(() => serverData.value?.net.port ?? 0);
// const serverDomain = computed(() => serverData.value?.net.domain ?? "");

const suggestionsList = ref<HTMLUListElement | null>(null);

useHead({
  title: `Overview - ${serverData.value?.name ?? "Server"} - Modrinth`,
});

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
  if (!socket || !cmd) return;
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
    socket.value?.send(JSON.stringify({ event: "command", cmd }));
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
