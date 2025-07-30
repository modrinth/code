<template>
  <NuxtLayout>
    <ModrinthLoadingIndicator />
    <NotificationPanel />
    <NuxtPage />
  </NuxtLayout>
</template>
<script setup lang="ts">
import { NotificationPanel, provideNotificationManager } from "@modrinth/ui";
import ModrinthLoadingIndicator from "~/components/ui/modrinth-loading-indicator.ts";
import { FrontendNotificationManager } from "./providers/web-notifications";

provideNotificationManager(new FrontendNotificationManager());
</script>
