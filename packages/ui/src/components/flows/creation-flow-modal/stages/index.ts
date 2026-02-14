import type { StageConfigInput } from '../../../base'
import type { CreationFlowContextValue } from '../creation-flow-context'
import { stageConfig as confirmStageConfig } from './confirm-stage'
import { stageConfig as customSetupStageConfig } from './custom-setup-stage'
import { stageConfig as finalConfigStageConfig } from './final-config-stage'
import { stageConfig as modpackStageConfig } from './modpack-stage'
import { stageConfig as worldTypeStageConfig } from './world-type-stage'

export const stageConfigs: StageConfigInput<CreationFlowContextValue>[] = [
	worldTypeStageConfig,
	modpackStageConfig,
	customSetupStageConfig,
	finalConfigStageConfig,
	confirmStageConfig,
]
