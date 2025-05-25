<template>
  <div
    v-if="filteredNotices.length > 0"
    class="experimental-styles-within relative mx-auto flex w-full min-w-0 max-w-[1280px] flex-col gap-3 px-6"
  >
    <ServerNotice
      v-for="notice in filteredNotices"
      :key="`notice-${notice.id}`"
      :level="notice.level"
      :message="notice.message"
      :dismissable="notice.dismissable"
      :title="notice.title"
      class="w-full"
      @dismiss="() => dismissNotice(notice.id)"
    />
  </div>
  <div
    v-if="serverData?.status === 'suspended' && serverData.suspension_reason === 'upgrading'"
    class="flex min-h-[calc(100vh-4rem)] items-center justify-center text-contrast"
  >
    <div class="flex max-w-lg flex-col items-center rounded-3xl bg-bg-raised p-6 shadow-xl">
      <div class="flex flex-col items-center text-center">
        <div class="flex flex-col items-center gap-4">
          <div class="grid place-content-center rounded-full bg-bg-blue p-4">
            <TransferIcon class="size-12 text-blue" />
          </div>
          <h1 class="m-0 mb-2 w-fit text-4xl font-bold">Server upgrading</h1>
        </div>
        <p class="text-lg text-secondary">
          Your server's hardware is currently being upgraded and will be back online shortly!
        </p>
      </div>
    </div>
  </div>
  <div
    v-else-if="serverData?.status === 'suspended'"
    class="flex min-h-[calc(100vh-4rem)] items-center justify-center text-contrast"
  >
    <div class="flex max-w-lg flex-col items-center rounded-3xl bg-bg-raised p-6 shadow-xl">
      <div class="flex flex-col items-center text-center">
        <div class="flex flex-col items-center gap-4">
          <div class="grid place-content-center rounded-full bg-bg-orange p-4">
            <LockIcon class="size-12 text-orange" />
          </div>
          <h1 class="m-0 mb-2 w-fit text-4xl font-bold">Server suspended</h1>
        </div>
        <p class="text-lg text-secondary">
          {{
            serverData.suspension_reason === "cancelled"
              ? "Your subscription has been cancelled."
              : serverData.suspension_reason
                ? `Your server has been suspended: ${serverData.suspension_reason}`
                : "Your server has been suspended."
          }}
          <br />
          Contact Modrinth Support if you believe this is an error.
        </p>
      </div>
      <ButtonStyled size="large" color="brand" @click="() => router.push('/settings/billing')">
        <button class="mt-6 !w-full">Go to billing settings</button>
      </ButtonStyled>
    </div>
  </div>
  <div
    v-else-if="
      server.general?.error?.error.statusCode === 403 ||
      server.general?.error?.error.statusCode === 404
    "
    class="flex min-h-[calc(100vh-4rem)] items-center justify-center text-contrast"
  >
    <div class="flex max-w-lg flex-col items-center rounded-3xl bg-bg-raised p-6 shadow-xl">
      <div class="flex flex-col items-center text-center">
        <div class="flex flex-col items-center gap-4">
          <div class="grid place-content-center rounded-full bg-bg-orange p-4">
            <TransferIcon class="size-12 text-orange" />
          </div>
          <h1 class="m-0 mb-2 w-fit text-4xl font-bold">Server not found</h1>
        </div>
        <p class="text-lg text-secondary">
          You don't have permission to view this server or it no longer exists. If you believe this
          is an error, please contact Modrinth Support.
        </p>
      </div>
      <UiCopyCode :text="JSON.stringify(server.general?.error)" />

      <ButtonStyled size="large" color="brand" @click="() => router.push('/servers/manage')">
        <button class="mt-6 !w-full">Go back to all servers</button>
      </ButtonStyled>
    </div>
  </div>
  <div
    v-else-if="server.general?.error?.error.statusCode === 503"
    class="flex min-h-[calc(100vh-4rem)] items-center justify-center text-contrast"
  >
    <div class="flex max-w-lg flex-col items-center rounded-3xl bg-bg-raised p-6 shadow-xl">
      <div class="flex flex-col items-center text-center">
        <div class="flex flex-col items-center gap-4">
          <div class="grid place-content-center rounded-full bg-bg-red p-4">
            <UiServersIconsPanelErrorIcon class="size-12 text-red" />
          </div>
          <h1 class="m-0 mb-4 w-fit text-4xl font-bold">Server Node Unavailable</h1>
        </div>
        <p class="m-0 mb-4 leading-[170%] text-secondary">
          Your server's node, where your Modrinth Server is physically hosted, is experiencing
          issues. We are working with our datacenter to resolve the issue as quickly as possible.
        </p>
        <p class="m-0 mb-4 leading-[170%] text-secondary">
          Your data is safe and will not be lost, and your server will be back online as soon as the
          issue is resolved.
        </p>
        <p class="m-0 mb-4 leading-[170%] text-secondary">
          For updates, please join the Modrinth Discord or contact Modrinth Support via the chat
          bubble in the bottom right corner and we'll be happy to help.
        </p>

        <div class="flex flex-col gap-2">
          <UiCopyCode :text="'Server ID: ' + server.serverId" />
          <UiCopyCode :text="'Node: ' + server.general?.datacenter" />
        </div>
      </div>
      <ButtonStyled
        size="large"
        color="standard"
        @click="
          () =>
            navigateTo('https://discord.modrinth.com', {
              external: true,
            })
        "
      >
        <button class="mt-6 !w-full">Join Modrinth Discord</button>
      </ButtonStyled>
      <ButtonStyled
        :disabled="formattedTime !== '00'"
        size="large"
        color="standard"
        @click="() => reloadNuxtApp()"
      >
        <button class="mt-3 !w-full">Reload</button>
      </ButtonStyled>
    </div>
  </div>
  <div
    v-else-if="server.general?.error"
    class="flex min-h-[calc(100vh-4rem)] items-center justify-center text-contrast"
  >
    <div class="flex max-w-lg flex-col items-center rounded-3xl bg-bg-raised p-6 shadow-xl">
      <div class="flex flex-col items-center text-center">
        <div class="flex flex-col items-center gap-4">
          <div class="grid place-content-center rounded-full bg-bg-orange p-4">
            <TransferIcon class="size-12 text-orange" />
          </div>
          <h1 class="m-0 mb-2 w-fit text-4xl font-bold">Connection lost</h1>
          <div class="text-center text-secondary">
            {{
              formattedTime == "00" ? "Reconnecting..." : `Retrying in ${formattedTime} seconds...`
            }}
          </div>
        </div>
        <p class="text-lg text-secondary">
          Something went wrong, and we couldn't connect to your server. This is likely due to a
          temporary network issue. You'll be reconnected automatically.
        </p>
      </div>
      <UiCopyCode :text="JSON.stringify(server.general?.error)" />
      <ButtonStyled
        :disabled="formattedTime !== '00'"
        size="large"
        color="brand"
        @click="() => reloadNuxtApp()"
      >
        <button class="mt-6 !w-full">Reload</button>
      </ButtonStyled>
    </div>
  </div>
  <!-- SERVER START -->
  <div
    v-else-if="serverData"
    data-pyro-server-manager-root
    class="experimental-styles-within mobile-blurred-servericon relative mx-auto mb-6 box-border flex min-h-screen w-full min-w-0 max-w-[1280px] flex-col gap-6 px-6 transition-all duration-300"
    :style="{
      '--server-bg-image': serverData.image
        ? `url(${serverData.image})`
        : `linear-gradient(180deg, rgba(153,153,153,1) 0%, rgba(87,87,87,1) 100%)`,
    }"
  >
    <div class="flex w-full min-w-0 select-none flex-col items-center gap-6 pt-4 sm:flex-row">
      <UiServersServerIcon :image="serverData.image" class="drop-shadow-lg sm:drop-shadow-none" />
      <div
        class="flex min-w-0 flex-1 flex-col-reverse items-center gap-2 sm:flex-col sm:items-start"
      >
        <div class="hidden shrink-0 flex-row items-center gap-1 sm:flex">
          <NuxtLink to="/servers/manage" class="breadcrumb goto-link flex w-fit items-center">
            <LeftArrowIcon />
            All servers
          </NuxtLink>
        </div>
        <div class="flex w-full flex-col items-center gap-4 sm:flex-row">
          <h1
            class="m-0 w-screen flex-shrink gap-3 truncate px-3 text-center text-4xl font-bold text-contrast sm:w-full sm:p-0 sm:text-left"
          >
            {{ serverData.name }}
          </h1>
          <div
            v-if="isConnected"
            data-pyro-server-action-buttons
            class="server-action-buttons-anim flex w-fit flex-shrink-0"
          >
            <UiServersPanelServerActionButton
              class="flex-shrink-0"
              :is-online="isServerRunning"
              :is-actioning="isActioning"
              :is-installing="serverData.status === 'installing'"
              :disabled="isActioning || !!error"
              :server-name="serverData.name"
              :server-data="serverData"
              :uptime-seconds="uptimeSeconds"
              @action="sendPowerAction"
            />
          </div>
        </div>

        <UiServersServerInfoLabels
          :server-data="serverData"
          :show-game-label="showGameLabel"
          :show-loader-label="showLoaderLabel"
          :uptime-seconds="uptimeSeconds"
          :linked="true"
          class="server-action-buttons-anim flex min-w-0 flex-col flex-wrap items-center gap-4 text-secondary *:hidden sm:flex-row sm:*:flex"
        />
      </div>
    </div>

    <div
      data-pyro-navigation
      class="isolate flex w-full select-none flex-col justify-between gap-4 overflow-auto md:flex-row md:items-center"
    >
      <UiNavTabs :links="navLinks" />
    </div>

    <div data-pyro-mount class="h-full w-full flex-1">
      <div
        v-if="error"
        class="mx-auto mb-4 flex justify-between gap-2 rounded-2xl border-2 border-solid border-red bg-bg-red p-4 font-semibold text-contrast"
      >
        <div class="flex flex-row gap-4">
          <IssuesIcon class="hidden h-8 w-8 shrink-0 text-red sm:block" />
          <div class="flex flex-col gap-2 leading-[150%]">
            <div class="flex items-center gap-3">
              <IssuesIcon class="flex h-8 w-8 shrink-0 text-red sm:hidden" />
              <div class="flex gap-2 text-2xl font-bold">{{ errorTitle }}</div>
            </div>

            <div v-if="errorTitle.toLocaleLowerCase() === 'installation error'" class="font-normal">
              <div
                v-if="errorMessage.toLocaleLowerCase() === 'the specified version may be incorrect'"
              >
                An invalid loader or Minecraft version was specified and could not be installed.
                <ul class="m-0 mt-4 p-0 pl-4">
                  <li>
                    If this version of Minecraft was released recently, please check if Modrinth
                    Servers supports it.
                  </li>
                  <li>
                    If you've installed a modpack, it may have been packaged incorrectly or may not
                    be compatible with the loader.
                  </li>
                  <li>
                    Your server may need to be reinstalled with a valid mod loader and version. You
                    can change the loader by clicking the "Change Loader" button.
                  </li>
                  <li>
                    If you're stuck, please contact Modrinth Support with the information below:
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
              <div v-if="errorMessage.toLocaleLowerCase() === 'this version is not yet supported'">
                An error occurred while installing your server because Modrinth Servers does not
                support the version of Minecraft or the loader you specified. Try reinstalling your
                server with a different version or loader, and if the problem persists, please
                contact Modrinth Support with your server's debug information.
              </div>

              <div
                v-if="errorTitle === 'Installation error'"
                class="mt-2 flex flex-col gap-4 sm:flex-row"
              >
                <ButtonStyled v-if="errorLog">
                  <button @click="openInstallLog"><FileIcon />Open Installation Log</button>
                </ButtonStyled>
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
        class="mb-4 flex w-full flex-row items-center gap-4 rounded-2xl bg-bg-orange p-4 text-sm text-contrast"
      >
        <UiServersPanelSpinner />
        Hang on, we're reconnecting to your server.
      </div>

      <div
        v-if="serverData.status === 'installing'"
        data-pyro-server-installing
        class="mb-4 flex w-full flex-row items-center gap-4 rounded-2xl bg-bg-blue p-4 text-sm text-contrast"
      >
        <UiServersServerIcon :image="serverData.image" class="!h-10 !w-10" />

        <div class="flex flex-col gap-1">
          <span class="text-lg font-bold"> We're preparing your server! </span>
          <div class="flex flex-row items-center gap-2">
            <UiServersPanelSpinner class="!h-3 !w-3" /> <LazyUiServersInstallingTicker />
          </div>
        </div>
      </div>
      <NuxtPage
        :route="route"
        :is-connected="isConnected"
        :is-ws-auth-incorrect="isWSAuthIncorrect"
        :is-server-running="isServerRunning"
        :stats="stats"
        :server-power-state="serverPowerState"
        :power-state-details="powerStateDetails"
        :socket="socket"
        :server="server"
        :backup-in-progress="backupInProgress"
        @reinstall="onReinstall"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import {
  CopyIcon,
  IssuesIcon,
  LeftArrowIcon,
  RightArrowIcon,
  CheckIcon,
  FileIcon,
  TransferIcon,
  LockIcon,
} from "@modrinth/assets";
import DOMPurify from "dompurify";
import { ButtonStyled, ServerNotice } from "@modrinth/ui";
import { Intercom, shutdown } from "@intercom/messenger-js-sdk";
import { reloadNuxtApp, navigateTo } from "#app";
import type { MessageDescriptor } from "@vintl/vintl";
import type { ServerState, Stats, WSEvent, WSInstallationResultEvent } from "~/types/servers";
import { usePyroConsole } from "~/store/console.ts";
import { type Backup } from "~/composables/pyroServers.ts";
import { usePyroFetch } from "~/composables/pyroFetch.ts";

