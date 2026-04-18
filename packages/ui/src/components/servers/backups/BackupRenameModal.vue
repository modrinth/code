<template>
	<NewModal ref="modal" :header="formatMessage(messages.modalTitle)" @show="focusInput">
		<div class="flex flex-col gap-2 md:w-[600px]">
			<label for="backup-name-input">
				<span class="text-lg font-semibold text-contrast">{{
					formatMessage(messages.nameLabel)
				}}</span>
			</label>
			<StyledInput
				id="backup-name-input"
				ref="input"
				v-model="backupName"
				:placeholder="formatMessage(messages.namePlaceholder, { number: backupNumber })"
				:maxlength="48"
				wrapper-class="w-full"
			/>
			<div v-if="nameExists" class="flex items-center gap-1">
				<IssuesIcon class="hidden text-orange sm:block" />
				<span class="text-sm text-orange">
					<IntlFormatted :message-id="messages.duplicateName" :values="{ name: trimmedName }">
						<template #name-highlight="{ children }">
							<span class="font-semibold"><component :is="() => children" /></span>
						</template>
					</IntlFormatted>
				</span>
			</div>
		</div>
		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button class="!border !border-surface-4" @click="hide">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button :disabled="renameMutation.isPending.value || nameExists" @click="renameBackup">
						<template v-if="renameMutation.isPending.value">
							<SpinnerIcon class="animate-spin" />
							{{ formatMessage(messages.renaming) }}
						</template>
						<template v-else>
							<SaveIcon />
							{{ formatMessage(commonMessages.saveChangesButton) }}
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

import IntlFormatted from '#ui/components/base/IntlFormatted.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'
import { commonMessages } from '#ui/utils/common-messages'

import ButtonStyled from '../../base/ButtonStyled.vue'
import StyledInput from '../../base/StyledInput.vue'
import NewModal from '../../modal/NewModal.vue'

const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const ctx = injectModrinthServerContext()

const messages = defineMessages({
	modalTitle: {
		id: 'servers.backups.rename-modal.title',
		defaultMessage: 'Renaming backup',
	},
	nameLabel: {
		id: 'servers.backups.rename-modal.name-label',
		defaultMessage: 'Name',
	},
	namePlaceholder: {
		id: 'servers.backups.rename-modal.name-placeholder',
		defaultMessage: 'Backup #{number}',
	},
	duplicateName: {
		id: 'servers.backups.rename-modal.duplicate-name',
		defaultMessage: "You already have a backup named '<name-highlight>{name}</name-highlight>'.",
	},
	renaming: {
		id: 'servers.backups.rename-modal.renaming',
		defaultMessage: 'Renaming...',
	},
	errorTitle: {
		id: 'servers.backups.rename-modal.notification.error.title',
		defaultMessage: 'Error renaming backup',
	},
})

const props = defineProps<{
	backups?: Archon.Backups.v1.Backup[]
}>()

const backupsQueryKey = ['backups', 'list', ctx.serverId]

const renameMutation = useMutation({
	mutationFn: ({ backupId, name }: { backupId: string; name: string }) =>
		client.archon.backups_v1.rename(ctx.serverId, ctx.worldId.value!, backupId, { name }),
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
			title: formatMessage(messages.errorTitle),
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
		newName = formatMessage(messages.namePlaceholder, { number: backupNumber.value })
	}

	renameMutation.mutate(
		{ backupId: currentBackup.value.id, name: newName },
		{
			onSuccess: () => {
				hide()
			},
			onError: (error) => {
				const message = error instanceof Error ? error.message : String(error)
				addNotification({
					type: 'error',
					title: formatMessage(messages.errorTitle),
					text: message,
				})
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
