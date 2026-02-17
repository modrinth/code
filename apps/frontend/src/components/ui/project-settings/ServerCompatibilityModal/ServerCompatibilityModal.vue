<template>
	<MultiStageModal ref="modal" :stages="ctx.stageConfigs" :context="ctx" @hide="handleHide" />
</template>

<script setup lang="ts">
import { MultiStageModal } from '@modrinth/ui'
import type { ComponentExposed } from 'vue-component-type-helpers'

import {
	createServerCompatibilityContext,
	provideServerCompatibilityContext,
} from './server-compatibility-modal'

const emit = defineEmits<{
	(e: 'save'): void
}>()

const modal = useTemplateRef<ComponentExposed<typeof MultiStageModal>>('modal')

const ctx = createServerCompatibilityContext(modal, () => emit('save'))
provideServerCompatibilityContext(ctx)

function show(stageId?: string | null) {
	modal.value?.setStage(stageId ?? 0)
	modal.value?.show()
}

function handleHide() {
	ctx.resetContext()
}

function hide() {
	handleHide()
	modal.value?.hide()
}

defineExpose({
	show,
	hide,
})
</script>