const app = useNuxtApp() as unknown as { $notify: any };

const socket = ref<WebSocket | null>(null);
const isReconnecting = ref(false);
const isLoading = ref(true);
const reconnectInterval = ref<ReturnType<typeof setInterval> | null>(null);
const isFirstMount = ref(true);
const isMounted = ref(true);

const INTERCOM_APP_ID = ref("ykeritl9");
const auth = (await useAuth()) as unknown as {
  value: { user: { id: string; username: string; email: string; created: string } };
};
const userId = ref(auth.value?.user?.id ?? null);
const username = ref(auth.value?.user?.username ?? null);
const email = ref(auth.value?.user?.email ?? null);
const createdAt = ref(
  auth.value?.user?.created ? Math.floor(new Date(auth.value.user.created).getTime() / 1000) : null,
);

const route = useNativeRoute();
const router = useRouter();
const serverId = route.params.id as string;

const server = await usePyroServer(serverId, ["general", "ws"]);

const loadModulesPromise = Promise.resolve().then(() => {
  if (server.general?.status === "suspended") {
    return;
  }
  return server.loadModules(["content", "backups", "network", "startup", "fs"]);
});

provide("modulesLoaded", loadModulesPromise);

watch(
  () => [server.general?.error, server.ws?.error],
  ([generalError, wsError]) => {
    if (server.general?.status === "suspended") return;

    const error = generalError?.error || wsError?.error;
    if (error && error.statusCode !== 403) {
      startPolling();
    }
  },
);

