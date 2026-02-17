<template>
	<MultiStageModal ref="modal" :stages="ctx.stageConfigs" :context="ctx" @hide="handleHide" />
</template>

<script setup lang="ts">
import { MultiStageModal } from '@modrinth/ui'
import type { ComponentExposed } from 'vue-component-type-helpers'

import {
	createServerCompatibilityContext,
	provideServerCompatibilityContext,
	type CompatibilityType,
} from './manage-server-compatibility-modal'

const modal = useTemplateRef<ComponentExposed<typeof MultiStageModal>>('modal')

const ctx = createServerCompatibilityContext(modal)
provideServerCompatibilityContext(ctx)

interface ShowModalOptions {
	stageId?: string | null
	updateContentKind?: CompatibilityType
}

async function show(options?: ShowModalOptions) {
	if (options?.updateContentKind) {
		ctx.compatibilityType.value = options.updateContentKind
		ctx.isEditingExistingCompatibility.value = true
		await nextTick()
		modal.value?.setStage(1)
	} else {
		modal.value?.setStage(options?.stageId ?? 0)
	}

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
