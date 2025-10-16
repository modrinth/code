<template>
	<ConfirmModal
		ref="modal"
		danger
		title="Are you sure you want to restore from this backup?"
		proceed-label="Restore from backup"
		description="This will **overwrite all files on your server** and replace them with the files from the backup."
		@proceed="restoreBackup"
	>
		<BackupItem
			v-if="currentBackup"
			:backup="currentBackup"
			preview
			class="border-px border-solid border-button-border"
		/>
	</ConfirmModal>
</template>

<script setup lang="ts">
import type { NewModal } from '@modrinth/ui'
import { ConfirmModal, injectNotificationManager } from '@modrinth/ui'
import type { Backup } from '@modrinth/utils'
import { ref } from 'vue'

import BackupItem from '~/components/ui/servers/BackupItem.vue'
import type { ModrinthServer } from '~/composables/servers/modrinth-servers.ts'

const { addNotification } = injectNotificationManager()

const props = defineProps<{
	server: ModrinthServer
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const currentBackup = ref<Backup | null>(null)

function show(backup: Backup) {
	currentBackup.value = backup
	modal.value?.show()
}

const restoreBackup = async () => {
	if (!currentBackup.value) {
		addNotification({
			type: 'error',
			title: 'Failed to restore backup',
			text: 'Current backup is null',
		})
		return
	}

	try {
		await props.server.backups?.restore(currentBackup.value.id)
	} catch (error) {
		const message = error instanceof Error ? error.message : String(error)
		addNotification({ type: 'error', title: 'Failed to restore backup', text: message })
	}
}

defineExpose({
	show,
})
</script>
