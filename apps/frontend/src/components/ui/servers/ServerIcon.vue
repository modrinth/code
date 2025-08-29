<template>
  <div
    class="experimental-styles-within border-button-border bg-button-bg flex size-24 shrink-0 overflow-hidden rounded-xl border shadow-xs"
  >
    <client-only>
      <img
        v-if="image"
        class="h-full w-full object-fill select-none"
        alt="Server Icon"
        :src="image"
      />
      <img
        v-else
        class="h-full w-full object-fill select-none"
        alt="Server Icon"
        src="~/assets/images/servers/minecraft_server_icon.png"
      />
    </client-only>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  image: string | undefined;
}>();
</script>
