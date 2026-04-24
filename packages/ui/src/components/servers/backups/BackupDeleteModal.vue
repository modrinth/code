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

			<div v-if="displayBackups.length" class="flex min-w-0 flex-col gap-2">
				<span class="font-semibold text-contrast">
					{{ formatMessage(messages.backupsLabel, { count }) }}
				</span>
				<div class="relative">
					<Transition
						enter-active-class="transition-all duration-200 ease-out"
						enter-from-class="opacity-0 max-h-0"
						enter-to-class="opacity-100 max-h-2"
						leave-active-class="transition-all duration-200 ease-in"
						leave-from-class="opacity-100 max-h-2"
						leave-to-class="opacity-0 max-h-0"
					>
						<div
							v-if="showTopFade"
							class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-2 bg-gradient-to-b from-bg-raised to-transparent"
						/>
					</Transition>
					<div
						ref="backupListRef"
						class="flex max-h-[240px] flex-col gap-2 overflow-y-auto"
						@scroll="checkScrollState"
					>
						<BackupItem
							v-for="backup in displayBackups"
							:key="backup.id"
							:backup="backup"
							preview
							class="!bg-surface-2 !shadow-none"
						/>
					</div>
					<Transition
						enter-active-class="transition-all duration-200 ease-out"
						enter-from-class="opacity-0 max-h-0"
						enter-to-class="opacity-100 max-h-2"
						leave-active-class="transition-all duration-200 ease-in"
						leave-from-class="opacity-100 max-h-2"
						leave-to-class="opacity-0 max-h-0"
					>
						<div
							v-if="showBottomFade"
							class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-2 bg-gradient-to-t from-bg-raised to-transparent"
						/>
					</Transition>
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
import { computed, nextTick, ref } from 'vue'

import { defineMessages, useVIntl } from '../../../composables/i18n'
import { useScrollIndicator } from '../../../composables/scroll-indicator'
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
const backupListRef = ref<HTMLElement | null>(null)
const singleBackup = ref<Archon.BackupsQueue.v1.BackupQueueBackup>()
const bulkBackups = ref<Archon.BackupsQueue.v1.BackupQueueBackup[]>([])
const { showTopFade, showBottomFade, checkScrollState, forceCheck } =
	useScrollIndicator(backupListRef)

const isBulk = computed(() => bulkBackups.value.length > 0)
const count = computed(() => (isBulk.value ? bulkBackups.value.length : 1))
const displayBackups = computed(() =>
	isBulk.value ? bulkBackups.value : singleBackup.value ? [singleBackup.value] : [],
)

function show(backup: Archon.BackupsQueue.v1.BackupQueueBackup) {
	singleBackup.value = backup
	bulkBackups.value = []
	modal.value?.show()
	nextTick(() => forceCheck())
}

function showBulk(backups: Archon.BackupsQueue.v1.BackupQueueBackup[]) {
	singleBackup.value = undefined
	bulkBackups.value = [...backups]
	modal.value?.show()
	nextTick(() => forceCheck())
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
