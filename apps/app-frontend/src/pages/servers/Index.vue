<template>
  <div>
    Hi servers!
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { handleError } from '@/store/state'
import { get as getCreds } from '@/helpers/mr_auth.js'
import { get_user } from '@/helpers/cache.js'

async function fetchCredentials() {
  const creds = await getCreds().catch(handleError)
  if (creds && creds.user_id) {
    creds.user = await get_user(creds.user_id).catch(handleError)
  }
  credentials.value = creds
}
const credentials = ref()
await fetchCredentials()
</script>
