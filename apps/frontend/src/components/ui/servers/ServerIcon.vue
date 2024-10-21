<template>
  <client-only>
    <img
      v-if="image"
      alt="Server Icon"
      class="experimental-styles-within size-24 rounded-xl border-[1px] border-solid border-button-border bg-button-bg shadow-sm"
      :src="image"
    />
    <img
      v-else
      alt="Server Icon"
      class="size-24 rounded-xl border-[1px] border-solid border-button-border bg-button-bg shadow-sm"
      src="~/assets/images/servers/minecraft_server_icon.png"
    />
  </client-only>
</template>

<script setup lang="ts">
defineProps<{
  image: string | undefined;
}>();
</script>
