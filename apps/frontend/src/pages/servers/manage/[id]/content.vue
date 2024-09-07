<template>
  <div>
    <div v-if="data && status === 'success'">
      <ServerSidebar :route="route" :navLinks="navLinks" />
    </div>
    <UiServersPyroError
      v-else-if="status === 'error'"
      title="Error Accessing Server"
      message="Dont worry, your server is safe. We just can't connect to it right now."
    />
    <PyroLoading v-else />
  </div>
</template>

<script setup lang="ts">
import { EditIcon, ImportIcon, BoxIcon, SearchIcon, FileIcon } from "@modrinth/assets";
import LoaderIcon from "~/components/ui/servers/LoaderIcon.vue";
import { useServerStore } from "~/stores/servers.ts";
import PyroLoading from "~/components/ui/servers/PyroLoading.vue";
import ServerSidebar from "~/components/ui/servers/ServerSidebar.vue";

const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();
const auth = await useAuth();

await serverStore.fetchServerData(serverId);
const { data, status } = await useLazyAsyncData(
  "contentServerData",
  async () => await serverStore.getServerData(serverId),
);

const navLinks = [
  { icon: BoxIcon, label: "Mods", href: `/servers/manage/${serverId}/content` },
  { icon: FileIcon, label: "Files", href: `/servers/manage/${serverId}/content/files` },
];
</script>
