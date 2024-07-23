<template>
  <div class="flex h-screen items-center justify-center">
    <div v-if="!loading" class="flex flex-col items-center gap-4">
      <div class="project-list display-mode--list">
        <ProjectCard
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
      <input
        type="text"
        class="rounded border p-2"
        :value="serverName"
        @input="updateServerName($event)"
      />
      <button type="submit" class="btn btn-primary" @click="() => createServer()">Create</button>
    </div>
    <PyroIcon v-else class="pyro-logo-animation size-10" />
  </div>
</template>

<script setup lang="ts">
import { PyroIcon } from "@modrinth/assets";
import ProjectCard from "~/components/ui/ProjectCard.vue";

const auth = await useAuth();
const route = useNativeRoute();
let loading = false;
const packId = route.params.id;
const serverName = ref("");

const pack: any = await toRaw(useBaseFetch(`project/${packId}`));

const updateServerName = (event: Event) => {
  serverName.value = (event.target as HTMLInputElement).value;
};

interface IntServer {
  uuid: string;
  ip: string;
  port: number;
}

const createServer = async () => {
  loading = true;
  const server: IntServer = await usePyroFetch(
    auth.value.token,
    `servers/create`,
    0,
    "POST",
    "application/json",
    {
      name: serverName.value,
      specs: {
        cpu: 4.0,
        memory_mb: 8192,
        swap_mb: 8192,
      },
      source: {
        modrinthid: pack.versions.slice(-1)[0],
      },
    },
  );
  const serverId = server.uuid;

  await new Promise((resolve) => setTimeout(resolve, 3000));

  // redirect to the new server
  await navigateTo(`/servers/manage/${serverId}`);
};
</script>

<style>
@keyframes zoom-in-out {
  0% {
    transform: scale(1);
  }

  50% {
    transform: scale(1.4);
  }

  100% {
    transform: scale(1);
  }
}

.pyro-logo-animation {
  animation: zoom-in-out 2s cubic-bezier(0.175, 0.885, 0.32, 1.275) infinite;
}
</style>
