<template>
	<NewModal ref="modal" :header="formatMessage(messages.modalTitle)" fade="danger">
		<div class="flex flex-col gap-6 max-w-[400px]">
			<Admonition type="critical" :header="formatMessage(messages.warningHeader)">
				{{ formatMessage(messages.warningBody) }}
			</Admonition>

			<div v-if="currentBackup" class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.backupLabel) }}</span>
				<BackupItem :backup="currentBackup" preview class="!bg-surface-2 !shadow-none" />
			</div>
		</div>

		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button class="!border !border-surface-4" @click="modal?.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="red">
					<button @click="deleteBackup">
						<TrashIcon />
						{{ formatMessage(messages.deleteButton) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { TrashIcon, XIcon } from '@modrinth/assets'
import { ref } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import Admonition from '../../base/Admonition.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'
import NewModal from '../../modal/NewModal.vue'
import BackupItem from './BackupItem.vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	modalTitle: {
		id: 'servers.backups.delete-modal.title',
		defaultMessage: 'Delete backup',
	},
	warningHeader: {
		id: 'servers.backups.delete-modal.warning.header',
		defaultMessage: 'Delete warning',
	},
	warningBody: {
		id: 'servers.backups.delete-modal.warning.body',
		defaultMessage: 'This backup will be permanently deleted. This action cannot be undone.',
	},
	backupLabel: {
		id: 'servers.backups.delete-modal.backup-label',
		defaultMessage: 'Backup',
	},
	deleteButton: {
		id: 'servers.backups.delete-modal.delete-button',
		defaultMessage: 'Delete backup',
	},
})

const emit = defineEmits<{
	(e: 'delete', backup: Archon.Backups.v1.Backup | undefined): void
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const currentBackup = ref<Archon.Backups.v1.Backup>()

function show(backup: Archon.Backups.v1.Backup) {
	currentBackup.value = backup
	modal.value?.show()
}

function deleteBackup() {
	modal.value?.hide()
	emit('delete', currentBackup.value)
}

defineExpose({
	show,
})
</script>
