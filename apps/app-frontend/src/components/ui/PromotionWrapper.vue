<script setup lang="ts">
import { ref } from 'vue'
import { Promotion } from '@modrinth/ui'
import { get as getCreds } from '@/helpers/mr_auth.js'
import { handleError } from '@/store/notifications.js'
import { get_user } from '@/helpers/cache.js'

const showAd = ref(true)

const creds = await getCreds().catch(handleError)
if (creds && creds.user_id) {
  console.log(creds)
  const user = await get_user(creds.user_id).catch(handleError)
  console.log(user)

  const MIDAS_BITFLAG = 1 << 0
  if (user && (user.badges & MIDAS_BITFLAG) === MIDAS_BITFLAG) {
    showAd.value = false
  }
}
</script>

<template>
  <Promotion v-if="showAd" :external="false" query-param="?r=launcher" />
</template>
