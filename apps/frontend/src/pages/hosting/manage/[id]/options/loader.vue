<template>
	<ServerInstallation :backup-in-progress="props.backupInProgress" @reinstall="emit('reinstall')" />
</template>

<script setup lang="ts">
import ServerInstallation from '~/components/ui/servers/ServerInstallation.vue'
import type { BackupInProgressReason } from '~/pages/hosting/manage/[id].vue'

const props = defineProps<{
	backupInProgress?: BackupInProgressReason
}>()

const emit = defineEmits<{
	reinstall: [any?]
}>()
</script>
