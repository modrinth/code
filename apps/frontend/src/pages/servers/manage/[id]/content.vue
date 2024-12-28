<template>
  <div class="flex h-full w-full flex-col">
    <NuxtPage :route="route" :server="props.server" />
  </div>
</template>

<script setup lang="ts">
import type { Server } from "~/composables/pyroServers";

const route = useNativeRoute();

const props = defineProps<{
  server: Server<["general", "content", "backups", "network", "startup", "ws", "fs"]>;
}>();

const data = computed(() => props.server.general);

useHead({
  title: `Content - ${data.value?.name ?? "Server"} - Modrinth`,
});
</script>
