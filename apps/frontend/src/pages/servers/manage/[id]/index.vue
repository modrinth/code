<template>
  <div
    v-if="isConnected && !isWsAuthIncorrect"
    class="relative flex select-none flex-col gap-6"
    data-pyro-servers-page="overview"
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
      <Stats :data="stats" />
      <div
        class="relative flex h-[700px] w-full flex-col gap-3 overflow-hidden rounded-2xl border border-divider bg-bg-raised p-4 transition-all duration-300 ease-in-out md:p-8"
      >
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-4">
            <h2 class="m-0 text-3xl font-extrabold text-contrast">Console</h2>
            <StatusPill :state="serverPowerState" />
          </div>
        </div>
        <Terminal :socket="socket" :is-server-running="isServerRunning" :full-screen="fullScreen" />
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
import { IssuesIcon, XIcon } from "@modrinth/assets";
import { ButtonStyled } from "@modrinth/ui";
import type { ServerState, Stats as StatsType } from "~/types/servers";
import type { Server } from "~/composables/pyroServers";
import Terminal from "~/components/ui/servers/overview/Terminal.vue";
import Stats from "~/components/ui/servers/overview/Stats.vue";
import StatusPill from "~/components/ui/servers/overview/StatusPill.vue";

type ServerProps = {
  socket: WebSocket | null;
  isConnected: boolean;
  isWsAuthIncorrect: boolean;
  stats: StatsType;
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

const fullScreen = ref(false);

const serverData = computed(() => props.server.general);

useHead({
  title: `Overview - ${serverData.value?.name ?? "Server"} - Modrinth`,
});
</script>
