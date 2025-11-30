<template>
	<NewModal ref="modal" header="Restore backup" fade="warning">
		<div class="flex flex-col gap-6 max-w-[600px]">
			<!-- TODO: Worlds: Replace "server" with "world" -->
			<Admonition type="warning" header="Restore warning">
				This will overwrite all files in the server and replace them with the files from the backup.
			</Admonition>

			<div v-if="currentBackup" class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">Backup</span>
				<BackupItem
					:backup="currentBackup"
					preview
					class="!bg-surface-2 border-solid border-[1px] border-surface-5 light:shadow-xl"
				/>
			</div>
		</div>

		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled>
					<button @click="modal?.hide()">
						<XIcon />
						Cancel
					</button>
				</ButtonStyled>
				<ButtonStyled color="red">
					<button @click="restoreBackup">
						<RotateCounterClockwiseIcon />
						Restore backup
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { RotateCounterClockwiseIcon, XIcon } from '@modrinth/assets'
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
