<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header, { count })"
		fade="danger"
		width="500px"
	>
		<div class="flex flex-col gap-6">
			<Admonition type="critical" :header="formatMessage(messages.admonitionHeader)">
				{{ formatMessage(messages.admonitionBody, { count }) }}
			</Admonition>

			<div v-if="displayBackups.length" class="flex flex-col gap-2 min-w-0">
				<span class="font-semibold text-contrast">
					{{ formatMessage(messages.backupsLabel, { count }) }}
				</span>
				<div class="flex flex-col gap-2 max-h-[240px] overflow-y-auto">
					<BackupItem
						v-for="backup in displayBackups"
						:key="backup.id"
						:backup="backup"
						preview
						class="!bg-surface-2 !shadow-none"
					/>
				</div>
			</div>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled type="outlined">
					<button class="!border !border-surface-4" @click="modal?.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="red">
					<button @click="confirmDelete">
						<TrashIcon />
						{{ formatMessage(messages.confirm, { count }) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { TrashIcon, XIcon } from '@modrinth/assets'
import { computed, ref } from 'vue'

import { defineMessages, useVIntl } from '../../../composables/i18n'
import { commonMessages } from '../../../utils'
import Admonition from '../../base/Admonition.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'
import NewModal from '../../modal/NewModal.vue'
import BackupItem from './BackupItem.vue'

const { formatMessage } = useVIntl()

const emit = defineEmits<{
	(e: 'delete', backup: Archon.BackupsQueue.v1.BackupQueueBackup | undefined): void
	(e: 'bulk-delete', backups: Archon.BackupsQueue.v1.BackupQueueBackup[]): void
}>()

const messages = defineMessages({
	header: {
		id: 'servers.backups.delete-modal.header',
		defaultMessage: 'Delete {count, plural, one {backup} other {backups}}',
	},
	admonitionHeader: {
		id: 'servers.backups.delete-modal.admonition-header',
		defaultMessage: 'Deletion warning',
	},
	admonitionBody: {
		id: 'servers.backups.delete-modal.admonition-body',
		defaultMessage:
			'Once deleted, {count, plural, one {this backup cannot} other {these backups cannot}} be recovered. Deletion is permanent.',
	},
	confirm: {
		id: 'servers.backups.delete-modal.confirm',
		defaultMessage: 'Delete {count, plural, one {backup} other {# backups}}',
	},
	backupsLabel: {
		id: 'servers.backups.delete-modal.backups-label',
		defaultMessage: '{count, plural, one {Backup} other {Backups ({count})}}',
	},
})

const modal = ref<InstanceType<typeof NewModal>>()
const singleBackup = ref<Archon.BackupsQueue.v1.BackupQueueBackup>()
const bulkBackups = ref<Archon.BackupsQueue.v1.BackupQueueBackup[]>([])

const isBulk = computed(() => bulkBackups.value.length > 0)
const count = computed(() => (isBulk.value ? bulkBackups.value.length : 1))
const displayBackups = computed(() =>
	isBulk.value ? bulkBackups.value : singleBackup.value ? [singleBackup.value] : [],
)

function show(backup: Archon.BackupsQueue.v1.BackupQueueBackup) {
	singleBackup.value = backup
	bulkBackups.value = []
	modal.value?.show()
}

function showBulk(backups: Archon.BackupsQueue.v1.BackupQueueBackup[]) {
	singleBackup.value = undefined
	bulkBackups.value = [...backups]
	modal.value?.show()
}

function confirmDelete() {
	modal.value?.hide()
	if (isBulk.value) {
		emit('bulk-delete', bulkBackups.value)
		bulkBackups.value = []
	} else {
		emit('delete', singleBackup.value)
	}
}

defineExpose({
	show,
	showBulk,
})
</script>
