<template>
  <div v-if="isConnected" data-pyro-server-manager-root class="flex flex-col gap-6">
    <transition name="fade-slide">
      <UiServersServerStats :data="stats" v-if="!fullScreen" />
    </transition>
    <div
      class="relative flex w-full flex-col gap-3 overflow-hidden rounded-2xl bg-bg-raised p-8 transition-[height] duration-500 ease-in-out"
      :style="consoleStyle"
    >
      <div class="experimental-styles-within flex flex-row items-center justify-between">
        <div class="flex flex-row items-center gap-4">
          <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">Console</h2>
          <UiServersPanelServerStatus :is-online="isServerOnline" />
        </div>
        <UiServersPanelServerActionButton
          :is-online="isServerOnline"
          :is-actioning="isActioning"
          @action="sendPowerAction"
        />
      </div>

      <UiServersPanelTerminal
        :console-output="consoleOutput"
        :full-screen="fullScreen"
        @toggle-full-screen="toggleFullScreen"
      />
    </div>
  </div>
  <PyroLoading v-else-if="!isConnected" />
  <PyroError
    v-else
    title="Error Accessing Server"
    message="Don't worry, your server is safe. We just can't connect to it right now."
  />
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from "vue";
import type { Stats, WSAuth, WSEvent } from "~/types/servers";
import PyroError from "~/components/ui/servers/PyroError.vue";
import PyroLoading from "~/components/ui/servers/PyroLoading.vue";

const fullScreen = ref(false);
const consoleStyle = ref({ height: "400px", marginTop: "0px" });
const isActioning = ref(false);
const isConnected = ref(false);
const consoleOutput = ref<string[]>([]);
const cpuData = ref<number[]>([]);
const ramData = ref<number[]>([]);

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

const isServerOnline = computed(() => stats.value.current.cpu_percent !== null);

const app = useNuxtApp();
const route = useRoute();
const auth = await useAuth();
const serverId = route.params.id as string;

let socket: WebSocket | null = null;

const toggleFullScreen = () => {
  fullScreen.value = !fullScreen.value;
  if (fullScreen.value) {
    consoleStyle.value.height = "70vh";
    animateMarginTop();
  } else {
    consoleStyle.value.height = "400px";
    consoleStyle.value.marginTop = "0px";
  }
};

const animateMarginTop = () => {
  setTimeout(() => {
    let mt = 254;
    const interval = setInterval(() => {
      mt -= 10;
      consoleStyle.value.marginTop = `${mt}px`;
      if (mt <= 0 || !fullScreen.value) clearInterval(interval);
    }, 10);
  }, 500);
};

const sendPowerAction = async (action: "restart" | "start" | "stop" | "kill") => {
  isActioning.value = true;
  const actionName = action.charAt(0).toUpperCase() + action.slice(1);
  // @ts-ignore
  app.$notify({
    group: "server",
    title: `${actionName}ing server`,
    text: `Your server is now ${actionName}ing, this may take a few moments`,
    type: "success",
  });

  try {
    await usePyroFetch(
      auth.value.token,
      `servers/${serverId}/power`,
      0,
      "POST",
      "application/json",
      {
        action: actionName,
      },
    );
  } catch (error) {
    console.error(`Error ${actionName}ing server:`, error);
    // @ts-ignore
    app.$notify({
      group: "server",
      title: `Error ${actionName}ing server`,
      text: "An error occurred while attempting to perform the action.",
      type: "error",
    });
  } finally {
    isActioning.value = false;
  }
};

const connectWebSocket = async () => {
  const wsAuth = await usePyroFetch<WSAuth>(auth.value.token, `servers/${serverId}/ws`);
  socket = new WebSocket(`ws://127.0.0.1:6632/v0/ws`);

  socket.onopen = () => {
    socket?.send(JSON.stringify({ event: "auth", jwt: wsAuth.token }));
  };

  socket.onmessage = (event) => handleWebSocketMessage(JSON.parse(event.data));

  socket.onclose = () => {
    consoleOutput.value.push("\nWS connection closed");
    isConnected.value = false;
  };

  socket.onerror = (error) => {
    console.error("WebSocket error:", error);
    isConnected.value = false;
  };
};

const handleWebSocketMessage = (data: WSEvent) => {
  switch (data.event) {
    case "log":
      consoleOutput.value.push(data.message);
      break;
    case "stats":
      // FIX PLS
      updateStats(data as any);
      break;
    case "auth-expiring":
      reauth();
      break;
  }
};

const updateStats = (data: Stats["current"]) => {
  isConnected.value = true;
  stats.value = {
    current: data,
    past: stats.value.current,
    graph: {
      cpu: updateDataArray(cpuData.value, Math.round(data.cpu_percent * 100) / 100),
      ram: updateDataArray(
        ramData.value,
        Math.floor((data.ram_usage_bytes / data.ram_total_bytes) * 100),
      ),
    },
  };
};

const updateDataArray = (arr: number[], newValue: number) => {
  arr.push(newValue);
  if (arr.length > 10) arr.shift();
  return [...arr];
};

const reauth = async () => {
  const wsAuth = await usePyroFetch<WSAuth>(auth.value.token, `servers/${serverId}/ws`);
  socket?.send(JSON.stringify({ event: "auth", jwt: wsAuth.token }));
};

onMounted(connectWebSocket);
onBeforeUnmount(() => socket?.close());
</script>

<style scoped>
.fade-slide-enter-active,
.fade-slide-leave-active {
  transition:
    opacity 0.5s ease,
    transform 0.5s ease;
}
.fade-slide-enter-from,
.fade-slide-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}

.console {
  transition:
    height 0.5s ease,
    margin-top 0.5s ease;
}
</style>
