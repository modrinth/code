<template>
  <NuxtLayout>
    <ModrinthLoadingIndicator />
    <Notifications />
    <NuxtPage />
  </NuxtLayout>
</template>
<script setup lang="ts">
import { provideModrinthServersConsole, provideNotificationManager } from "@modrinth/ui";
import { FrontendNotificationManager } from "./providers/web-notifications.ts";
import { FrontendModrinthServersConsole } from "./providers/servers-console.ts";
import ModrinthLoadingIndicator from "~/components/ui/modrinth-loading-indicator.ts";
import Notifications from "~/components/ui/Notifications.vue";

provideNotificationManager(new FrontendNotificationManager());
provideModrinthServersConsole(new FrontendModrinthServersConsole());
</script>
