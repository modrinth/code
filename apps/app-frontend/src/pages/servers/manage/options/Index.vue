<template>
  <div>a</div>
</template>

<script setup lang="ts">
import type { Server } from '@/types/servers'
import type { Credentials } from '../Index.vue'

defineProps<{
  server: Server
  credentials: Credentials
}>()
</script>