const errorTitle = ref("Error");
const errorMessage = ref("An unexpected error occurred.");
const errorLog = ref("");
const errorLogFile = ref("");
const serverData = computed(() => server.general);
const isConnected = ref(false);
const isWSAuthIncorrect = ref(false);
const pyroConsole = usePyroConsole();
const cpuData = ref<number[]>([]);
const ramData = ref<number[]>([]);
const isActioning = ref(false);
const isServerRunning = computed(() => serverPowerState.value === "running");
const serverPowerState = ref<ServerState>("stopped");
const powerStateDetails = ref<{ oom_killed?: boolean; exit_code?: number }>();

const uptimeSeconds = ref(0);
const firstConnect = ref(true);
const copied = ref(false);
const error = ref<Error | null>(null);

const initialConsoleMessage = [
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

const filteredNotices = computed(
  () => serverData.value?.notices?.filter((n) => n.level !== "survey") ?? [],
);
const surveyNotice = computed(() => serverData.value?.notices?.find((n) => n.level === "survey"));

async function dismissSurvey() {
  const noticeId = surveyNotice.value?.id;
  if (noticeId === undefined) {
    console.warn("No survey notice to dismiss");
    return;
  }
  await dismissNotice(noticeId);
  console.log(`Dismissed survey notice ${noticeId}`);
}

type TallyPopupOptions = {
  key?: string;
  layout?: "default" | "modal";
  width?: number;
  alignLeft?: boolean;
  hideTitle?: boolean;
  overlay?: boolean;
  emoji?: {
    text: string;
    animation:
      | "none"
      | "wave"
      | "tada"
      | "heart-beat"
      | "spin"
      | "flash"
      | "bounce"
      | "rubber-band"
      | "head-shake";
  };
  autoClose?: number;
  showOnce?: boolean;
  doNotShowAfterSubmit?: boolean;
  customFormUrl?: string;
  hiddenFields?: {
    [key: string]: unknown;
  };
  onOpen?: () => void;
  onClose?: () => void;
  onPageView?: (page: number) => void;
  onSubmit?: (payload: unknown) => void;
};

const popupOptions = computed(
  () =>
    ({
      layout: "default",
      width: 400,
      autoClose: 2000,
      hideTitle: true,
      hiddenFields: {
        username: auth.value?.user?.username,
        user_id: auth.value?.user?.id,
        user_email: auth.value?.user?.email,
        server_id: serverData.value?.server_id,
        loader: serverData.value?.loader,
        game_version: serverData.value?.mc_version,
        modpack_id: serverData.value?.project?.id,
        modpack_name: serverData.value?.project?.title,
      },
      onOpen: () => console.log(`Opened survey notice: ${surveyNotice.value?.id}`),
      onClose: async () => await dismissSurvey(),
      onSubmit: (payload: any) => {
        console.log("Form submitted:", payload);
      },
    }) satisfies TallyPopupOptions,
);

function showSurvey() {
  if (!surveyNotice.value) {
    console.warn("No survey notice to open");
    return;
  }

  try {
    if ((window as any).Tally?.openPopup) {
      console.log(
        `Opening Tally popup for survey notice ${surveyNotice.value?.id} (form ID: ${surveyNotice.value?.message})`,
      );
      (window as any).Tally.openPopup(surveyNotice.value?.message, popupOptions.value);
    } else {
      console.warn("Tally script not yet loaded");
    }
  } catch (e) {
    console.error("Error opening Tally popup:", e);
  }
}

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

      pyroConsole.clear();
      socket.value?.send(JSON.stringify({ event: "auth", jwt: wsAuth.value?.token }));
      isConnected.value = true;
      isReconnecting.value = false;
      isLoading.value = false;

      if (firstConnect.value) {
        for (let i = 0; i < initialConsoleMessage.length; i++) {
          pyroConsole.addLine(initialConsoleMessage[i]);
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
        pyroConsole.addLine("\nSomething went wrong with the connection, we're reconnecting...");
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
      pyroConsole.addLines(log);
      break;
    case "stats":
      updateStats(data);
      break;
    case "auth-expiring":
    case "auth-incorrect":
      reauthenticate();
      break;
    case "power-state":
      if (data.state === "crashed") {
        updatePowerState(data.state, {
          oom_killed: data.oom_killed,
          exit_code: data.exit_code,
        });
      } else {
        updatePowerState(data.state);
      }
      break;
    case "installation-result":
      handleInstallationResult(data);
      break;
    case "new-mod":
      server.refresh(["content"]);
      console.log("New mod:", data);
      break;
    case "auth-ok":
      break;
    case "uptime":
      stopUptimeUpdates();
      uptimeSeconds.value = data.uptime;
      startUptimeUpdates();
      break;
    case "backup-progress": {
      // Update a backup's state
      const curBackup = server.backups?.data.find((backup) => backup.id === data.id);

      if (!curBackup) {
        console.log(`Ignoring backup-progress event for unknown backup: ${data.id}`);
      } else {
        console.log(
          `Handling backup progress for ${curBackup.name} (${data.id}) task: ${data.task} state: ${data.state} progress: ${data.progress}`,
        );

        if (!curBackup.task) {
          curBackup.task = {};
        }

        curBackup.task[data.task] = {
          progress: data.progress,
          state: data.state,
        };

        curBackup.ongoing = data.task === "create" && data.state === "ongoing";
      }

      break;
    }
    case "filesystem-ops": {
      if (!server.fs) {
        console.error("FilesystemOps received, but server.fs is not available", data.all);
        break;
      }
      if (JSON.stringify(server.fs.ops) !== JSON.stringify(data.all)) {
        server.fs.ops = data.all;
      }

      server.fs.queuedOps = server.fs.queuedOps.filter(
        (queuedOp) => !data.all.some((x) => x.src === queuedOp.src),
      );

      const cancelled = data.all.filter((x) => x.state === "cancelled");
      Promise.all(cancelled.map((x) => server.fs?.modifyOp(x.id, "dismiss")));

      const completed = data.all.filter((x) => x.state === "done");
      if (completed.length > 0) {
        setTimeout(
          async () =>
            await Promise.all(
              completed.map((x) => {
                if (!server.fs?.opsQueuedForModification.includes(x.id)) {
                  server.fs?.opsQueuedForModification.push(x.id);
                  return server.fs?.modifyOp(x.id, "dismiss");
                }
                return Promise.resolve();
              }),
            ),
          3000,
        );
      }
      break;
    }
    default:
      console.warn("Unhandled WebSocket event:", data);
  }
};

const newLoader = ref<string | null>(null);
const newLoaderVersion = ref<string | null>(null);
const newMCVersion = ref<string | null>(null);

const onReinstall = (potentialArgs: any) => {
  if (!serverData.value) return;

  serverData.value.status = "installing";

  if (potentialArgs?.loader) {
    newLoader.value = potentialArgs.loader;
  }
  if (potentialArgs?.lVersion) {
    newLoaderVersion.value = potentialArgs.lVersion;
  }
  if (potentialArgs?.mVersion) {
    newMCVersion.value = potentialArgs.mVersion;
  }

  error.value = null;
  errorTitle.value = "Error";
  errorMessage.value = "An unexpected error occurred.";
};

const handleInstallationResult = async (data: WSInstallationResultEvent) => {
  switch (data.result) {
    case "ok": {
      if (!serverData.value) break;

      stopPolling();

      try {
        await new Promise((resolve) => setTimeout(resolve, 2000));

        let attempts = 0;
        const maxAttempts = 3;
        let hasValidData = false;

        while (!hasValidData && attempts < maxAttempts) {
          attempts++;

          await server.refresh(["general"], {
            preserveConnection: true,
            preserveInstallState: true,
          });

          if (serverData.value?.loader && serverData.value?.mc_version) {
            hasValidData = true;
            serverData.value.status = "available";
            await server.refresh(["content", "startup"]);
            break;
          }

          await new Promise((resolve) => setTimeout(resolve, 2000));
        }

        if (!hasValidData) {
          console.error("Failed to get valid server data after installation");
        }
      } catch (err: unknown) {
        console.error("Error refreshing data after installation:", err);
      }

      newLoader.value = null;
      newLoaderVersion.value = null;
      newMCVersion.value = null;
      error.value = null;
      break;
    }
    case "err": {
      console.log("failed to install");
      console.log(data);
      errorTitle.value = "Installation error";
      errorMessage.value = data.reason ?? "Unknown error";
      error.value = new Error(data.reason ?? "Unknown error");
      let files = await server.fs?.listDirContents("/", 1, 100);
      if (files) {
        if (files.total > 1) {
          for (let i = 1; i < files.total; i++) {
            const nextFiles = await server.fs?.listDirContents("/", i, 100);
            if (nextFiles?.items?.length === 0) break;
            if (nextFiles) files = nextFiles;
          }
        }
      }
      const fileName = files?.items?.find((file: { name: string }) =>
        file.name.startsWith("modrinth-installation"),
      )?.name;
      errorLogFile.value = fileName ?? "";
      if (fileName) {
        errorLog.value = await server.fs?.downloadFile(fileName);
      }
      break;
    }
  }
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

const updatePowerState = (
  state: ServerState,
  details?: { oom_killed?: boolean; exit_code?: number },
) => {
  serverPowerState.value = state;

  if (state === "crashed") {
    powerStateDetails.value = details;
  } else {
    powerStateDetails.value = undefined;
  }

  if (state === "stopped" || state === "crashed") {
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
const countdown = ref(15);

const formattedTime = computed(() => {
  const seconds = countdown.value % 60;
  return `${seconds.toString().padStart(2, "0")}`;
});

export type BackupInProgressReason = {
  type: string;
  tooltip: MessageDescriptor;
};

const RestoreInProgressReason = {
  type: "restore",
  tooltip: defineMessage({
    id: "servers.backup.restore.in-progress.tooltip",
    defaultMessage: "Backup restore in progress",
  }),
} satisfies BackupInProgressReason;

const CreateInProgressReason = {
  type: "create",
  tooltip: defineMessage({
    id: "servers.backup.create.in-progress.tooltip",
    defaultMessage: "Backup creation in progress",
  }),
} satisfies BackupInProgressReason;

const backupInProgress = computed(() => {
  const backups = server.backups?.data;
  if (!backups) {
    return undefined;
  }
  if (backups.find((backup: Backup) => backup?.task?.create?.state === "ongoing")) {
    return CreateInProgressReason;
  }
  if (backups.find((backup: Backup) => backup?.task?.restore?.state === "ongoing")) {
    return RestoreInProgressReason;
  }
  return undefined;
});

const stopPolling = () => {
  if (intervalId) {
    clearInterval(intervalId);
    intervalId = null;
  }
};

const startPolling = () => {
  countdown.value = 15;
  intervalId = setInterval(() => {
    if (countdown.value <= 0) {
      reloadNuxtApp();
    } else {
      countdown.value--;
    }
  }, 1000);
};

const copyServerDebugInfo = () => {
  const debugInfo = `Server ID: ${serverData.value?.server_id}\nError: ${errorMessage.value}\nKind: ${serverData.value?.upstream?.kind}\nProject ID: ${serverData.value?.upstream?.project_id}\nVersion ID: ${serverData.value?.upstream?.version_id}\nLog: ${errorLog.value}`;
  navigator.clipboard.writeText(debugInfo);
  copied.value = true;
  setTimeout(() => {
    copied.value = false;
  }, 5000);
};

const openInstallLog = () => {
  router.replace({
    path: `/servers/manage/${serverId}/files`,
    query: { ...route.query, editing: errorLogFile.value },
  });
};

const cleanup = () => {
  isMounted.value = false;

  shutdown();

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

async function dismissNotice(noticeId: number) {
  await usePyroFetch(`servers/${serverId}/notices/${noticeId}/dismiss`, {
    method: "POST",
  }).catch((err) => {
    app.$notify({
      group: "main",
      title: "Error dismissing notice",
      text: err,
      type: "error",
    });
  });
  await server.refresh(["general"]);
}

onMounted(() => {
  isMounted.value = true;
  if (server.general?.status === "suspended") {
    isLoading.value = false;
    return;
  }
  if (server.error) {
    if (!server.error.message.includes("Forbidden")) {
      startPolling();
    }
  } else {
    connectWebSocket();
  }

  if (username.value && email.value && userId.value && createdAt.value) {
    const currentUser = auth.value?.user as any;
    const matches =
      username.value === currentUser?.username &&
      email.value === currentUser?.email &&
      userId.value === currentUser?.id &&
      createdAt.value === Math.floor(new Date(currentUser?.created).getTime() / 1000);

    if (matches) {
      Intercom({
        app_id: INTERCOM_APP_ID.value,
        userId: userId.value,
        name: username.value,
        email: email.value,
        created_at: createdAt.value,
      });
    } else {
      console.warn("[PYROSERVERS][INTERCOM] mismatch");
    }
  }

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

  if (surveyNotice.value) {
    showSurvey();
  }
});

onUnmounted(() => {
  cleanup();
});

watch(
  () => serverData.value?.status,
  (newStatus, oldStatus) => {
    if (isFirstMount.value) {
      isFirstMount.value = false;
      return;
    }

    if (newStatus === "installing" && oldStatus !== "installing") {
      countdown.value = 15;
      startPolling();
    }
  },
);

definePageMeta({
  middleware: "auth",
});

useHead({
  script: [
    {
      src: "https://tally.so/widgets/embed.js",
      defer: true,
    },
  ],
});
</script>

<style>
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

.mobile-blurred-servericon::before {
  position: absolute;
  left: 0;
  top: 0;
  display: block;
  height: 9rem;
  width: 100%;
  background-size: cover;
  background-position: center;
  background-repeat: no-repeat;
  filter: blur(1rem);
  content: "";
  background-image: linear-gradient(
      to bottom,
      rgba(from var(--color-raised-bg) r g b / 0.2),
      rgb(from var(--color-raised-bg) r g b / 0.8)
    ),
    var(--server-bg-image);
}

@media screen and (min-width: 640px) {
  .mobile-blurred-servericon::before {
    display: none;
  }
}
</style>
