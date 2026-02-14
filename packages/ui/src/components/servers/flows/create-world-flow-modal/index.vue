<template>
	<MultiStageModal
		ref="modal"
		:stages="ctx.stageConfigs"
		:context="ctx"
		@hide="$emit('hide')"
	/>
</template>

<script setup lang="ts">
import { useTemplateRef } from 'vue'
import type { ComponentExposed } from 'vue-component-type-helpers'
import MultiStageModal from '../../../base/MultiStageModal.vue'
import {
	createWorldContext,
	provideCreateWorldContext,
} from './create-world-context'

const emit = defineEmits<{
	(e: 'hide'): void
	(e: 'browse-modpacks'): void
	(e: 'create-world', config: any): void
}>()

const modal = useTemplateRef<ComponentExposed<typeof MultiStageModal>>('modal')

const ctx = createWorldContext(modal, {
	browseModpacks: () => emit('browse-modpacks'),
	createWorld: (config) => emit('create-world', config),
})
provideCreateWorldContext(ctx)

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
