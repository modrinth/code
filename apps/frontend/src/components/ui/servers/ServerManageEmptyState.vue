<template>
  <div class="flex h-full flex-col items-center justify-center gap-8">
    <img
      src="https://cdn.modrinth.com/servers/excitement.webp"
      alt=""
      class="max-w-[360px]"
      style="mask-image: radial-gradient(97% 77% at 50% 25%, #d9d9d9 0, hsla(0, 0%, 45%, 0) 100%)"
    />
    <h1 class="m-0 text-contrast">You don't have any servers yet!</h1>
    <p class="m-0">Modrinth Servers is a new way to play modded Minecraft with your friends.</p>
    <ButtonStyled size="large" type="standard" color="brand">
      <NuxtLink to="/servers#plan">Create a Server</NuxtLink>
    </ButtonStyled>
  </div>
</template>

<script setup lang="ts">
import { ButtonStyled } from "@modrinth/ui";
</script>
