<template>
  <div>
    <div v-if="data && status === 'success'">
      <UiServersServerSidebar :route="route" :nav-links="navLinks" />
    </div>
    <UiServersPyroError
      v-else-if="status === 'error'"
      title="Error Accessing Server"
      message="Dont worry, your server is safe. We just can't connect to it right now."
    />
    <UiServersPyroLoading v-else />
  </div>
</template>

<script setup lang="ts">
import { BoxIcon, FileIcon } from "@modrinth/assets";

const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();

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
