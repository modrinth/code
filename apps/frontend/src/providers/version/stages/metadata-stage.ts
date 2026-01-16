import { LeftArrowIcon, RightArrowIcon, XIcon } from '@modrinth/assets'
import type { StageConfigInput } from '@modrinth/ui'
import { markRaw } from 'vue'

import MetadataStage from '~/components/ui/create-project-version/stages/MetadataStage.vue'

import type { ManageVersionContextValue } from '../manage-version-modal'

export const stageConfig: StageConfigInput<ManageVersionContextValue> = {
	id: 'metadata',
	stageContent: markRaw(MetadataStage),
	title: 'Metadata',
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
				}
			: {
					label: ctx.getNextLabel(),
					icon: RightArrowIcon,
					iconPosition: 'after',
					onClick: () => ctx.modal.value?.nextStage(),
				},
}
