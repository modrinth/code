import { LeftArrowIcon, RightArrowIcon } from '@modrinth/assets'
import type { StageConfigInput } from '@modrinth/ui'
import { markRaw } from 'vue'

import EnvironmentStage from '~/components/ui/create-project-version/stages/EnvironmentStage.vue'

import type { ManageVersionContextValue } from '../manage-version-modal'

export const stageConfig: StageConfigInput<ManageVersionContextValue> = {
	id: 'add-environment',
	stageContent: markRaw(EnvironmentStage),
	title: (ctx) => (ctx.editingVersion.value ? 'Edit environment' : 'Environment'),
	skip: (ctx) =>
		ctx.noEnvironmentProject.value ||
		(!ctx.editingVersion.value && !!ctx.inferredVersionData.value?.environment) ||
		(ctx.editingVersion.value && !!ctx.draftVersion.value.environment),
	hideStageInBreadcrumb: (ctx) => !ctx.primaryFile.value || ctx.handlingNewFiles.value,
	cannotNavigateForward: (ctx) => !ctx.draftVersion.value.environment,
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
	stageContent: markRaw(EnvironmentStage),
	title: 'Edit environment',
	nonProgressStage: true,
	leftButtonConfig: (ctx) => ({
		label: 'Back',
		icon: LeftArrowIcon,
		disabled: !ctx.draftVersion.value.environment,
		onClick: () => ctx.modal.value?.setStage('metadata'),
	}),
	rightButtonConfig: (ctx) =>
		ctx.editingVersion.value
			? {
					...ctx.saveButtonConfig(),
					disabled: !ctx.draftVersion.value.environment,
				}
			: {
					label: 'Add details',
					icon: RightArrowIcon,
					iconPosition: 'after',
					disabled: !ctx.draftVersion.value.environment,
					onClick: () => ctx.modal.value?.setStage('add-details'),
				},
}
