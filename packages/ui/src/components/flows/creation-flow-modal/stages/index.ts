import type { StageConfigInput } from '../../../base'
import type { CreationFlowContextValue } from '../creation-flow-context'
import { stageConfig as curseforgeStageConfig } from './curseforge-stage'
import { stageConfig as customSetupStageConfig } from './custom-setup-stage'
import { stageConfig as finalConfigStageConfig } from './final-config-stage'
import { stageConfig as importInstanceStageConfig } from './import-instance-stage'
import { stageConfig as inviteFriendsStageConfig } from './invite-friends-stage'
import { stageConfig as setupTypeStageConfig } from './setup-type-stage'
import { stageConfig as uploadingStageConfig } from './uploading-stage'

export const stageConfigs: StageConfigInput<CreationFlowContextValue>[] = [
	setupTypeStageConfig,
	importInstanceStageConfig,
	customSetupStageConfig,
	curseforgeStageConfig,
	finalConfigStageConfig,
	uploadingStageConfig,
	inviteFriendsStageConfig,
]
