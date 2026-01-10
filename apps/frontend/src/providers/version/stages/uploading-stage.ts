import type { StageConfigInput } from '@modrinth/ui'
import { markRaw } from 'vue'

import UploadingStage from '~/components/ui/create-project-version/stages/UploadingStage.vue'

import type { ManageVersionContextValue } from '../manage-version-modal'

export const stageConfig: StageConfigInput<ManageVersionContextValue> = {
	id: 'uploading',
	stageContent: markRaw(UploadingStage),
	title: 'Uploading',
	maxWidth: '500px',
	hideStageInBreadcrumb: true,
	skip: (ctx) => !ctx.isUploading.value,
	leftButtonConfig: () => null,
	rightButtonConfig: () => null,
	nonProgressStage: true,
}
