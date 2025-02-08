<template>
  <UiServersServerSidebar :route="route" :nav-links="navLinks" :server="props.server" />
</template>

<script setup lang="ts">
import {
  InfoIcon,
  ListIcon,
  SettingsIcon,
  TextQuoteIcon,
  VersionIcon,
  CardIcon,
  ModrinthIcon,
  UserIcon,
  WrenchIcon,
} from "@modrinth/assets";
import { isAdmin as isUserAdmin, type User } from "@modrinth/utils";
import type { Server } from "~/composables/pyroServers";

const route = useRoute();
const serverId = route.params.id as string;
const auth = await useAuth();

const props = defineProps<{
  server: Server<["general", "content", "backups", "network", "startup", "ws", "fs"]>;
}>();

useHead({
  title: `Options - ${props.server.general?.name ?? "Server"} - Modrinth`,
});

const ownerId = computed(() => `Dc7EYhxG`);
const isOwner = computed(() => (auth.value?.user as User | null)?.id === ownerId.value);
const isAdmin = computed(() => isUserAdmin(auth.value?.user));

const navLinks = computed(() => [
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
    shown: isOwner.value,
  },
  {
    icon: ModrinthIcon,
    label: "Admin Billing",
    href: `/admin/billing/${ownerId.value}`,
    external: true,
    shown: isAdmin.value,
  },
  { icon: InfoIcon, label: "Info", href: `/servers/manage/${serverId}/options/info` },
]);
</script>
