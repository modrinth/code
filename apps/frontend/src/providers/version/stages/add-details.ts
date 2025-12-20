import { LeftArrowIcon, RightArrowIcon, XIcon } from '@modrinth/assets'
import type { StageConfigInput } from '@modrinth/ui'
import { markRaw } from 'vue'

import AddDetailsStage from '~/components/ui/create-project-version/stages/AddDetailsStage.vue'

import type { ManageVersionContextValue } from '../manage-version-modal'

export const stageConfig: StageConfigInput<ManageVersionContextValue> = {
	id: 'add-details',
	stageContent: markRaw(AddDetailsStage),
	title: (ctx) => (ctx.editingVersion.value ? 'Edit details' : 'Add details'),
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
			? {
					...ctx.saveButtonConfig(),
					disabled:
						ctx.draftVersion.value.version_number.trim().length === 0 || ctx.isSubmitting.value,
				}
			: {
					label: ctx.getNextLabel(),
					icon: RightArrowIcon,
					iconPosition: 'after',
					disabled: ctx.draftVersion.value.version_number.trim().length === 0,
					onClick: () => ctx.modal.value?.nextStage(),
				},
	nonProgressStage: (ctx) => ctx.editingVersion.value,
}
