<template>
  <div class="flex flex-col gap-6">
    <UiServersServerStats
      :cpu-percent="stats.cpu_percent"
      :ram-usage-bytes="stats.ram_usage_bytes"
      :ram-total-bytes="stats.ram_total_bytes"
      :storage-usage-bytes="stats.storage_usage_bytes"
      :storage-total-bytes="stats.storage_total_bytes"
    />
    <div class="relative w-full overflow-hidden rounded-2xl bg-[var(--color-raised-bg)] p-8">
      <div class="experimental-styles-within flex flex-row items-center justify-between">
        <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">Console</h2>
        <Button color="secondary">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke-width="1.5"
            stroke="currentColor"
            class="size-6"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0 3.181 3.183a8.25 8.25 0 0 0 13.803-3.7M4.031 9.865a8.25 8.25 0 0 1 13.803-3.7l3.181 3.182m0-4.991v4.99"
            />
          </svg>
          Restart Server
        </Button>
      </div>

      <div
        class="terminal-font console-container relative mt-4 h-full w-full overflow-hidden rounded-xl bg-black p-6 text-sm"
      >
        <button color="secondary" class="absolute right-8 top-8" @click="toggleFullScreen">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke-width="1.5"
            stroke="currentColor"
            class="size-6"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M3.75 3.75v4.5m0-4.5h4.5m-4.5 0L9 9M3.75 20.25v-4.5m0 4.5h4.5m-4.5 0L9 15M20.25 3.75h-4.5m4.5 0v4.5m0-4.5L15 9m5.25 11.25h-4.5m4.5 0v-4.5m0 4.5L15 15"
            />
          </svg>
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
import { Button } from "@modrinth/ui";
import type { Stats } from "~/types/servers";

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
const base = process.server ? config.pyroBaseUrl : config.public.pyroBaseUrl;

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

let socket: WebSocket | null = null;

const reauth = () => {
  socket?.send(
    JSON.stringify({
      event: "auth",
      modrinth_token: "auth_ok",
    }),
  );
};

const connectWebSocket = () => {
  socket = new WebSocket(
    `wss://${base.replace(/^https?:\/\//, "")}modrinth/v0/servers/2000151e-5da2-4ef0-b2ba-1407dec07729/ws`,
  );

  socket.onopen = () => {
    reauth();
  };

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
    consoleOutput.value += `\nWebSocket error: ${error}`;
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
