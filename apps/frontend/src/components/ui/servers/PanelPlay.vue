<template>
  <a class="btn btn-primary" :href="playUrl">
    <PlayIcon />
    Play
  </a>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { PlayIcon } from "@modrinth/assets";

const props = defineProps<{
  serverId: string;
  ip: string;
  port: number;
}>();

const playUrl = computed(() => `modrinth://servers/${props.serverId}/${props.ip}:${props.port}`);
</script>
