<template>
	<MultiStageModal
		ref="modal"
		:stages="ctx.stageConfigs"
		:context="ctx"
		:fade="fade"
		disable-progress
		@hide="$emit('hide')"
	/>
</template>

<script setup lang="ts">
import { useTemplateRef } from 'vue'
import type { ComponentExposed } from 'vue-component-type-helpers'

import MultiStageModal from '../../base/MultiStageModal.vue'
import {
	createCreationFlowContext,
	type CreationFlowContextValue,
	type FlowType,
	type ModpackSearchResult,
	provideCreationFlowContext,
} from './creation-flow-context'

const props = withDefaults(
	defineProps<{
		type?: FlowType
		availableLoaders?: string[]
		showSnapshotToggle?: boolean
		disableClose?: boolean
		isInitialSetup?: boolean
		initialLoader?: string
		initialGameVersion?: string
		onBack?: (() => void) | null
		fade?: 'standard' | 'warning' | 'danger'
		searchModpacks?: (query: string, limit?: number) => Promise<ModpackSearchResult>
		getProjectVersions?: (projectId: string) => Promise<{ id: string }[]>
	}>(),
	{
		type: 'world',
		availableLoaders: () => ['fabric', 'neoforge', 'forge', 'quilt'],
		showSnapshotToggle: false,
		disableClose: false,
		isInitialSetup: false,
		initialLoader: undefined,
		initialGameVersion: undefined,
		onBack: null,
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
		initialLoader: props.initialLoader,
		initialGameVersion: props.initialGameVersion,
		onBack: props.onBack ?? undefined,
		searchModpacks: props.searchModpacks,
		getProjectVersions: props.getProjectVersions,
	},
)
provideCreationFlowContext(ctx)

function show(instanceCount?: number) {
	ctx.reset(instanceCount)
	modal.value?.setStage(0)
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

defineExpose({ show, hide, ctx })
</script>
