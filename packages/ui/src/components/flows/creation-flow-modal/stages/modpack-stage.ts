import { markRaw } from 'vue'

import type { StageConfigInput } from '../../../base'
import ModpackStage from '../components/ModpackStage.vue'
import type { CreationFlowContextValue } from '../creation-flow-context'

export const stageConfig: StageConfigInput<CreationFlowContextValue> = {
	id: 'modpack',
	title: 'Choose modpack',
	stageContent: markRaw(ModpackStage),
	skip: (ctx) => ctx.setupType.value !== 'modpack' || ctx.isImportMode.value,
	leftButtonConfig: null,
	rightButtonConfig: null,
	maxWidth: '520px',
}
