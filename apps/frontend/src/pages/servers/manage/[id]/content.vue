<template>
  <div class="contents">
    <div v-if="data">
      <UiServersServerSidebar :route="route" :nav-links="navLinks" />
    </div>
    <UiServersPyroLoading v-else />
  </div>
</template>

<script setup lang="ts">
import { BookIcon, BoxIcon } from "@modrinth/assets";

const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();

const { data } = await useLazyAsyncData("serverData", async () => {
  await serverStore.fetchServerData(serverId);
  const serverData = serverStore.serverData[serverId];

  useHead({
    title: `Content - ${serverData?.name ?? "Server"} - Modrinth`,
  });

  return serverData;
});

const navLinks = [
  { icon: BoxIcon, label: "Mods", href: `/servers/manage/${serverId}/content` },
  { icon: BookIcon, label: "Datapacks", href: `/servers/manage/${serverId}/content/datapacks` },
];
</script>
