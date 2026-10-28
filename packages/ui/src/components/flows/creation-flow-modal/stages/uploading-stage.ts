import { markRaw } from 'vue'

import type { StageConfigInput } from '../../../base'
import UploadingStage from '../components/UploadingStage.vue'
import type { CreationFlowContextValue } from '../creation-flow-context'

export const stageConfig: StageConfigInput<CreationFlowContextValue> = {
	id: 'uploading',
	title: (ctx) =>
		ctx.formatMessage({
			id: 'servers.setup.uploading-modpack.header',
			defaultMessage: 'Uploading modpack',
		}),
	stageContent: markRaw(UploadingStage),
	skip: (ctx) => ctx.uploadProgress.value === null,
	nonProgressStage: true,
	disableClose: true,
	leftButtonConfig: null,
	rightButtonConfig: null,
	maxWidth: '448px',
}
