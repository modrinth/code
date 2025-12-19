import { LeftArrowIcon, RightArrowIcon } from '@modrinth/assets'
import type { StageConfigInput } from '@modrinth/ui'
import { markRaw } from 'vue'

import AddLoadersStage from '~/components/ui/create-project-version/stages/AddLoadersStage.vue'

import type { ManageVersionContextValue } from '../manage-version-modal'

export const stageConfig: StageConfigInput<ManageVersionContextValue> = {
	id: 'add-loaders',
	stageContent: markRaw(AddLoadersStage),
	title: (ctx) => (ctx.editingVersion.value ? 'Edit loaders' : 'Add loaders'),
	skip: (ctx) =>
		ctx.noLoadersProject.value ||
		(ctx.inferredVersionData.value?.loaders?.length ?? 0) > 0 ||
		ctx.editingVersion.value,
	leftButtonConfig: (ctx) => ({
		label: 'Back',
		icon: LeftArrowIcon,
		onClick: () => ctx.modal.value?.prevStage(),
	}),
	rightButtonConfig: (ctx) => ({
		label: ctx.getNextLabel(),
		icon: RightArrowIcon,
		iconPosition: 'after',
		disabled: ctx.draftVersion.value.loaders.length === 0,
		onClick: () => ctx.modal.value?.nextStage(),
	}),
}

export const fromDetailsStageConfig: StageConfigInput<ManageVersionContextValue> = {
	id: 'from-details-loaders',
	stageContent: markRaw(AddLoadersStage),
	title: 'Edit loaders',
	nonProgressStage: true,
	leftButtonConfig: (ctx) => ({
		label: 'Back',
		icon: LeftArrowIcon,
		disabled: ctx.draftVersion.value.loaders.length === 0,
		onClick: () => ctx.modal.value?.setStage('add-details'),
	}),
	rightButtonConfig: (ctx) =>
		ctx.editingVersion.value
			? {
					...ctx.saveButtonConfig(),
					disabled: ctx.draftVersion.value.loaders.length === 0,
				}
			: {
					label: ctx.getNextLabel(2),
					icon: RightArrowIcon,
					iconPosition: 'after',
					disabled: ctx.draftVersion.value.loaders.length === 0,
					onClick: () => ctx.modal.value?.setStage(2),
				},
}
