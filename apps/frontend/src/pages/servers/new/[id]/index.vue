<template>
  <div class="flex h-screen items-center justify-center">
    <div v-if="!loading" class="flex flex-col items-center gap-4">
      <div
        v-if="loader"
        class="flex w-full items-center gap-1 rounded-xl bg-bg-raised p-2 pr-4 text-2xl font-bold text-contrast"
      >
        <UiServersLoaderIcon
          :loader="
            packId as string
            // e
          "
          class="[&&]:size-10"
        />
        {{ packId }}
      </div>
      <div class="project-list display-mode--list" v-else>
        <UiProjectCard
          v-if="pack"
          :id="packId"
          :key="pack.project_id"
          :featured-image="pack.featured_gallery ? pack.featured_gallery : pack.gallery[0]"
          :type="pack.project_type"
          :author="pack.author"
          :name="pack.title"
          :description="pack.description"
          :created-at="pack.date_created"
          :updated-at="pack.date_modified"
          :icon-url="pack.icon_url"
          :client-side="pack.client_side"
          :server-side="pack.server_side"
          :categories="pack.display_categories"
          :show-updated-date="true"
          :color="pack.color"
        />
      </div>
      <div class="flex w-full items-center justify-between gap-2 rounded-xl bg-bg-raised p-2 pr-4">
        <input
          type="text"
          class="rounded border p-2 [&&]:w-full"
          :value="serverName"
          @input="updateServerName($event)"
        />
        <Button icon-only @click="() => createServer()">
          <ChevronRightIcon />
        </Button>
      </div>
    </div>
    <UiServersPyroLoading v-else />
  </div>
</template>

<script setup lang="ts">
import { PyroIcon, ChevronRightIcon } from "@modrinth/assets";
import { Button } from "@modrinth/ui";

const config = useRuntimeConfig();
const auth = (await useAuth()) as any;
const route = useNativeRoute();
let loading = false;
const packId = route.params.id;
const version_id = route.query.version;
const serverName = ref("");

const updateServerName = (event: Event) => {
  serverName.value = (event.target as HTMLInputElement).value;
};

interface IntServer {
  uuid: string;
  ip: string;
  port: number;
}

const loaders = ["Forge", "Fabric", "Neoforge", "Quilt", "Vanilla"];

const loader = loaders.includes(packId as string);

const pack = loader
  ? null
  : ((await toRaw(
      useBaseFetch(`project/${Array.isArray(packId) ? packId[0] : packId}`, {}, false, true),
    )) as any);

const createServer = async () => {
  loading = true;

  let path = "servers/create";
  const version = 0;
  const body = {
    name: serverName.value,
    specs: {
      cpu: 4.0,
      memory_mb: 4192,
      swap_mb: 4192,
    },
    source: loader
      ? {
          loader: packId,
          version: "1.20.1-47.3.7",
        }
      : {
          project_id: packId,
          version_id: version_id ? version_id : pack.versions.slice(-1)[0],
        },
    user_id: auth.value?.user?.id ?? "",
  };
  const method = "POST";

  const timeout = 10000;
  const retryAmount = 3;

  let base = import.meta.server ? config.pyroBaseUrl : config.public.pyroBaseUrl;

  if (!base) {
    throw new Error(
      "Cannot pyrofetch without base url. Make sure to set a PYRO_BASE_URL in environment variables (10001)",
    );
  }

  if (base.endsWith("/")) {
    base = base.slice(0, -1);
  }

  if (path.startsWith("/")) {
    path = path.slice(1);
  }

  const fullUrl: string = `${base}/modrinth/v${version}/${path}`;

  const request: any = {
    method,
    headers: {
      Accept: "application/json",
      "ngrok-skip-browser-warning": "true",
      "X-Pinggy-No-Screen": "true",
      "X-Master-Key": "IytvaAQD3BgancASUHKjlUn3dXG4Gz6c8v2bIHrO2uytV4kqLujCt292FqEXWXb1",
      "Access-Control-Allow-Origin": "*",
      "Access-Control-Allow-Methods": "*",
      "User-Agent": "Pyro/1.0 (https://pyro.host)",
      "Content-Type": "application/json",
      retry: 0,
      Vary: "Accept",
    },
    timeout,
    retry: retryAmount,
  };

  request.body = JSON.stringify(body);

  const server: IntServer = await $fetch(fullUrl, request);

  const serverId = server.uuid;

  await new Promise((resolve) => setTimeout(resolve, 3000));

  // redirect to the new server
  await navigateTo(`/servers/manage/${serverId}`);
};
</script>
