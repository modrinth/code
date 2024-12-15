<template>
  <div class="universal-card">
    <p>You can manage your server's billing from Settings > Billing and subscriptions.</p>
    <ButtonStyled>
      <NuxtLink to="/settings/billing">Go to Billing</NuxtLink>
    </ButtonStyled>
  </div>
</template>

<script setup lang="ts">
import { ButtonStyled } from "@modrinth/ui";
</script>
