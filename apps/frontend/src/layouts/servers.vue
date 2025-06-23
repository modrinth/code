<template>
  <NuxtLayout name="default">
    <div v-if="server">
      <slot />
    </div>
    <div
      v-else-if="error"
      class="flex min-h-[calc(100vh-4rem)] items-center justify-center text-contrast"
    >
      <ErrorInformationCard
        title="An error occurred"
        :description="error.message"
        :icon="TriangleAlertIcon"
        icon-color="red"
        :action="generalErrorAction"
      />
    </div>
    <div v-else class="flex min-h-[calc(100vh-4rem)] items-center justify-center">
      <PanelSpinner class="size-10 animate-spin" />
    </div>
  </NuxtLayout>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import {
  ErrorInformationCard,
  ModrinthServer,
  PanelSpinner,
  provideModrinthServerContext,
  useModrinthServers,
} from "@modrinth/ui";
import { TriangleAlertIcon } from "@modrinth/assets";

const route = useNativeRoute();
const router = useRouter();

const serverId = computed(() => route.params.id as string);
const server = ref<ModrinthServer | null>(null);
const error = ref<Error | null>(null);
const isLoading = ref(true);

async function initializeServer() {
  try {
    isLoading.value = true;
    error.value = null;

    const auth = await useAuth();
    const config = useRuntimeConfig();
    const base = import.meta.server ? config.pyroBaseUrl : config.public.pyroBaseUrl;

    server.value = await useModrinthServers(
      serverId.value,
      auth.value.token,
      base,
      new NuxtStateStorage(),
      ["general", "ws"],
    );

    if (server.value?.general?.status !== "suspended") {
      server.value
        ?.refresh(["content", "backups", "network", "startup", "fs"])
        .catch((err: any) => {
          console.warn("Failed to load additional server modules:", err);
        });
    }
  } catch (err) {
    console.error("Failed to initialize server:", err);
    error.value = err as Error;
    server.value = null as never;
  } finally {
    isLoading.value = false;
  }
}

watch(
  () => server.value?.moduleErrors?.general,
  (generalError) => {
    if (server.value?.general?.status === "suspended") {
      return;
    }

    if (generalError?.error && generalError.error.statusCode !== 403) {
      error.value = generalError.error;
    }
  },
  { deep: true },
);

provideModrinthServerContext({
  server: server as Ref<ModrinthServer | null>,
});

const generalErrorAction = computed(() => ({
  label: "Go back to all servers",
  onClick: () => router.push("/servers/manage"),
  color: "brand" as const,
}));

onMounted(() => {
  initializeServer();
});

onUnmounted(() => {
  server.value = null;
});

watch(
  () => serverId,
  () => {
    initializeServer();
  },
  { immediate: false },
);
</script>
