<script setup lang="ts">
import { ServerSettingsInstallationPage } from '@modrinth/ui'

const emit = defineEmits<{
	reinstall: [unknown?]
	'reinstall-failed': []
}>()
</script>

<template>
	<ServerSettingsInstallationPage
		@reinstall="(event: unknown) => emit('reinstall', event)"
		@reinstall-failed="emit('reinstall-failed')"
	/>
</template>
