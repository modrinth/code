<template>
  <div class="account-dropdown">
    <OverflowMenu
      class="btn btn-transparent headless-button"
      :options="[
        {
          id: 'play',
          color: 'danger',
          action: () => {},
          hoverFilledOnly: true,
        },
      ]"
      direction="up"
      position="right"
    >
      <Avatar circle size="sm" :src="credentials?.user?.avatar_url" />
      <template #play> <LogOutIcon /> Sign out </template>
    </OverflowMenu>
  </div>
</template>

<script setup>
import { onMounted, ref } from 'vue'
import { Avatar, OverflowMenu, LogOutIcon } from 'omorphia'
import { get as getCredentials } from '@/helpers/mr_auth.js'
import { useNotifications } from '@/store/notifications.js'

const notifs = useNotifications()

const credentials = ref(null)

const refreshCredentials = async () => {
  try {
    credentials.value = await getCredentials()
  } catch (error) {
    notifs.addNotification({
      title: 'An error occurred',
      text: error.message ?? error,
      type: 'error',
    })
    console.error(error)
  }
}

onMounted(async () => {
  await refreshCredentials()
})
</script>

<style lang="scss">
.account-dropdown {
  *.headless-button {
    padding: 0 !important;
  }
}
</style>
