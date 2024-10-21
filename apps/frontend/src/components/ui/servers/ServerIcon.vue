<template>
  <client-only>
    <img v-if="image" alt="Server Icon" :class="computedClass" :src="image" />
    <img
      v-else
      alt="Server Icon"
      :class="computedClass"
      src="~/assets/images/servers/minecraft_server_icon.png"
    />
  </client-only>
</template>

<script setup lang="ts">
const props = defineProps<{
  image: string | undefined;
  class?: string;
}>();

const computedClass = computed(() => {
  return (
    props.class ||
    "h-[6rem] w-[6rem] rounded-xl bg-bg-raised border-[1px] border-solid border-button-bg"
  );
});
</script>
