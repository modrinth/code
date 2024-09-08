<template>
  <UiServersServerSidebar :route="route" :nav-links="navLinks" />
</template>

<script setup lang="ts">
import { CogIcon, FileIcon, ListIcon, TextQuoteIcon, VersionIcon } from "@modrinth/assets";

const route = useNativeRoute();
const serverId = route.params.id;

const navLinks = [
  { icon: CogIcon, label: "General", href: `/servers/manage/${serverId}/options` },
  { icon: TextQuoteIcon, label: "Startup", href: `/servers/manage/${serverId}/options/startup` },
  { icon: VersionIcon, label: "Network", href: `/servers/manage/${serverId}/options/network` },
  { icon: ListIcon, label: "Properties", href: `/servers/manage/${serverId}/options/properties` },
  { icon: FileIcon, label: "Info", href: `/servers/manage/${serverId}/options/info` },
];
</script>
