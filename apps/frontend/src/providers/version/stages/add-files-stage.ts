import { LeftArrowIcon, RightArrowIcon, XIcon } from '@modrinth/assets'
import type { StageConfigInput } from '@modrinth/ui'
import { markRaw } from 'vue'

import AddFilesStage from '~/components/ui/create-project-version/stages/AddFilesStage.vue'

import type { ManageVersionContextValue } from '../manage-version-modal'

export const stageConfig: StageConfigInput<ManageVersionContextValue> = {
	id: 'add-files',
	stageContent: markRaw(AddFilesStage),
	title: (ctx) => (ctx.editingVersion.value ? 'Edit files' : 'Files'),
	nonProgressStage: (ctx) => ctx.editingVersion.value,
	cannotNavigateForward: (ctx) => {
		const hasFiles =
			ctx.filesToAdd.value.length !== 0 ||
			(ctx.draftVersion.value.existing_files?.length ?? 0) !== 0
		return !hasFiles || ctx.handlingNewFiles.value
	},
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

		if (!hasFiles || ctx.handlingNewFiles.value) return null

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

		if (!hasFiles || ctx.handlingNewFiles.value) return null

		return {
			label: ctx.getNextLabel(),
			icon: RightArrowIcon,
			iconPosition: 'after',
			disabled: !hasFiles,
			onClick: () => ctx.modal.value?.nextStage(),
		}
	},
}

export const fromDetailsStageConfig: StageConfigInput<ManageVersionContextValue> = {
	id: 'from-details-files',
	stageContent: markRaw(AddFilesStage),
	title: 'Edit files',
	nonProgressStage: true,
	leftButtonConfig: (ctx) => {
		const hasFiles =
			ctx.filesToAdd.value.length !== 0 ||
			(ctx.draftVersion.value.existing_files?.length ?? 0) !== 0

		return {
			label: 'Back',
			icon: LeftArrowIcon,
			disabled: !hasFiles,
			onClick: () => ctx.modal.value?.setStage('metadata'),
		}
	},
	rightButtonConfig: (ctx) => {
		const hasFiles =
			ctx.filesToAdd.value.length !== 0 ||
			(ctx.draftVersion.value.existing_files?.length ?? 0) !== 0

		return ctx.editingVersion.value
			? {
					...ctx.saveButtonConfig(),
					label: 'Save files',
					disabled: !hasFiles || ctx.isSubmitting.value,
				}
			: {
					label: 'Add details',
					icon: RightArrowIcon,
					iconPosition: 'after',
					disabled: !hasFiles,
					onClick: () => ctx.modal.value?.setStage('add-details'),
				}
	},
}
