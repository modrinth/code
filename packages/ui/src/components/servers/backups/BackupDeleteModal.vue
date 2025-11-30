<template>
	<NewModal ref="modal" header="Delete backup" fade="danger">
		<div class="flex flex-col gap-6">
			<Admonition type="critical" header="Delete warning">
				This backup will be permanently deleted. This action cannot be undone.
			</Admonition>

			<div v-if="currentBackup" class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">Backup</span>
				<BackupItem
					:backup="currentBackup"
					preview
					class="!bg-surface-2 border-solid border-[1px] border-surface-5"
				/>
			</div>
		</div>

		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled>
					<button @click="modal?.hide()">
						<XIcon />
						Cancel
					</button>
				</ButtonStyled>
				<ButtonStyled color="red">
					<button @click="deleteBackup">
						<TrashIcon />
						Delete backup
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

import Admonition from '../../base/Admonition.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'
import NewModal from '../../modal/NewModal.vue'
import BackupItem from './BackupItem.vue'

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
