<template>
  <div class="account-dropdown">
    <Modal
      ref="modrinthLoginModal"
      class="login-screen-modal"
      :noblur="!themeStore.advancedRendering"
    >
      <ModrinthLoginScreen :modal="true" :prev-page="signInAfter" :next-page="signInAfter" />
    </Modal>
    <OverflowMenu
      v-if="mrAuth.auth.value?.user"
      class="btn btn-transparent headless-button"
      :options="[
        {
          id: 'sign-out',
          color: 'danger',
          action: () => {},
          hoverFilledOnly: true,
        },
      ]"
      direction="up"
      position="right"
    >
      <Avatar circle size="sm" :src="mrAuth.auth.value?.user?.avatar_url" />
      <template #sign-out> <LogOutIcon /> Sign out </template>
    </OverflowMenu>
    <OverflowMenu
      v-else
      class="btn btn-transparent headless-button"
      :options="[
        {
          id: 'sign-in',
          color: 'primary',
          action: () => {
            modrinthLoginModal?.show()
          },
        },
      ]"
      direction="up"
      position="right"
    >
      <Avatar circle size="sm" />
      <template #sign-in> <LogInIcon /> Sign in </template>
    </OverflowMenu>
  </div>
</template>

<script setup>
import { onMounted, ref } from 'vue'
import { Avatar, OverflowMenu, LogOutIcon, LogInIcon, Modal } from 'omorphia'

import { useTheming } from '@/store/state'
import ModrinthLoginScreen from '@/components/ui/tutorial/ModrinthLoginScreen.vue'
import { useMrAuth } from '@/composables/auth.js'

const themeStore = useTheming()

const mrAuth = useMrAuth()

const modrinthLoginModal = ref(null)

const refreshCredentials = async () => {
  await mrAuth.get()
}

onMounted(async () => {
  await refreshCredentials()
})

const signInAfter = async () => {
  modrinthLoginModal.value?.hide()
  await refreshCredentials()
}
</script>

<style scoped lang="scss">
:deep {
  .headless-button {
    padding: 0 !important;
    border-radius: 99999px;
  }

  .login-screen-modal {
    .modal-container .modal-body {
      width: auto;

      .content {
        background: none;
      }
    }
  }
}
</style>
