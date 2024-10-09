<template>
  <client-only>
    <img
      v-if="serverImage"
      no-shadow
      size="lg"
      alt="Server Icon"
      :class="computedClass"
      :src="serverImage"
    />
    <img
      v-else
      no-shadow
      size="lg"
      alt="Server Icon"
      :class="computedClass"
      src="~/assets/images/servers/minecraft_server_icon.png"
    />
  </client-only>
</template>

<script setup lang="ts">
const props = defineProps<{
  serverId: string,
  class?: string,
}>();

const computedClass = computed(() => {
  return props.class || "h-[6rem] w-[6rem] rounded-xl bg-bg-raised";
});

const serverStore = useServerStore();
const serverImage = computed(() => {
  const serverData = serverStore.serverData[props.serverId];
  const image = serverData?.image;
  return typeof image === "string" ? image : "";
});
</script>
