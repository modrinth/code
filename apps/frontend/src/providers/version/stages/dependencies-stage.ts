import { LeftArrowIcon, RightArrowIcon, XIcon } from '@modrinth/assets'
import type { StageConfigInput } from '@modrinth/ui'
import { markRaw } from 'vue'

import DependenciesStage from '~/components/ui/create-project-version/stages/DependenciesStage.vue'

import type { ManageVersionContextValue } from '../manage-version-modal'

export const stageConfig: StageConfigInput<ManageVersionContextValue> = {
	id: 'add-dependencies',
	stageContent: markRaw(DependenciesStage),
	title: (ctx) => (ctx.editingVersion.value ? 'Edit dependencies' : 'Dependencies'),
	skip: (ctx) => ctx.suggestedDependencies.value != null || ctx.projectType.value === 'modpack',
	leftButtonConfig: (ctx) =>
		ctx.editingVersion.value
			? {
					label: 'Cancel',
					icon: XIcon,
					onClick: () => ctx.modal.value?.hide(),
				}
			: {
					label: 'Back',
					icon: LeftArrowIcon,
					onClick: () => ctx.modal.value?.prevStage(),
				},
	rightButtonConfig: (ctx) =>
		ctx.editingVersion.value
			? ctx.saveButtonConfig()
			: {
					label: ctx.getNextLabel(),
					icon: RightArrowIcon,
					iconPosition: 'after',
					onClick: () => ctx.modal.value?.nextStage(),
				},
	nonProgressStage: (ctx) => ctx.editingVersion.value,
}

export const fromDetailsStageConfig: StageConfigInput<ManageVersionContextValue> = {
	id: 'from-details-dependencies',
	stageContent: markRaw(DependenciesStage),
	title: 'Edit dependencies',
	nonProgressStage: true,
	leftButtonConfig: (ctx) => ({
		label: 'Back',
		icon: LeftArrowIcon,
		onClick: () => ctx.modal.value?.setStage('metadata'),
	}),
	rightButtonConfig: (ctx) =>
		ctx.editingVersion.value
			? {
					...ctx.saveButtonConfig(),
				}
			: {
					label: 'Add details',
					icon: RightArrowIcon,
					iconPosition: 'after',
					onClick: () => ctx.modal.value?.setStage('add-details'),
				},
}
