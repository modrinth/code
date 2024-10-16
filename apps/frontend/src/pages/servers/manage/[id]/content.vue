<template>
  <div class="flex h-full w-full flex-col gap-4">
    <div
      class="flex h-full w-full items-center gap-2 rounded-xl border border-solid border-blue bg-bg-blue p-4 text-contrast"
    >
      <UnknownIcon class="h-8 w-8 text-blue" />
      This page is under construction some features may not be available yet, or are subject to
      change.
    </div>
    <UiServersServerSidebar :route="route" :nav-links="navLinks" :server="props.server" />
  </div>
</template>

<script setup lang="ts">
import { BookIcon, BoxIcon, UnknownIcon } from "@modrinth/assets";

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

const navLinks = computed(() => {
  if (data.value?.loader === "Vanilla") {
    return [
      { icon: BookIcon, label: "Datapacks", href: `/servers/manage/${serverId}/content/datapacks` },
    ];
  } else {
    return [
      { icon: BoxIcon, label: "Mods", href: `/servers/manage/${serverId}/content` },
      { icon: BookIcon, label: "Datapacks", href: `/servers/manage/${serverId}/content/datapacks` },
    ];
  }
});
</script>
