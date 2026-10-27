<script setup lang="ts">
import IconEditorModal from '@/components/ui/instance_settings/InstanceIconEditorModal.vue'
import RemoveSharedInstancesModal from '@/components/ui/library/remove-shared-instances-modal.vue'
import { useLibrary } from '@/components/ui/library/use-library'
import ConfirmDeleteInstanceModal from '@/components/ui/modal/ConfirmDeleteInstanceModal.vue'
import SharedInstanceInstallModal from '@/components/ui/shared-instances/shared-instance-install-modal/index.vue'

const {
	confirmDeleteModal,
	iconEditorModal,
	sharedInstanceInstallModal,
	removeSharedInstancesModal,
	currentIconEditorInstance,
	currentDeleteInstances,
	deleteInstance,
	handleInstanceIconSaved,
} = useLibrary()

function setConfirmDeleteModal(component: unknown) {
	confirmDeleteModal.value = component as InstanceType<typeof ConfirmDeleteInstanceModal> | null
}

function setIconEditorModal(component: unknown) {
	iconEditorModal.value = component as InstanceType<typeof IconEditorModal> | null
}

function setRemoveSharedInstancesModal(component: unknown) {
	removeSharedInstancesModal.value = component as InstanceType<
		typeof RemoveSharedInstancesModal
	> | null
}

function setSharedInstanceInstallModal(component: unknown) {
	sharedInstanceInstallModal.value = component as InstanceType<
		typeof SharedInstanceInstallModal
	> | null
}
</script>

<template>
	<ConfirmDeleteInstanceModal
		:ref="setConfirmDeleteModal"
		:instances="currentDeleteInstances"
		@delete="deleteInstance"
	/>
	<IconEditorModal
		:ref="setIconEditorModal"
		:instance-id="currentIconEditorInstance?.id"
		:config="currentIconEditorInstance?.icon_config"
		@saved="handleInstanceIconSaved"
	/>
	<SharedInstanceInstallModal :ref="setSharedInstanceInstallModal" />
	<RemoveSharedInstancesModal :ref="setRemoveSharedInstancesModal" />
</template>
