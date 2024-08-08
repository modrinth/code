<template>
  <div v-if="isConnected" data-pyro-server-manager-root class="flex flex-col gap-6">
    <UiServersServerStats :data="stats" />
    <div class="relative w-full overflow-hidden rounded-2xl bg-bg-raised p-8">
      <div class="experimental-styles-within flex flex-row items-center justify-between">
        <div class="flex flex-row items-center gap-4">
          <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">Console</h2>
          <div
            :class="`flex items-center gap-2 rounded-full px-2 py-1 ${stats.current.cpu_percent === null ? 'bg-red-400/10 text-red-500' : 'bg-green-400/10 text-green-500'}`"
          >
            <span class="text-sm font-semibold">
              {{ stats.current.cpu_percent === null ? "Server Offline" : "Server Online" }}
            </span>
          </div>
        </div>
        <Button
          v-if="stats.current.cpu_percent === null"
          @click="sendPowerAction('start')"
          color="secondary"
          :disabled="isActioning"
        >
          <div v-if="isActioning" class="grid place-content-center">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 20 20"
              fill="currentColor"
              class="size-5 animate-spin"
            >
              <path
                fill-rule="evenodd"
                d="M15.312 11.424a5.5 5.5 0 0 1-9.201 2.466l-.312-.311h2.433a.75.75 0 0 0 0-1.5H3.989a.75.75 0 0 0-.75.75v4.242a.75.75 0 0 0 1.5 0v-2.43l.31.31a7 7 0 0 0 11.712-3.138.75.75 0 0 0-1.449-.39Zm1.23-3.723a.75.75 0 0 0 .219-.53V2.929a.75.75 0 0 0-1.5 0V5.36l-.31-.31A7 7 0 0 0 3.239 8.188a.75.75 0 1 0 1.448.389A5.5 5.5 0 0 1 13.89 6.11l.311.31h-2.432a.75.75 0 0 0 0 1.5h4.243a.75.75 0 0 0 .53-.219Z"
                clip-rule="evenodd"
              />
            </svg>
          </div>

          <div class="contents" v-else>
            <PlayIcon />
            Start
          </div>
        </Button>

        <Button v-else @click="sendPowerAction('restart')" color="secondary" :loading="isActioning">
          <div v-if="isActioning" class="grid place-content-center">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 20 20"
              fill="currentColor"
              class="size-5 animate-spin"
            >
              <path
                fill-rule="evenodd"
                d="M15.312 11.424a5.5 5.5 0 0 1-9.201 2.466l-.312-.311h2.433a.75.75 0 0 0 0-1.5H3.989a.75.75 0 0 0-.75.75v4.242a.75.75 0 0 0 1.5 0v-2.43l.31.31a7 7 0 0 0 11.712-3.138.75.75 0 0 0-1.449-.39Zm1.23-3.723a.75.75 0 0 0 .219-.53V2.929a.75.75 0 0 0-1.5 0V5.36l-.31-.31A7 7 0 0 0 3.239 8.188a.75.75 0 1 0 1.448.389A5.5 5.5 0 0 1 13.89 6.11l.311.31h-2.432a.75.75 0 0 0 0 1.5h4.243a.75.75 0 0 0 .53-.219Z"
                clip-rule="evenodd"
              />
            </svg>
          </div>

          <div class="contents" v-else>
            <UpdatedIcon />
            Restart
          </div>
        </Button>
      </div>

      <div
        class="monocraft-font console-container relative mt-4 h-full w-full overflow-hidden rounded-xl bg-black p-6 text-sm"
      >
        <div id="console" class="h-[300px] overflow-y-auto">
          <VirtualScroller
            ref="scroller"
            :default-size="30"
            :items="consoleOutput"
            style="white-space: pre; word-wrap: break-word; width: 100%; line-height: 170%"
          >
            <template #item="{ index, offset, ref }">
              <LogParser v-if="ref" :log="ref" />
            </template>
          </VirtualScroller>
        </div>
      </div>
    </div>
  </div>
  <div v-else class="flex h-full items-center justify-center">
    <UiServersPyroLoading />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from "vue";
import { PlayIcon, UpdatedIcon } from "@modrinth/assets";
import { Button } from "@modrinth/ui";
import type { Stats, WSAuth, WSEvent } from "~/types/servers";
import { createVirtualScroller } from "vue-typed-virtual-list";
import LogParser from "~/components/ui/servers/LogParser.vue";

const VirtualScroller = createVirtualScroller<string>();

const app = useNuxtApp();
type VirtualListInstance = InstanceType<typeof VirtualScroller>;
const scroller = ref<VirtualListInstance | null>(null);

const route = useNativeRoute();

const serverId = route.params.id;
const auth = await useAuth();

type TPowerAction = "restart" | "start" | "stop" | "kill";

const isActioning = ref(false);
const isConnected = ref(false);

const sendPowerAction = async (action: TPowerAction) => {
  console.log("Sending power action", action);
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
    action: actionName,
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
  try {
    socket.onmessage = (event) => {
      const data = JSON.parse(event.data);
      if (data.event === "log") {
        consoleOutput.value.push(data.message);
      } else if (data.event === "stats") {
        isConnected.value = true;
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
      consoleOutput.value.push("\nWS died, oops");
      isConnected.value = false;
    };
  } catch (error) {
    console.error("ws failed:", error);
  }
};

onMounted(() => {
  connectWebSocket();
});

onBeforeUnmount(() => {
  if (socket) {
    socket.close();
    socket = null;
  }
});
</script>
