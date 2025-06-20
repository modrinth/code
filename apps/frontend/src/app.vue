<template>
  <NuxtLayout>
    <ModrinthLoadingIndicator />
    <Notifications />
    <NuxtPage />
  </NuxtLayout>
</template>
<script setup lang="ts">
import { provideNotificationManager } from "@modrinth/ui";
import { FrontendNotificationManager } from "./helpers/web-notifications.ts";
import ModrinthLoadingIndicator from "~/components/ui/modrinth-loading-indicator.ts";
import Notifications from "~/components/ui/Notifications.vue";

provideNotificationManager(new FrontendNotificationManager());
</script>
