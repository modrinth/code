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

useHead({
  title: `Content - ${serverStore.serverData[serverId]?.name ?? "Server"} - Modrinth`,
});

await serverStore.fetchServerData(serverId);
const data = computed(() => serverStore.serverData[serverId]);

const navLinks = [
  { icon: BoxIcon, label: "Mods", href: `/servers/manage/${serverId}/content` },
  { icon: BookIcon, label: "Datapacks", href: `/servers/manage/${serverId}/content/datapacks` },
];
</script>
