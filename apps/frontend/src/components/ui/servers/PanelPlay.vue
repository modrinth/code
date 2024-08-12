<template>
  <a class="btn btn-primary" :href="playUrl">
    <PlayIcon />
    Play
  </a>
</template>

<script lang="ts">
import { defineComponent, computed } from "vue";
import { PlayIcon } from "@modrinth/assets";

export default defineComponent({
  name: "PlayButton",
  components: {
    PlayIcon,
  },
  props: {
    serverId: {
      type: String,
      required: true,
    },
    ip: {
      type: String,
      required: true,
    },
    port: {
      type: Number,
      required: true,
    },
  },
  setup(props) {
    const playUrl = computed(
      () => `modrinth://servers/${props.serverId}/${props.ip}:${props.port}`,
    );

    return { playUrl };
  },
});
</script>
