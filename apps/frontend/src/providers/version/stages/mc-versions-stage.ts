import { LeftArrowIcon, RightArrowIcon } from '@modrinth/assets'
import type { StageConfigInput } from '@modrinth/ui'
import { markRaw } from 'vue'

import McVersionsStage from '~/components/ui/create-project-version/stages/McVersionsStage.vue'

import type { ManageVersionContextValue } from '../manage-version-modal'

export const stageConfig: StageConfigInput<ManageVersionContextValue> = {
	id: 'add-mc-versions',
	stageContent: markRaw(McVersionsStage),
	title: (ctx) => (ctx.editingVersion.value ? 'Edit game versions' : 'Game versions'),
	skip: (ctx) =>
		(ctx.inferredVersionData.value?.game_versions?.length ?? 0) > 0 || !ctx.primaryFile.value,
	hideStageInBreadcrumb: (ctx) => !ctx.primaryFile.value || ctx.handlingNewFiles.value,

	cannotNavigateForward: (ctx) => ctx.draftVersion.value.game_versions.length === 0,
	leftButtonConfig: (ctx) => ({
		label: 'Back',
		icon: LeftArrowIcon,
		onClick: () => ctx.modal.value?.prevStage(),
	}),
	rightButtonConfig: (ctx) => ({
		label: ctx.getNextLabel(),
		icon: RightArrowIcon,
		iconPosition: 'after',
		disabled: ctx.draftVersion.value.game_versions.length === 0,
		onClick: () => ctx.modal.value?.nextStage(),
	}),
}

export const fromDetailsStageConfig: StageConfigInput<ManageVersionContextValue> = {
	id: 'from-details-mc-versions',
	stageContent: markRaw(McVersionsStage),
	title: 'Edit game versions',
	nonProgressStage: true,
	leftButtonConfig: (ctx) => ({
		label: 'Back',
		icon: LeftArrowIcon,
		disabled: ctx.draftVersion.value.game_versions.length === 0,
		onClick: () => ctx.modal.value?.setStage('metadata'),
	}),
	rightButtonConfig: (ctx) =>
		ctx.editingVersion.value
			? {
					...ctx.saveButtonConfig(),
					disabled: ctx.draftVersion.value.game_versions.length === 0,
				}
			: {
					label: 'Add details',
					icon: RightArrowIcon,
					iconPosition: 'after',
					disabled: ctx.draftVersion.value.game_versions.length === 0,
					onClick: () => ctx.modal.value?.setStage('add-details'),
				},
}
