<template>
	<NewModal ref="modal" header="Creating backup" @show="focusInput">
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
				:placeholder="`Backup #${newBackupAmount}`"
				maxlength="48"
			/>
			<div v-if="nameExists && !createMutation.isPending.value" class="flex items-center gap-1">
				<IssuesIcon class="hidden text-orange sm:block" />
				<span class="text-sm text-orange">
					You already have a backup named '<span class="font-semibold">{{ trimmedName }}</span
					>'
				</span>
			</div>
			<div v-if="isRateLimited" class="mt-2 text-sm text-red">
				You're creating backups too fast. Please wait a moment before trying again.
			</div>
		</div>
		<div class="mt-2 flex justify-start gap-2">
			<ButtonStyled color="brand">
				<button :disabled="createMutation.isPending.value || nameExists" @click="createBackup">
					<PlusIcon />
					Create backup
				</button>
			</ButtonStyled>
			<ButtonStyled>
				<button @click="hideModal">
					<XIcon />
					Cancel
				</button>
			</ButtonStyled>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import { IssuesIcon, PlusIcon, XIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	injectModrinthClient,
	injectNotificationManager,
	NewModal,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, nextTick, ref } from 'vue'

import { injectModrinthServerContext } from '~/providers/server-context.ts'

const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const ctx = injectModrinthServerContext()

const backupsQueryKey = ['backups', 'list', ctx.serverId]

const { data: backups } = useQuery({
	queryKey: backupsQueryKey,
	queryFn: () => client.archon.backups_v1.list(ctx.serverId),
})

const createMutation = useMutation({
	mutationFn: (name: string) => client.archon.backups_v1.create(ctx.serverId, { name }),
	onSuccess: () => queryClient.invalidateQueries({ queryKey: backupsQueryKey }),
})

const modal = ref<InstanceType<typeof NewModal>>()
const input = ref<HTMLInputElement>()
const isRateLimited = ref(false)
const backupName = ref('')
const newBackupAmount = computed(() => (backups.value?.length ?? 0) + 1)

const trimmedName = computed(() => backupName.value.trim())

const nameExists = computed(() => {
	if (!backups.value) return false
	return backups.value.some(
		(backup) => backup.name.trim().toLowerCase() === trimmedName.value.toLowerCase(),
	)
})

const focusInput = () => {
	nextTick(() => {
		setTimeout(() => {
			input.value?.focus()
		}, 100)
	})
}

function show() {
	backupName.value = ''
	isRateLimited.value = false
	modal.value?.show()
}

const hideModal = () => {
	modal.value?.hide()
}

const createBackup = () => {
	const name = trimmedName.value || `Backup #${newBackupAmount.value}`
	isRateLimited.value = false

	createMutation.mutate(name, {
		onSuccess: () => {
			hideModal()
		},
		onError: (error) => {
			if (error instanceof Error && error.message.includes('429')) {
				isRateLimited.value = true
				addNotification({
					type: 'error',
					title: 'Error creating backup',
					text: "You're creating backups too fast.",
				})
			} else {
				const message = error instanceof Error ? error.message : String(error)
				addNotification({ type: 'error', title: 'Error creating backup', text: message })
			}
		},
	})
}

defineExpose({
	show,
	hide: hideModal,
})
</script>
