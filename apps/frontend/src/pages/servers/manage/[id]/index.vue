<template>
  <div class="flex flex-col gap-6">
    <UiServersServerStats :data="stats" />
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
        <Button @click="sendPowerAction('stop')" :disabled="isActioning" color="secondary">
          <UpdatedIcon />
          Stop Server
        </Button>
        <Button @click="sendPowerAction('kill')" :disabled="isActioning" color="secondary">
          <UpdatedIcon />
          Kill Server
        </Button>
      </div>

      <div
        class="monocraft-font console-container relative mt-4 h-full w-full overflow-hidden rounded-xl bg-black p-6 text-sm"
      >
        <div id="console" class="h-[300px] overflow-y-auto">
          <VirtualScroller
            :default-size="30"
            :items="consoleOutput"
            style="white-space: pre; word-wrap: break-word; width: 100%; line-height: 170%"
          >
            <template #item="{ index, offset, ref }">
              <Log v-if="ref" :log-line="ref" />
            </template>
          </VirtualScroller>
        </div>
        <button color="secondary" class="absolute right-8 top-8" @click="toggleFullScreen">
          <ExpandIcon />
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from "vue";
import { ExpandIcon, UpdatedIcon } from "@modrinth/assets";
import { Button } from "@modrinth/ui";
import type { Stats, WSAuth, WSEvent } from "~/types/servers";
import { useNotifications } from "@/composables/notifs";
import { defineComponent } from "vue";
import { createVirtualScroller } from "vue-typed-virtual-list";
import Log from "~/components/ui/servers/Log.vue";

const VirtualScroller = createVirtualScroller<string>();

const app = useNuxtApp();
const notifications = useNotifications();
const isFullScreen = ref(false);

const config = useRuntimeConfig();
const route = useNativeRoute();
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
    action: action,
  });
  isActioning.value = false;
};

let socket: WebSocket | null = null;

const reauth = async () => {
  const wsAuth = await usePyroFetch<WSAuth>(auth.value.token, `servers/${serverId}/ws`);
  socket?.send(
    JSON.stringify({
      event: "auth",
      jwt: wsAuth.token,
    }),
  );
  return wsAuth;
};

const consoleOutput = ref<string[]>([]);
const cpu_data = ref<number[]>([]);
const ram_data = ref<number[]>([]);

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

const connectWebSocket = async () => {
  const wsAuth = await usePyroFetch<WSAuth>(auth.value.token, `servers/${serverId}/ws`);
  socket = new WebSocket(`ws://127.0.0.1:6527/v0/ws`);
  await reauth();

  socket.onmessage = (event) => {
    const data = JSON.parse(event.data);
    if (data.event === "log") {
      consoleOutput.value.push(data.message);
    } else if (data.event === "stats") {
      stats.value = {
        current: data,
        past: stats.value.current,
        graph: {
          cpu: cpu_data.value,
          ram: ram_data.value,
        },
      };

      cpu_data.value.push(Math.round(data.cpu_percent * 100) / 100);
      if (cpu_data.value.length > 10) {
        cpu_data.value.shift();
      }

      ram_data.value.push(Math.floor((data.ram_usage_bytes / data.ram_total_bytes) * 100));
      if (ram_data.value.length > 10) {
        ram_data.value.shift();
      }
    } else if (data.event === "auth-expiring") {
      reauth();
    }
  };

  socket.onclose = () => {
    consoleOutput.value.push("\nDisconnected from the server");
  };

  socket.onerror = (error) => {
    console.log(error);
    consoleOutput.value.push(
      `\nError: Failed to establish a websocket connection to the server, please try refreshing the page or contact support if the issue persists.`,
    );
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
