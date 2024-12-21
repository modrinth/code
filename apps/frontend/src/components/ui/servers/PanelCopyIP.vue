<template>
  <ButtonStyled type="standard">
    <button aria-label="Copy server IP" @click="copyText">
      <CopyIcon />
      Copy IP
    </button>
  </ButtonStyled>
</template>

<script setup lang="ts">
import { CopyIcon } from "@modrinth/assets";
import { ButtonStyled } from "@modrinth/ui";

const props = defineProps<{
  ip: string;
  port: number;
  subdomain?: string | null;
}>();

const copyText = () => {
  const text = props.subdomain ? `${props.subdomain}.modrinth.gg` : `${props.ip}:${props.port}`;
  navigator.clipboard.writeText(text);

  addNotification({
    group: "server",
    title: `Copied IP`,
    text: `Your server's IP has been copied to your clipboard`,
    type: "success",
  });
};
</script>
