import { LeftArrowIcon, PlusIcon, SpinnerIcon } from '@modrinth/assets'
import type { StageConfigInput } from '@modrinth/ui'
import { markRaw } from 'vue'

import AddChangelogStage from '~/components/ui/create-project-version/stages/AddChangelogStage.vue'

import type { ManageVersionContextValue } from '../manage-version-modal'

export const stageConfig: StageConfigInput<ManageVersionContextValue> = {
	id: 'add-changelog',
	stageContent: markRaw(AddChangelogStage),
	title: (ctx) => (ctx.editingVersion.value ? 'Edit changelog' : 'Add changelog'),
	leftButtonConfig: (ctx) => ({
		label: 'Back',
		icon: LeftArrowIcon,
		onClick: () => ctx.modal.value?.prevStage(),
	}),
	rightButtonConfig: (ctx) => ({
		label: ctx.editingVersion.value ? 'Save changes' : 'Create version',
		icon: ctx.isSubmitting.value ? SpinnerIcon : PlusIcon,
		iconPosition: 'before',
		iconClass: ctx.isSubmitting.value ? 'animate-spin' : undefined,
		color: 'green',
		disabled: ctx.isSubmitting.value,
		onClick: () =>
			ctx.editingVersion.value ? ctx.handleSaveVersionEdits() : ctx.handleCreateVersion(),
	}),
}
