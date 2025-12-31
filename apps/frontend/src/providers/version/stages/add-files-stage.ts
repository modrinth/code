import { RightArrowIcon, XIcon } from '@modrinth/assets'
import type { StageConfigInput } from '@modrinth/ui'
import { markRaw } from 'vue'

import AddFilesStage from '~/components/ui/create-project-version/stages/AddFilesStage.vue'

import type { ManageVersionContextValue } from '../manage-version-modal'

export const stageConfig: StageConfigInput<ManageVersionContextValue> = {
	id: 'add-files',
	stageContent: markRaw(AddFilesStage),
	title: (ctx) => (ctx.editingVersion.value ? 'Edit files' : 'Add files'),
	leftButtonConfig: (ctx) => {
		const hasFiles =
			ctx.filesToAdd.value.length !== 0 ||
			(ctx.draftVersion.value.existing_files?.length ?? 0) !== 0

		if (ctx.editingVersion.value)
			return {
				label: 'Cancel',
				icon: XIcon,
				onClick: () => ctx.modal.value?.hide(),
			}

		if (!hasFiles) return null

		return {
			label: 'Cancel',
			icon: XIcon,
			onClick: () => ctx.modal.value?.hide(),
		}
	},
	rightButtonConfig: (ctx) => {
		const hasFiles =
			ctx.filesToAdd.value.length !== 0 ||
			(ctx.draftVersion.value.existing_files?.length ?? 0) !== 0

		if (ctx.editingVersion.value)
			return {
				...ctx.saveButtonConfig(),
				label: 'Save files',
				disabled: ctx.isSubmitting.value,
			}

		if (!hasFiles) return null

		return {
			label: ctx.getNextLabel(),
			icon: RightArrowIcon,
			iconPosition: 'after',
			disabled: !hasFiles,
			onClick: () => ctx.modal.value?.nextStage(),
		}
	},
	nonProgressStage: (ctx) => ctx.editingVersion.value,
}
