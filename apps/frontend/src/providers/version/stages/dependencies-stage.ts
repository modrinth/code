import { PlusIcon, XIcon } from '@modrinth/assets'
import type { StageConfigInput } from '@modrinth/ui'
import { markRaw } from 'vue'

import DependenciesStage from '~/components/ui/create-project-version/stages/DependenciesStage.vue'

import type { ManageVersionContextValue } from '../manage-version-modal'

export const fromDetailsStageConfig: StageConfigInput<ManageVersionContextValue> = {
	id: 'from-details-dependencies',
	stageContent: markRaw(DependenciesStage),
	title: 'Add dependency',
	nonProgressStage: true,
	leftButtonConfig: (ctx) => ({
		label: 'Cancel',
		icon: XIcon,
		onClick: () => {
			ctx.resetNewDependency()
			ctx.modal.value?.setStage('metadata')
		},
	}),
	rightButtonConfig: (ctx) => ({
		label: 'Add dependency',
		icon: PlusIcon,
		iconPosition: 'before',
		color: 'green',
		disabled: !ctx.newDependencyProjectId.value,
		onClick: () => {
			if (ctx.addNewDependency()) ctx.modal.value?.setStage('metadata')
		},
	}),
}
