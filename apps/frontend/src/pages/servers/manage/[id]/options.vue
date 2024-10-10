<template>
  <UiServersServerSidebar :route="route" :nav-links="navLinks" />
</template>

<script setup lang="ts">
import {
  InfoIcon,
  ListIcon,
  LoaderIcon,
  SettingsIcon,
  TextQuoteIcon,
  VersionIcon,
} from "@modrinth/assets";

const route = useNativeRoute();
const serverId = route.params.id.toString();
const serverStore = useServerStore();

useHead({
  title: `Options - ${serverStore.serverData[serverId]?.name ?? "Server"} - Modrinth`,
});

const navLinks = [
  { icon: SettingsIcon, label: "General", href: `/servers/manage/${serverId}/options` },
  { icon: LoaderIcon, label: "Loader", href: `/servers/manage/${serverId}/options/loader` },
  { icon: TextQuoteIcon, label: "Startup", href: `/servers/manage/${serverId}/options/startup` },
  { icon: VersionIcon, label: "Network", href: `/servers/manage/${serverId}/options/network` },
  { icon: ListIcon, label: "Properties", href: `/servers/manage/${serverId}/options/properties` },
  { icon: InfoIcon, label: "Info", href: `/servers/manage/${serverId}/options/info` },
];
</script>
