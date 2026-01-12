import { LeftArrowIcon, RightArrowIcon } from '@modrinth/assets'
import type { StageConfigInput } from '@modrinth/ui'
import { markRaw } from 'vue'

import AddMcVersionsStage from '~/components/ui/create-project-version/stages/AddMcVersionsStage.vue'

import type { ManageVersionContextValue } from '../manage-version-modal'

export const stageConfig: StageConfigInput<ManageVersionContextValue> = {
	id: 'add-mc-versions',
	stageContent: markRaw(AddMcVersionsStage),
	title: (ctx) => (ctx.editingVersion.value ? 'Edit game versions' : 'Add game versions'),
	skip: (ctx) =>
		(ctx.inferredVersionData.value?.game_versions?.length ?? 0) > 0 || ctx.editingVersion.value,
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
	stageContent: markRaw(AddMcVersionsStage),
	title: 'Edit game versions',
	nonProgressStage: true,
	leftButtonConfig: (ctx) => ({
		label: 'Back',
		icon: LeftArrowIcon,
		disabled: ctx.draftVersion.value.game_versions.length === 0,
		onClick: () => ctx.modal.value?.setStage('add-details'),
	}),
	rightButtonConfig: (ctx) =>
		ctx.editingVersion.value
			? {
					...ctx.saveButtonConfig(),
					disabled: ctx.draftVersion.value.game_versions.length === 0,
				}
			: {
					label: ctx.getNextLabel(2),
					icon: RightArrowIcon,
					iconPosition: 'after',
					disabled: ctx.draftVersion.value.game_versions.length === 0,
					onClick: () => ctx.modal.value?.setStage(2),
				},
}
