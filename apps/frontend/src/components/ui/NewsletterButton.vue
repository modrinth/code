<script setup lang="ts">
import { ButtonStyled } from "@modrinth/ui";
import { MailIcon, CheckIcon } from "@modrinth/assets";
import { ref, watchEffect } from "vue";
import { useBaseFetch } from "~/composables/fetch.js";

const auth = await useAuth();
const showSubscriptionConfirmation = ref(false);
const subscribed = ref(false);

async function checkSubscribed() {
  if (auth.value?.user) {
    try {
      const { data } = await useBaseFetch("auth/email/subscribe", {
        method: "GET",
      });
      subscribed.value = data?.subscribed || false;
    } catch {
      subscribed.value = false;
    }
  }
}

watchEffect(() => {
  checkSubscribed();
});

async function subscribe() {
  try {
    await useBaseFetch("auth/email/subscribe", {
      method: "POST",
    });
    showSubscriptionConfirmation.value = true;
  } catch {
  } finally {
    setTimeout(() => {
      showSubscriptionConfirmation.value = false;
      subscribed.value = true;
    }, 2500);
  }
}
</script>

<template>
  <ButtonStyled v-if="auth?.user && !subscribed" color="brand" type="outlined">
    <button v-tooltip="`Subscribe to the Modrinth newsletter`" @click="subscribe">
      <template v-if="!showSubscriptionConfirmation"> <MailIcon /> Subscribe </template>
      <template v-else> <CheckIcon /> Subscribed! </template>
    </button>
  </ButtonStyled>
</template>
