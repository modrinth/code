<template>
  <div
    class="experimental-styles-within flex size-24 shrink-0 overflow-hidden rounded-xl border-[1px] border-solid border-button-border bg-button-bg shadow-sm"
  >
    <client-only>
      <img
        v-if="image"
        class="h-full w-full select-none object-fill"
        alt="Server Icon"
        :src="image"
      />
      <img
        v-else
        class="h-full w-full select-none object-fill"
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
