<template>
	<MultiStageModal ref="modal" :stages="ctx.stageConfigs" :context="ctx" @hide="$emit('hide')" />
</template>

<script setup lang="ts">
import { useTemplateRef } from 'vue'
import type { ComponentExposed } from 'vue-component-type-helpers'

import MultiStageModal from '../../base/MultiStageModal.vue'
import {
	createCreationFlowContext,
	type CreationFlowContextValue,
	type FlowType,
	provideCreationFlowContext,
} from './creation-flow-context'

const props = withDefaults(
	defineProps<{
		type?: FlowType
		availableLoaders?: string[]
		showSnapshotToggle?: boolean
		disableClose?: boolean
		isInitialSetup?: boolean
	}>(),
	{
		type: 'world',
		availableLoaders: () => ['fabric', 'neoforge', 'forge', 'quilt'],
		showSnapshotToggle: false,
		disableClose: false,
		isInitialSetup: false,
	},
)

const emit = defineEmits<{
	(e: 'hide' | 'browse-modpacks'): void
	(e: 'create', config: CreationFlowContextValue): void
}>()

const modal = useTemplateRef<ComponentExposed<typeof MultiStageModal>>('modal')

const ctx = createCreationFlowContext(
	modal,
	props.type,
	{
		browseModpacks: () => emit('browse-modpacks'),
		create: (config) => emit('create', config),
	},
	{
		availableLoaders: props.availableLoaders,
		showSnapshotToggle: props.showSnapshotToggle,
		disableClose: props.disableClose,
		isInitialSetup: props.isInitialSetup,
	},
)
provideCreationFlowContext(ctx)

function show() {
	ctx.reset()
	modal.value?.setStage(0)
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

defineExpose({ show, hide, ctx })
</script>
