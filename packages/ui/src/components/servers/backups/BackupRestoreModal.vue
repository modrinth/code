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
import type { Archon } from '@modrinth/api-client'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { ref } from 'vue'

import type { NewModal } from '../../..'
import { ConfirmModal, injectModrinthClient, injectNotificationManager } from '../../..'
import { injectModrinthServerContext } from '../../../providers'
import BackupItem from './BackupItem.vue'

const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const ctx = injectModrinthServerContext()

const backupsQueryKey = ['backups', 'list', ctx.serverId]
const restoreMutation = useMutation({
	mutationFn: (backupId: string) => client.archon.backups_v0.restore(ctx.serverId, backupId),
	onSuccess: () => queryClient.invalidateQueries({ queryKey: backupsQueryKey }),
})

const modal = ref<InstanceType<typeof NewModal>>()
const currentBackup = ref<Archon.Backups.v1.Backup | null>(null)

function show(backup: Archon.Backups.v1.Backup) {
	currentBackup.value = backup
	modal.value?.show()
}

const restoreBackup = () => {
	if (!currentBackup.value) {
		addNotification({
			type: 'error',
			title: 'Failed to restore backup',
			text: 'Current backup is null',
		})
		return
	}

	restoreMutation.mutate(currentBackup.value.id, {
		onSuccess: () => {
			modal.value?.hide()
		},
		onError: (error) => {
			const message = error instanceof Error ? error.message : String(error)
			addNotification({ type: 'error', title: 'Failed to restore backup', text: message })
		},
	})
}

defineExpose({
	show,
})
</script>
