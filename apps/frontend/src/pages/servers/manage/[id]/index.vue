<template>
  <div
    v-if="isConnected && !isWsAuthIncorrect"
    class="relative flex select-none flex-col gap-6"
    data-pyro-servers-page="overview"
  >
    <div
      v-if="inspectingError"
      data-pyro-servers-inspecting-error
      class="relative flex justify-between rounded-3xl bg-bg-red p-4"
    >
      <ButtonStyled type="transparent" circular @click="clearError">
        <button class="absolute right-4 top-4">
          <XIcon />
        </button>
      </ButtonStyled>
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

        <div v-else-if="props.serverPowerState === 'crashed'" class="flex w-full flex-row gap-4">
          <div class="flex w-full flex-col gap-4">
            <div class="flex flex-row gap-4">
              <IssuesIcon class="hidden min-h-8 min-w-8 text-red sm:block" />

              <div class="flex flex-col gap-2">
                <div class="text-xl font-semibold text-contrast">
                  {{ serverData?.name }} shut down unexpectedly
                </div>
                <template v-if="props.powerStateDetails?.oom_killed">
                  The server stopped because it ran out of memory. There may be a memory leak caused
                  by a mod or plugin, or you may need to upgrade your Modrinth Server.
                </template>
                <template v-else-if="props.powerStateDetails?.exit_code !== undefined">
                  <div class="w-full">
                    <div class="leading-[190%]">
                      The server crashed with exit code {{ props.powerStateDetails.exit_code }}.
                      We've gathered some steps to help you troubleshoot the issue.
                    </div>
                  </div>
                </template>
                <template v-else>
                  We could not determine the specific cause of the crash.
                </template>
              </div>
            </div>
            <div class="flex w-full flex-col gap-4 lg:flex-row">
              <div class="flex w-full flex-col gap-2 rounded-2xl bg-bg p-4">
                <div class="flex flex-row items-center gap-2 font-semibold text-contrast">
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="24"
                    height="24"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    class="lucide lucide-shapes"
                  >
                    <path
                      d="M8.3 10a.7.7 0 0 1-.626-1.079L11.4 3a.7.7 0 0 1 1.198-.043L16.3 8.9a.7.7 0 0 1-.572 1.1Z"
                    />
                    <rect x="3" y="14" width="7" height="7" rx="1" />
                    <circle cx="17.5" cy="17.5" r="3.5" />
                  </svg>
                  Check your server's content
                </div>
                <div class="text-sm">
                  Disable any mods or plugins that you don't need, or any that are only needed on
                  the client.
                </div>
                <ButtonStyled>
                  <NuxtLink
                    class="mt-2 !w-full whitespace-pre"
                    :to="`/servers/manage/${props.server.serverId}/content`"
                  >
                    Manage server content
                  </NuxtLink>
                </ButtonStyled>
              </div>
              <div class="flex w-full flex-col gap-2 rounded-2xl bg-bg p-4">
                <div class="flex flex-row items-center gap-2 font-semibold text-contrast">
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="24"
                    height="24"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    class="lucide lucide-logs"
                  >
                    <path d="M13 12h8" />
                    <path d="M13 18h8" />
                    <path d="M13 6h8" />
                    <path d="M3 12h1" />
                    <path d="M3 18h1" />
                    <path d="M3 6h1" />
                    <path d="M8 12h1" />
                    <path d="M8 18h1" />
                    <path d="M8 6h1" />
                  </svg>
                  Read your server's logs
                </div>
                <div class="text-sm">
                  Check the logs for errors or warnings. Consider any recent changes made to your
                  server's configuration.
                </div>
                <ButtonStyled @click="fullScreen = !fullScreen">
                  <button class="mt-2 !w-full whitespace-pre">View logs</button>
                </ButtonStyled>
              </div>
              <div class="flex w-full flex-col gap-2 rounded-2xl bg-bg p-4">
                <div class="flex flex-row items-center gap-2 font-semibold text-contrast">
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="24"
                    height="24"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    class="lucide lucide-megaphone"
                  >
                    <path d="m3 11 18-5v12L3 14v-3z" />
                    <path d="M11.6 16.8a3 3 0 1 1-5.8-1.6" />
                  </svg>
                  Report the issue
                </div>
                <div class="text-sm">
                  Your server may be configured incorrectly. Report the issue to the mod or plugin's
                  creators.
                </div>
                <template v-if="props.server.general?.project">
                  <ButtonStyled>
                    <a
                      class="mt-2 !w-full whitespace-pre"
                      :href="`https://modrinth.com/modpack/${props.server.general.project.id}`"
                      target="_blank"
                      rel="noopener noreferrer"
                    >
                      Report to {{ props.server.general.project.title }}
                    </a>
                  </ButtonStyled>
                </template>
              </div>
            </div>
            <div class="flex flex-row gap-2">
              <InfoIcon />
              If you believe this is an issue with the Modrinth Servers platform, please contact
              support using the chat bubble in the bottom right.
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
  <OverviewSkeleton v-else-if="!isConnected && !isWsAuthIncorrect" />
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
import { InfoIcon, IssuesIcon, XIcon } from "@modrinth/assets";
import { ButtonStyled } from "@modrinth/ui";
import type { ServerState, Stats as StatsType } from "~/types/servers";
import type { Server } from "~/composables/pyroServers";
import Terminal from "~/components/ui/servers/overview/Terminal.vue";
import Stats from "~/components/ui/servers/overview/Stats.vue";
import StatusPill from "~/components/ui/servers/overview/StatusPill.vue";
import OverviewSkeleton from "~/components/ui/servers/overview/OverviewSkeleton.vue";

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
