import { markRaw } from 'vue'

import type { StageConfigInput } from '../../../base'
import SetupTypeStage from '../components/SetupTypeStage.vue'
import { type CreationFlowContextValue, flowTypeHeadings } from '../creation-flow-context'

export const stageConfig: StageConfigInput<CreationFlowContextValue> = {
	id: 'setup-type',
	title: (ctx) => flowTypeHeadings[ctx.flowType],
	stageContent: markRaw(SetupTypeStage),
	leftButtonConfig: null,
	rightButtonConfig: null,
	maxWidth: '520px',
}
