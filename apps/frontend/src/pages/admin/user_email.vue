<template>
  <div class="normal-page no-sidebar">
    <h1>User account request</h1>
    <div class="normal-page__content">
      <div class="card flex flex-col gap-3">
        <div class="flex flex-col gap-2">
          <label for="name">
            <span class="text-lg font-semibold text-contrast">
              User email
              <span class="text-brand-red">*</span>
            </span>
          </label>
          <input
            id="name"
            v-model="userEmail"
            type="email"
            maxlength="64"
            :placeholder="`Enter user email...`"
            autocomplete="off"
          />
        </div>
        <div class="flex gap-2">
          <ButtonStyled color="brand">
            <button @click="getUserFromEmail">
              <MailIcon aria-hidden="true" />
              Get user account
            </button>
          </ButtonStyled>
        </div>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { ButtonStyled } from "@modrinth/ui";
import { MailIcon } from "@modrinth/assets";

const userEmail = ref("");

async function getUserFromEmail() {
  startLoading();

  try {
    const result = await useBaseFetch(`user_email?email=${encodeURIComponent(userEmail.value)}`, {
      method: "GET",
      apiVersion: 3,
    });

    await navigateTo(`/user/${result.username}`);
  } catch (err) {
    console.error(err);
    addNotification({
      group: "main",
      title: "An error occurred",
      text: err.data.description,
      type: "error",
    });
  }
  stopLoading();
}
</script>
