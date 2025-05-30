<template>
  <div
    v-if="subdomain && !isHidden"
    v-tooltip="'Copy custom URL'"
    class="flex min-w-0 flex-row items-center gap-4 truncate hover:cursor-pointer"
  >
    <div v-if="!noSeparator" class="experimental-styles-within h-6 w-0.5 bg-button-border"></div>
    <div class="flex flex-row items-center gap-2">
      <LinkIcon class="flex size-5 shrink-0" />
      <div
        class="flex min-w-0 text-sm font-semibold"
        :class="serverId ? 'hover:underline' : ''"
        @click="copySubdomain"
      >
        {{ subdomain }}.modrinth.gg
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { LinkIcon } from "@modrinth/assets";
import { useStorage } from "@vueuse/core";

const props = defineProps<{
  subdomain: string;
  noSeparator?: boolean;
}>();

const copySubdomain = () => {
  navigator.clipboard.writeText(props.subdomain + ".modrinth.gg");
  addNotification({
    group: "servers",
    title: "Custom URL copied",
    text: "Your server's URL has been copied to your clipboard.",
    type: "success",
  });
};

const route = useNativeRoute();
const serverId = computed(() => route.params.id as string);

const userPreferences = useStorage(`pyro-server-${serverId.value}-preferences`, {
  hideSubdomainLabel: false,
});

const isHidden = computed(() => userPreferences.value.hideSubdomainLabel);
</script>
