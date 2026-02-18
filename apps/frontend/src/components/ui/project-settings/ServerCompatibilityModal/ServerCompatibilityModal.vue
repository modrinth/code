<template>
	<MultiStageModal
		ref="modal"
		:stages="ctx.stageConfigs"
		:context="ctx"
		@hide="handleHide"
		:fade="ctx.isSwitchingCompatibilityType.value ? 'danger' : 'standard'"
	/>
</template>

<script setup lang="ts">
import { injectProjectPageContext, MultiStageModal } from '@modrinth/ui'
import type { ComponentExposed } from 'vue-component-type-helpers'

import {
	createServerCompatibilityContext,
	provideServerCompatibilityContext,
	type CompatibilityType,
} from '../../../../providers/manage-server-compatibility-modal'

const modal = useTemplateRef<ComponentExposed<typeof MultiStageModal>>('modal')

const { projectV3 } = injectProjectPageContext()
const ctx = createServerCompatibilityContext(modal)
provideServerCompatibilityContext(ctx)

interface ShowModalOptions {
	stageId?: string | null
	updateContentKind?: CompatibilityType
	isSwitchingCompatibilityType?: boolean
}

async function show(options?: ShowModalOptions) {
	if (options?.updateContentKind) {
		ctx.compatibilityType.value = options.updateContentKind
		ctx.isEditingExistingCompatibility.value = true

		// Prefill existing values for vanilla
		const content = projectV3.value?.minecraft_java_server?.content
		if (options.updateContentKind === 'vanilla' && content && content.kind === 'vanilla') {
			ctx.supportedGameVersions.value = content.supported_game_versions ?? []
			ctx.recommendedGameVersion.value = content.recommended_game_version ?? null
		}

		await nextTick()
		modal.value?.setStage(1)
	} else {
		modal.value?.setStage(options?.stageId ?? 0)
	}

	if (options?.isSwitchingCompatibilityType) {
		ctx.isSwitchingCompatibilityType.value = true
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
