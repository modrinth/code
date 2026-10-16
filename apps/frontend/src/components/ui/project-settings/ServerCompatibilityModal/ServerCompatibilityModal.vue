<template>
	<MultiStageModal
		ref="modal"
		:stages="ctx.stageConfigs"
		:context="ctx"
		:fade="ctx.isSwitchingCompatibilityType.value ? 'danger' : 'standard'"
		@hide="handleHide"
	/>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { MultiStageModal } from '@modrinth/ui'
import type { ComponentExposed } from 'vue-component-type-helpers'

import {
	type CompatibilityType,
	createServerCompatibilityContext,
	provideServerCompatibilityContext,
} from '../../../../providers/manage-server-compatibility-modal'

const modal = useTemplateRef<ComponentExposed<typeof MultiStageModal>>('modal')

const props = defineProps<{
	project: Labrinth.Projects.v3.Project | undefined
	saveProject: (patch: Labrinth.Projects.v3.EditProjectRequest) => Promise<boolean>
}>()
const projectV3 = computed(() => props.project)
const ctx = createServerCompatibilityContext(modal, projectV3, (patch) => props.saveProject(patch))
provideServerCompatibilityContext(ctx)
const emit = defineEmits<{ busy: [value: boolean] }>()
watch(ctx.isSubmitting, (value) => emit('busy', value), { flush: 'sync' })

interface ShowModalOptions {
	stageId?: string | null
	updateContentKind?: CompatibilityType
	isSwitchingCompatibilityType?: boolean
}

async function show(options?: ShowModalOptions) {
	const content = projectV3.value?.minecraft_java_server?.content

	if (options?.updateContentKind) {
		ctx.compatibilityType.value = options.updateContentKind
		ctx.isEditingExistingCompatibility.value = true

		// Prefill existing values for vanilla
		if (options.updateContentKind === 'vanilla' && content && content.kind === 'vanilla') {
			ctx.supportedGameVersions.value = content.supported_game_versions ?? []
			ctx.recommendedGameVersion.value = content.recommended_game_version ?? null
		}

		if (options.updateContentKind === 'published-modpack') {
			ctx.selectedProjectId.value = content?.kind === 'modpack' ? content.project_id || '' : ''
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
