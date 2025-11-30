<template>
	<NewModal ref="modal" header="Create backup" @show="focusInput">
		<div class="flex flex-col gap-2 md:w-[600px] -mb-2">
			<label for="backup-name-input">
				<span class="text-lg font-semibold text-contrast">Name</span>
			</label>
			<input
				id="backup-name-input"
				ref="input"
				v-model="backupName"
				type="text"
				class="w-full rounded-lg bg-bg-input p-4"
				:placeholder="`Backup #${newBackupAmount}`"
				maxlength="48"
			/>
			<Transition
				enter-active-class="transition-all duration-300 ease-out"
				enter-from-class="opacity-0 max-h-0"
				enter-to-class="opacity-100 max-h-20"
				leave-active-class="transition-all duration-200 ease-in"
				leave-from-class="opacity-100 max-h-20"
				leave-to-class="opacity-0 max-h-0"
			>
				<div
					v-if="nameExists && !createMutation.isPending.value"
					class="flex items-center gap-1 mt-2 overflow-hidden"
				>
					<IssuesIcon class="hidden text-orange sm:block" />
					<span class="text-sm text-orange">
						You already have a backup named '<span class="font-semibold">{{ trimmedName }}</span
						>'
					</span>
				</div>
			</Transition>
			<Transition
				enter-active-class="transition-all duration-300 ease-out"
				enter-from-class="opacity-0 max-h-0"
				enter-to-class="opacity-100 max-h-20"
				leave-active-class="transition-all duration-200 ease-in"
				leave-from-class="opacity-100 max-h-20"
				leave-to-class="opacity-0 max-h-0"
			>
				<div v-if="isRateLimited" class="overflow-hidden text-sm text-red">
					You're creating backups too fast. Please wait a moment before trying again.
				</div>
			</Transition>
		</div>
		<template #actions>
			<div class="w-full flex flex-row gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button class="!border-[1px] !border-surface-4" @click="hideModal">
						<XIcon />
						Cancel
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button :disabled="createMutation.isPending.value || nameExists" @click="createBackup">
						<PlusIcon />
						Create backup
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { IssuesIcon, PlusIcon, XIcon } from '@modrinth/assets'
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

const createMutation = useMutation({
	mutationFn: (name: string) => client.archon.backups_v0.create(ctx.serverId, { name }),
	onSuccess: () => queryClient.invalidateQueries({ queryKey: backupsQueryKey }),
})

const modal = ref<InstanceType<typeof NewModal>>()
const input = ref<HTMLInputElement>()
const isRateLimited = ref(false)
const backupName = ref('')
const newBackupAmount = computed(() => (props.backups?.length ?? 0) + 1)

const trimmedName = computed(() => backupName.value.trim())

const nameExists = computed(() => {
	if (!props.backups) return false
	return props.backups.some(
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
