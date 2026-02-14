import { markRaw } from 'vue'

import type { StageConfigInput } from '../../../base'
import WorldTypeStage from '../components/WorldTypeStage.vue'
import { type CreationFlowContextValue, flowTypeHeadings } from '../creation-flow-context'

export const stageConfig: StageConfigInput<CreationFlowContextValue> = {
	id: 'world-type',
	title: (ctx) => flowTypeHeadings[ctx.flowType],
	stageContent: markRaw(WorldTypeStage),
	nonProgressStage: true,
	leftButtonConfig: null,
	rightButtonConfig: null,
	maxWidth: '520px',
}
