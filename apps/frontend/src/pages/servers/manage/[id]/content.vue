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
const server = await usePyroServer(serverId, ["general", "mods"]);

const data = computed(() => server.general);

useHead({
  title: `Content - ${data.value?.name ?? "Server"} - Modrinth`,
});

const navLinks = [
  { icon: BoxIcon, label: "Mods", href: `/servers/manage/${serverId}/content` },
  { icon: BookIcon, label: "Datapacks", href: `/servers/manage/${serverId}/content/datapacks` },
];
</script>
