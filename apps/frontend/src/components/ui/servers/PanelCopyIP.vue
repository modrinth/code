<template>
  <button aria-label="Copy server IP" class="btn btn-transparent text-sm" @click="copyText">
    <CopyIcon />
    Copy IP
  </button>
</template>

<script setup lang="ts">
import { CopyIcon } from "@modrinth/assets";
import { useNuxtApp } from "#app";

const app = useNuxtApp();

const props = defineProps<{
  ip: string;
  port: number;
  subdomain?: string | null;
}>();

const copyText = () => {
  const text = props.subdomain ? `${props.subdomain}.modrinth.gg` : `${props.ip}:${props.port}`;
  navigator.clipboard.writeText(text);

  // @ts-ignore
  app.$notify({
    group: "server",
    title: `Copied IP`,
    text: `Your server's IP has been copied to your clipboard`,
    type: "success",
  });
};
</script>
