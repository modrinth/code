<template>
  <div
    v-if="subdomain"
    v-tooltip="'Copy subdomain'"
    class="flex flex-row items-center gap-4 hover:cursor-pointer"
  >
    <div class="experimental-styles-within h-6 w-0.5 bg-button-border"></div>
    <div class="flex flex-row items-center gap-2">
      <LinkIcon />
      <div
        class="text-sm font-semibold"
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
const props = defineProps<{
  subdomain: string;
}>();

const copySubdomain = () => {
  navigator.clipboard.writeText(props.subdomain + ".modrinth.gg");
  addNotification({
    group: "servers",
    title: "Subdomain copied",
    text: "Your subdomain has been copied to your clipboard.",
    type: "success",
  });
};

const route = useNativeRoute();
const serverId = computed(() => route.params.id as string);
</script>
