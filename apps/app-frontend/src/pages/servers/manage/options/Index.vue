<template>
  <div>a</div>
</template>

<script setup lang="ts">
import type { Server } from '@/types/servers'
import type { Credentials } from '@/store/credentials'

defineProps<{
  server: Server
  credentials: Credentials
}>()
</script>
