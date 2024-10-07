<template>
  <client-only>
    <img
      v-if="serverImage"
      no-shadow
      size="lg"
      alt="Server Icon"
      class="h-[6rem] w-[6rem] rounded-xl bg-bg-raised"
      :src="serverImage"
    />
    <img
      v-else
      no-shadow
      size="lg"
      alt="Server Icon"
      class="h-[6rem] w-[6rem] rounded-xl bg-bg-raised"
      src="~/assets/images/servers/minecraft_server_icon.png"
    />
  </client-only>
</template>

<script setup lang="ts">
const props = defineProps({
  serverId: {
    type: String,
    required: true,
  },
});

const serverStore = useServerStore();
const serverImage = computed(() => {
  const serverData = serverStore.serverData[props.serverId];
  const image = serverData?.image;
  return typeof image === "string" ? image : "";
});
</script>
