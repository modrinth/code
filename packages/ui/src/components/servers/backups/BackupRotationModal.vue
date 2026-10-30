<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { TrashIcon, XIcon } from '@modrinth/assets'
import { ref } from 'vue'

import { Button } from '#ui/components/base/buttons'

import { defineMessages, useVIntl } from '../../../composables/i18n'
import { commonMessages } from '../../../utils'
import Admonition from '../../base/Admonition.vue'
import NewModal from '../../modal/NewModal.vue'
import BackupPreviewList from './BackupPreviewList.vue'

const emit = defineEmits<{
	proceed: []
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'servers.backups.rotation-modal.header',
		defaultMessage: 'Remove auto backups?',
	},
	admonitionHeader: {
		id: 'servers.backups.rotation-modal.admonition-header',
		defaultMessage: 'Deletion warning',
	},
	admonitionBody: {
		id: 'servers.backups.rotation-modal.admonition-body',
		defaultMessage:
			'Fewer auto backup slots means {count, plural, one {your oldest auto backup} other {your # oldest auto backups}} will be permanently deleted. This cannot be undone.',
	},
	backupsLabel: {
		id: 'servers.backups.rotation-modal.backups-label',
		defaultMessage: '{count, plural, one {Backup} other {Backups ({count})}}',
	},
	confirm: {
		id: 'servers.backups.rotation-modal.confirm',
		defaultMessage: 'Save and delete',
	},
})

const modal = ref<InstanceType<typeof NewModal>>()
const backups = ref<Archon.BackupsQueue.v1.BackupQueueBackup[]>([])
const count = ref(0)

function show(rotatedBackups: Archon.BackupsQueue.v1.BackupQueueBackup[], rotatedCount: number) {
	backups.value = rotatedBackups
	count.value = rotatedCount
	modal.value?.show()
}

function confirm() {
	modal.value?.hide()
	emit('proceed')
}

defineExpose({ show })
</script>

<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" fade="danger" width="500px">
		<div class="flex flex-col gap-6">
			<Admonition type="critical" :header="formatMessage(messages.admonitionHeader)">
				{{ formatMessage(messages.admonitionBody, { count }) }}
			</Admonition>

			<BackupPreviewList
				v-if="backups.length"
				:backups="backups"
				:label="formatMessage(messages.backupsLabel, { count: backups.length })"
			/>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="outlined" @click="modal?.hide()">
					<XIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button type="colored" color="red" @click="confirm">
					<TrashIcon aria-hidden="true" />
					{{ formatMessage(messages.confirm) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>
