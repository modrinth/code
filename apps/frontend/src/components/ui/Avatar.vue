<template>
  <OmorphiaAvatar
    :src="src"
    :alt="alt"
    :size="size"
    :circle="circle"
    :no-shadow="noShadow"
    :loading="loading"
    :raised="raised"
  />
</template>

<script setup>
import { Avatar as OmorphiaAvatar } from "@modrinth/ui";

const props = defineProps({
  src: {
    type: String,
    default: null,
  },
  alt: {
    type: String,
    default: "",
  },
  size: {
    type: String,
    default: "2rem",
  },
  circle: {
    type: Boolean,
    default: false,
  },
  noShadow: {
    type: Boolean,
    default: false,
  },
  loading: {
    type: String,
    default: "eager",
  },
  raised: {
    type: Boolean,
    default: false,
  },
});
</script>
