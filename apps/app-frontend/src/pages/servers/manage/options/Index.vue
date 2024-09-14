<template>
  <div>a</div>
</template>

<script setup lang="ts">
import { useBreadcrumbs } from '@/store/breadcrumbs'
import type { Server } from '@/types/servers'

const props = defineProps<{
  server: Server
}>()

const server = props.server
const breadcrumbs = useBreadcrumbs()

await breadcrumbs.setName('Server', server.name)
</script>
