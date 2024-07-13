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
