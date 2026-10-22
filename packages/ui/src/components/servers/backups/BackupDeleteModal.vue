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

			<BackupPreviewList
				v-if="displayBackups.length"
				:backups="displayBackups"
				:label="formatMessage(messages.backupsLabel, { count })"
			/>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="outlined" @click="modal?.hide()">
					<XIcon />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button
					v-tooltip="deleteDisabledTooltip"
					type="colored"
					color="red"
					:disabled="!props.canDelete"
					@click="confirmDelete"
				>
					<TrashIcon />
					{{ formatMessage(messages.confirm, { count }) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { TrashIcon, XIcon } from '@modrinth/assets'
import { computed, ref } from 'vue'

import { Button } from '#ui/components/base/buttons'

import { defineMessages, useVIntl } from '../../../composables/i18n'
import { commonMessages } from '../../../utils'
import Admonition from '../../base/Admonition.vue'
import NewModal from '../../modal/NewModal.vue'
import BackupPreviewList from './BackupPreviewList.vue'

const { formatMessage } = useVIntl()

const props = withDefaults(
	defineProps<{
		canDelete?: boolean
		permissionDeniedMessage?: string
	}>(),
	{
		canDelete: true,
		permissionDeniedMessage: undefined,
	},
)

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
const deleteDisabledTooltip = computed(() =>
	props.canDelete
		? undefined
		: (props.permissionDeniedMessage ?? formatMessage(commonMessages.noPermissionAction)),
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
	if (!props.canDelete) return
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
