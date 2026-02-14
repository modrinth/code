import type { StageConfigInput } from '../../../../base'
import { markRaw } from 'vue'
import WorldTypeStage from '../components/WorldTypeStage.vue'
import type { CreateWorldContextValue } from '../create-world-context'

export const stageConfig: StageConfigInput<CreateWorldContextValue> = {
	id: 'world-type',
	title: 'Create world',
	stageContent: markRaw(WorldTypeStage),
	nonProgressStage: true,
	leftButtonConfig: null,
	rightButtonConfig: null,
	maxWidth: '520px',
}
