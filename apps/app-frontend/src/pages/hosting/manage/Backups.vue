<script setup lang="ts">
import { injectModrinthServerContext, ServersManageBackupsPage } from '@modrinth/ui'

const { isServerRunning } = injectModrinthServerContext()
</script>

<template>
	<ServersManageBackupsPage :is-server-running="isServerRunning" />
</template>
