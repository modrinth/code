<template>
  <UiServersServerSidebar :route="route" :nav-links="navLinks" :server="props.server" />
</template>

<script setup lang="ts">
import { BookIcon, BoxIcon } from "@modrinth/assets";

import type { Server } from "~/composables/pyroServers";

const route = useNativeRoute();
const serverId = route.params.id as string;

const props = defineProps<{
  server: Server<["general", "mods", "backups", "network", "startup", "ws", "fs"]>;
}>();

const data = computed(() => props.server.general);

useHead({
  title: `Content - ${data.value?.name ?? "Server"} - Modrinth`,
});

const navLinks = [
  { icon: BoxIcon, label: "Mods", href: `/servers/manage/${serverId}/content` },
  { icon: BookIcon, label: "Datapacks", href: `/servers/manage/${serverId}/content/datapacks` },
];
</script>
