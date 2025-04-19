<template>
  <UiServersServerSidebar
    :route="route"
    :nav-links="navLinks"
    :server="server"
    :backup-in-progress="backupInProgress"
  />
</template>
<script setup lang="ts">
import {
  InfoIcon,
  ListIcon,
  SettingsIcon,
  TextQuoteIcon,
  VersionIcon,
  CardIcon,
  UserIcon,
  WrenchIcon,
} from "@modrinth/assets";
import type { Server } from "~/composables/pyroServers";
import type { BackupInProgressReason } from "~/pages/servers/manage/[id].vue";

const route = useRoute();
const serverId = route.params.id as string;

const props = defineProps<{
  server: Server<["general", "content", "backups", "network", "startup", "ws", "fs"]>;
  backupInProgress?: BackupInProgressReason;
}>();

useHead({
  title: `Options - ${props.server.general?.name ?? "Server"} - Modrinth`,
});

const navLinks = [
  { icon: SettingsIcon, label: "General", href: `/servers/manage/${serverId}/options` },
  { icon: WrenchIcon, label: "Platform", href: `/servers/manage/${serverId}/options/loader` },
  { icon: TextQuoteIcon, label: "Startup", href: `/servers/manage/${serverId}/options/startup` },
  { icon: VersionIcon, label: "Network", href: `/servers/manage/${serverId}/options/network` },
  { icon: ListIcon, label: "Properties", href: `/servers/manage/${serverId}/options/properties` },
  {
    icon: UserIcon,
    label: "Preferences",
    href: `/servers/manage/${serverId}/options/preferences`,
  },
  {
    icon: CardIcon,
    label: "Billing",
    href: `/settings/billing#server-${serverId}`,
    external: true,
  },
  { icon: InfoIcon, label: "Info", href: `/servers/manage/${serverId}/options/info` },
];
</script>
