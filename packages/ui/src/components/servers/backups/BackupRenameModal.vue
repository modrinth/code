<template>
	<NewModal ref="modal" header="Renaming backup" @show="focusInput">
		<div class="flex flex-col gap-2 md:w-[600px]">
			<label for="backup-name-input">
				<span class="text-lg font-semibold text-contrast"> Name </span>
			</label>
			<input
				id="backup-name-input"
				ref="input"
				v-model="backupName"
				type="text"
				class="bg-bg-input w-full rounded-lg p-4"
				:placeholder="`Backup #${backupNumber}`"
				maxlength="48"
			/>
			<div v-if="nameExists" class="flex items-center gap-1">
				<IssuesIcon class="hidden text-orange sm:block" />
				<span class="text-sm text-orange">
					You already have a backup named '<span class="font-semibold">{{ trimmedName }}</span
					>'
				</span>
			</div>
		</div>
		<div class="mt-2 flex justify-start gap-2">
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
			<ButtonStyled>
				<button @click="hide">
					<XIcon />
					Cancel
				</button>
			</ButtonStyled>
		</div>
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
import NewModal from '../../modal/NewModal.vue'

const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const ctx = injectModrinthServerContext()

const props = defineProps<{
	backups?: Archon.Backups.v1.Backup[]
}>()

const backupsQueryKey = ['backups', 'list', ctx.serverId]

const renameMutation = useMutation({
	mutationFn: ({ backupId, name }: { backupId: string; name: string }) =>
		client.archon.backups_v0.rename(ctx.serverId, backupId, { name }),
	onSuccess: () => queryClient.invalidateQueries({ queryKey: backupsQueryKey }),
})

const modal = ref<InstanceType<typeof NewModal>>()
const input = ref<HTMLInputElement>()
const backupName = ref('')
const originalName = ref('')

const currentBackup = ref<Archon.Backups.v1.Backup | null>(null)

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

function show(backup: Archon.Backups.v1.Backup) {
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
