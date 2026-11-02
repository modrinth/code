<script setup lang="ts">
import { injectNotificationManager } from '@modrinth/ui'
import { computed, nextTick, ref } from 'vue'

import IconEditorModal from '@/components/ui/instance_settings/InstanceIconEditorModal.vue'
import ConfirmDeleteInstanceModal from '@/components/ui/modal/ConfirmDeleteInstanceModal.vue'
import { trackEvent } from '@/helpers/analytics'
import { toError } from '@/helpers/errors'
import { remove } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'

const { handleError } = injectNotificationManager()

const deleteModal = ref<InstanceType<typeof ConfirmDeleteInstanceModal>>()
const iconEditorModal = ref<InstanceType<typeof IconEditorModal>>()
const instanceToDelete = ref<GameInstance | null>(null)
const iconEditorInstance = ref<GameInstance | null>(null)
const instancesToDelete = computed(() => (instanceToDelete.value ? [instanceToDelete.value] : []))

function confirmDelete(instance: GameInstance) {
	instanceToDelete.value = instance
	deleteModal.value?.show()
}

async function deleteInstance() {
	if (!instanceToDelete.value) return

	await remove(instanceToDelete.value.id).catch((error) => handleError(toError(error)))
	instanceToDelete.value = null
}

async function editIcon(instance: GameInstance) {
	iconEditorInstance.value = instance
	await nextTick()
	iconEditorModal.value?.show()
	trackEvent(instance.icon_config ? 'InstanceEditCreatedIcon' : 'InstanceCreateIcon')
}

defineExpose({ confirmDelete, editIcon })
</script>

<template>
	<ConfirmDeleteInstanceModal
		ref="deleteModal"
		:instances="instancesToDelete"
		@delete="deleteInstance"
	/>
	<IconEditorModal
		ref="iconEditorModal"
		:instance-id="iconEditorInstance?.id"
		:config="iconEditorInstance?.icon_config"
		@saved="trackEvent('InstanceSaveCreatedIcon')"
	/>
</template>
