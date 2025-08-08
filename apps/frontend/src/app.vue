<template>
  <NuxtLayout>
    <ModrinthLoadingIndicator />
    <NotificationPanel />
    <NuxtPage />
  </NuxtLayout>
</template>
<script setup lang="ts">
import { NotificationPanel, provideNotificationManager } from "@modrinth/ui";
import { FrontendNotificationManager } from "./providers/frontend-notifications.ts";
import ModrinthLoadingIndicator from "~/components/ui/modrinth-loading-indicator.ts";

provideNotificationManager(new FrontendNotificationManager());
</script>
