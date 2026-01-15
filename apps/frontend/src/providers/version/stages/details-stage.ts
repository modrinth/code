import { LeftArrowIcon, PlusIcon, SaveIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import type { StageConfigInput } from '@modrinth/ui'
import { markRaw } from 'vue'

import DetailsStage from '~/components/ui/create-project-version/stages/DetailsStage.vue'

import type { ManageVersionContextValue } from '../manage-version-modal'

export const stageConfig: StageConfigInput<ManageVersionContextValue> = {
	id: 'add-details',
	stageContent: markRaw(DetailsStage),
	title: (ctx) => (ctx.editingVersion.value ? 'Edit details' : 'Details'),
	maxWidth: '744px',
	disableClose: (ctx) => ctx.isUploading.value,
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
					disabled: ctx.isUploading.value,
					onClick: () => ctx.modal.value?.prevStage(),
				},
	rightButtonConfig: (ctx) => ({
		label: ctx.editingVersion.value
			? 'Save changes'
			: ctx.isUploading.value
				? ctx.uploadProgress.value.progress >= 1
					? 'Creating version'
					: `Uploading ${Math.round(ctx.uploadProgress.value.progress * 100)}%`
				: 'Create version',
		icon: ctx.isSubmitting.value ? SpinnerIcon : ctx.editingVersion.value ? SaveIcon : PlusIcon,
		iconPosition: 'before',
		iconClass: ctx.isSubmitting.value ? 'animate-spin' : undefined,
		color: 'green',
		disabled: ctx.isSubmitting.value,
		onClick: () =>
			ctx.editingVersion.value ? ctx.handleSaveVersionEdits() : ctx.handleCreateVersion(),
	}),
	nonProgressStage: (ctx) => ctx.editingVersion.value,
}
