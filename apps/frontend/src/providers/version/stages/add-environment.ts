import { LeftArrowIcon, RightArrowIcon } from '@modrinth/assets'
import type { StageConfigInput } from '@modrinth/ui'
import { markRaw } from 'vue'

import AddEnvironmentStage from '~/components/ui/create-project-version/stages/AddEnvironmentStage.vue'

import type { ManageVersionContextValue } from '../manage-version-modal'

export const stageConfig: StageConfigInput<ManageVersionContextValue> = {
	id: 'add-environment',
	stageContent: markRaw(AddEnvironmentStage),
	title: (ctx) => (ctx.editingVersion.value ? 'Edit environment' : 'Add environment'),
	skip: (ctx) =>
		ctx.noEnvironmentProject.value ||
		(!ctx.editingVersion.value && !!ctx.inferredVersionData.value?.environment) ||
		(ctx.editingVersion.value && !!ctx.draftVersion.value.environment),
	leftButtonConfig: (ctx) => ({
		label: 'Back',
		icon: LeftArrowIcon,
		onClick: () => ctx.modal.value?.prevStage(),
	}),
	rightButtonConfig: (ctx) => ({
		label: ctx.getNextLabel(),
		icon: RightArrowIcon,
		iconPosition: 'after',
		disabled: !ctx.draftVersion.value.environment,
		onClick: () => ctx.modal.value?.nextStage(),
	}),
}

export const fromDetailsStageConfig: StageConfigInput<ManageVersionContextValue> = {
	id: 'from-details-environment',
	stageContent: markRaw(AddEnvironmentStage),
	title: 'Edit environment',
	nonProgressStage: true,
	leftButtonConfig: (ctx) => ({
		label: 'Back',
		icon: LeftArrowIcon,
		disabled: !ctx.draftVersion.value.environment,
		onClick: () => ctx.modal.value?.setStage('add-details'),
	}),
	rightButtonConfig: (ctx) =>
		ctx.editingVersion.value
			? {
					...ctx.saveButtonConfig(),
					disabled: !ctx.draftVersion.value.environment,
				}
			: {
					label: ctx.getNextLabel(2),
					icon: RightArrowIcon,
					iconPosition: 'after',
					disabled: !ctx.draftVersion.value.environment,
					onClick: () => ctx.modal.value?.setStage(2),
				},
}
