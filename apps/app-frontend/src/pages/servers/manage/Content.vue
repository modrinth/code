<template>
  <pre>{{ server }}</pre>
  {{ credentials }}
</template>

<script setup lang="ts">
import type { Credentials } from '@/store/credentials'
import type { Server } from '@/types/servers'

defineProps<{
  server: Server
  credentials: Credentials
}>()
</script>
