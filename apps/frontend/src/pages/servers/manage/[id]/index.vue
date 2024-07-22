<template>
  <div class="flex flex-col gap-6">
    <UiServersServerStats
      :cpu-percent="stats.cpu_percent"
      :ram-usage-bytes="stats.ram_usage_bytes"
      :ram-total-bytes="stats.ram_total_bytes"
      :storage-usage-bytes="stats.storage_usage_bytes"
      :storage-total-bytes="stats.storage_total_bytes"
    />
    <div class="relative w-full overflow-hidden rounded-2xl bg-bg-raised p-8">
      <div class="experimental-styles-within flex flex-row items-center justify-between">
        <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">Console</h2>
        <Button @click="sendPowerAction('start')" :disabled="isActioning" color="secondary">
          <UpdatedIcon />
          Start Server
        </Button>
        <Button @click="sendPowerAction('restart')" :disabled="isActioning" color="secondary">
          <UpdatedIcon />
          Restart Server
        </Button>
      </div>

      <div
        class="terminal-font console-container relative mt-4 h-full w-full overflow-hidden rounded-xl bg-black p-6 text-sm"
      >
        <button color="secondary" class="absolute right-8 top-8" @click="toggleFullScreen">
          <ExpandIcon />
        </button>
        <div id="console" class="h-[300px] overflow-y-auto">
          <pre
            style="
              all: unset;
              white-space: pre;
              word-wrap: break-word;
              width: 100%;
              line-height: 170%;
            "
          >
            {{ consoleOutput }}
          </pre>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from "vue";
import { ExpandIcon, UpdatedIcon } from "@modrinth/assets";
import { Button } from "@modrinth/ui";
import type { Stats, WSAuth } from "~/types/servers";
import { useNotifications } from "@/composables/notifs";

const app = useNuxtApp();
const notifications = useNotifications();
const isFullScreen = ref(false);
const consoleOutput = ref("");
const stats = ref<Stats>({
  cpu_percent: 0,
  ram_usage_bytes: 0,
  ram_total_bytes: 1,
  storage_usage_bytes: 0,
  storage_total_bytes: 0,
});

const route = useNativeRoute();
const config = useRuntimeConfig();
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const serverId = route.params.id;
const auth = await useAuth();

const toggleFullScreen = () => {
  const consoleElement = document.querySelector(".console-container");
  const conElement = document.getElementById("console");
  if (!consoleElement || !conElement) return;

  if (!document.fullscreenElement) {
    consoleElement.requestFullscreen().then(() => {
      isFullScreen.value = true;
    });
    conElement.style.height = "100%";
  } else {
    document.exitFullscreen().then(() => {
      isFullScreen.value = false;
    });
    conElement.style.height = "300px";
  }
};

type TPowerAction = "restart" | "start" | "stop" | "kill";

const isActioning = ref(false);
const sendPowerAction = async (action: TPowerAction) => {
  isActioning.value = true;
  const actionName = action.charAt(0).toUpperCase() + action.slice(1);
  // @ts-ignore
  app.$notify({
    group: "server",
    title: `${actionName}ing server`,
    text: `Your server is now ${actionName}ing, this may take a few moments`,
    type: "success",
  });
  await usePyroFetch(auth.value.token, `servers/${serverId}/power`, 0, "POST", "application/json", {
    action: action
  });
  isActioning.value = false;
};

let socket: WebSocket | null = null;

const wsAuth = await usePyroFetch<WSAuth>(auth.value.token, `servers/${serverId}/ws`);


const reauth = () => {
  socket?.send(
    JSON.stringify({
      event: "auth",
      modrinth_token: wsAuth.token,
    }),
  );
};

const connectWebSocket = () => {
  socket = new WebSocket(
    `wss://082c-207-171-252-31.ngrok-free.app/modrinth/v0/servers/${serverId}/ws`,
  );

  socket.onmessage = (event) => {
    const data = JSON.parse(event.data);
    if (data.event === "log") {
      consoleOutput.value += `\n${data.message}`;
    } else if (data.event === "stats") {
      stats.value = data;
    } else if (data.event === "auth-expiring") {
      reauth();
    }
  };

  socket.onclose = () => {
    consoleOutput.value += "\nDisconnected from the server";
  };

  socket.onerror = (error) => {
    console.log(error);
    consoleOutput.value += `\nError: Failed to establish a websocket connection to the server, please try refreshing the page or contact support if the issue persists.`;
  };
};

onMounted(() => {
  connectWebSocket();
});

onBeforeUnmount(() => {
  if (socket) {
    socket.close();
  }
});
</script>
