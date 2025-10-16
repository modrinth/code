<template>
	<ConfirmModal
		ref="modal"
		danger
		title="Are you sure you want to delete this backup?"
		proceed-label="Delete backup"
		:confirmation-text="currentBackup?.name ?? 'null'"
		has-to-type
		@proceed="emit('delete', currentBackup)"
	>
		<BackupItem
			v-if="currentBackup"
			:backup="currentBackup"
			preview
			class="border-px border-solid border-button-border"
		/>
	</ConfirmModal>
</template>

<script setup lang="ts">
import { ConfirmModal } from '@modrinth/ui'
import type { Backup } from '@modrinth/utils'
import { ref } from 'vue'

import BackupItem from '~/components/ui/servers/BackupItem.vue'

const emit = defineEmits<{
	(e: 'delete', backup: Backup | undefined): void
}>()

const modal = ref<InstanceType<typeof ConfirmModal>>()
const currentBackup = ref<Backup | undefined>(undefined)

function show(backup: Backup) {
	currentBackup.value = backup
	modal.value?.show()
}

defineExpose({
	show,
})
</script>
