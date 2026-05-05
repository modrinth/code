<template>
	<NewModal ref="modal" header="Restore backup" fade="danger" width="500px">
		<div class="flex flex-col gap-6">
			<Admonition v-if="ctx.isServerRunning.value" type="critical" header="Server is running">
				Stop the server before restoring a backup.
			</Admonition>
			<Admonition v-else type="critical" header="Your server files will be replaced">
				Restoring your server will replace the current world and server files. Any changes made
				since that backup will be permanently lost.
			</Admonition>

			<div v-if="currentBackup" class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">Backup</span>
				<BackupItem :backup="currentBackup" preview class="!bg-surface-2 !shadow-none" />
			</div>
		</div>

		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button @click="modal?.hide()">
						<XIcon />
						Cancel
					</button>
				</ButtonStyled>
				<ButtonStyled color="red">
					<button :disabled="isRestoring || ctx.isServerRunning.value" @click="restoreBackup">
						<SpinnerIcon v-if="isRestoring" class="animate-spin" />
						<RotateCounterClockwiseIcon v-else />
						{{ isRestoring ? 'Restoring...' : 'Restore backup' }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { RotateCounterClockwiseIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { ref } from 'vue'

import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '../../../providers'
import Admonition from '../../base/Admonition.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'
import NewModal from '../../modal/NewModal.vue'
import BackupItem from './BackupItem.vue'

const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const ctx = injectModrinthServerContext()

const backupsQueryKey = ['backups', 'queue', ctx.serverId]

function safetyBackupName(backupName: string) {
	const base = `Before restoring "${backupName}"`
	return base.slice(0, 92)
}

const restoreMutation = useMutation({
	mutationFn: ({ backupId, name }: { backupId: string; name: string }) =>
		client.archon.backups_queue_v1.restore(ctx.serverId, ctx.worldId.value!, backupId, { name }),
	onSuccess: () => queryClient.invalidateQueries({ queryKey: backupsQueryKey }),
})

const modal = ref<InstanceType<typeof NewModal>>()
const currentBackup = ref<Archon.BackupsQueue.v1.BackupQueueBackup | null>(null)
const isRestoring = ref(false)

function show(backup: Archon.BackupsQueue.v1.BackupQueueBackup) {
	currentBackup.value = backup
	modal.value?.show()
}

const restoreBackup = () => {
	if (!currentBackup.value || isRestoring.value) {
		if (!currentBackup.value) {
			addNotification({
				type: 'error',
				title: 'Failed to restore backup',
				text: 'Current backup is null',
			})
		}
		return
	}

	isRestoring.value = true
	restoreMutation.mutate(
		{
			backupId: currentBackup.value.id,
			name: safetyBackupName(currentBackup.value.name),
		},
		{
			onSuccess: () => {
				modal.value?.hide()
			},
			onError: (error) => {
				const message = error instanceof Error ? error.message : String(error)
				addNotification({ type: 'error', title: 'Failed to restore backup', text: message })
			},
			onSettled: () => {
				isRestoring.value = false
			},
		},
	)
}

defineExpose({
	show,
})
</script>
