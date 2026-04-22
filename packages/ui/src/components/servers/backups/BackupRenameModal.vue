<template>
	<NewModal ref="modal" header="Renaming backup" max-width="500px" @show="focusInput">
		<div class="flex flex-col gap-2">
			<label for="backup-name-input">
				<span class="text-lg font-semibold text-contrast"> Name </span>
			</label>
			<StyledInput
				id="backup-name-input"
				ref="input"
				v-model="backupName"
				:placeholder="`Backup #${backupNumber}`"
				:maxlength="48"
				wrapper-class="w-full"
			/>
			<div v-if="nameExists" class="flex items-center gap-1">
				<IssuesIcon class="hidden text-orange sm:block" />
				<span class="text-sm text-orange">
					You already have a backup named '<span class="font-semibold">{{ trimmedName }}</span
					>'
				</span>
			</div>
		</div>
		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button class="!border !border-surface-4" @click="hide">
						<XIcon />
						Cancel
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button :disabled="renameMutation.isPending.value || nameExists" @click="renameBackup">
						<template v-if="renameMutation.isPending.value">
							<SpinnerIcon class="animate-spin" />
							Renaming...
						</template>
						<template v-else>
							<SaveIcon />
							Save changes
						</template>
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { IssuesIcon, SaveIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { computed, nextTick, ref } from 'vue'

import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '../../../providers'
import ButtonStyled from '../../base/ButtonStyled.vue'
import StyledInput from '../../base/StyledInput.vue'
import NewModal from '../../modal/NewModal.vue'

const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const ctx = injectModrinthServerContext()

const props = defineProps<{
	backups?: Archon.BackupsQueue.v1.BackupQueueBackup[]
}>()

const backupsQueryKey = ['backups', 'queue', ctx.serverId]

const renameMutation = useMutation({
	mutationFn: ({ backupId, name }: { backupId: string; name: string }) =>
		client.archon.backups_v1.rename(ctx.serverId, ctx.worldId.value!, backupId, { name }),
	onSuccess: () => queryClient.invalidateQueries({ queryKey: backupsQueryKey }),
})

const modal = ref<InstanceType<typeof NewModal>>()
const input = ref<HTMLInputElement>()
const backupName = ref('')
const originalName = ref('')

const currentBackup = ref<Archon.BackupsQueue.v1.BackupQueueBackup | null>(null)

const trimmedName = computed(() => backupName.value.trim())

const nameExists = computed(() => {
	if (
		!props.backups ||
		trimmedName.value === originalName.value ||
		renameMutation.isPending.value
	) {
		return false
	}

	return props.backups.some(
		(backup) => backup.name.trim().toLowerCase() === trimmedName.value.toLowerCase(),
	)
})

const backupNumber = computed(
	() => (props.backups?.findIndex((b) => b.id === currentBackup.value?.id) ?? 0) + 1,
)

const focusInput = () => {
	nextTick(() => {
		setTimeout(() => {
			input.value?.focus()
		}, 100)
	})
}

function show(backup: Archon.BackupsQueue.v1.BackupQueueBackup) {
	currentBackup.value = backup
	backupName.value = backup.name
	originalName.value = backup.name
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

const renameBackup = () => {
	if (!currentBackup.value) {
		addNotification({
			type: 'error',
			title: 'Error renaming backup',
			text: 'Current backup is null',
		})
		return
	}

	if (trimmedName.value === originalName.value) {
		hide()
		return
	}

	let newName = trimmedName.value
	if (newName.length === 0) {
		newName = `Backup #${backupNumber.value}`
	}

	renameMutation.mutate(
		{ backupId: currentBackup.value.id, name: newName },
		{
			onSuccess: () => {
				hide()
			},
			onError: (error) => {
				const message = error instanceof Error ? error.message : String(error)
				addNotification({ type: 'error', title: 'Error renaming backup', text: message })
				hide()
			},
		},
	)
}

defineExpose({
	show,
	hide,
})
</script>
