import { stageConfig as addChangelogStageConfig } from './add-changelog'
import { stageConfig as addFilesStageConfig } from './add-files-stage'
import {
	stageConfig as addMcVersionsStageConfig,
	fromDetailsStageConfig as fromDetailsMcVersionsStageConfig,
} from './add-mc-versions-stage'
import { stageConfig as addDependenciesStageConfig } from './dependencies-stage'
import { stageConfig as addDetailsStageConfig } from './details-stage'
import {
	stageConfig as addEnvironmentStageConfig,
	fromDetailsStageConfig as fromDetailsEnvironmentStageConfig,
} from './environment-stage'
import {
	stageConfig as addLoadersStageConfig,
	fromDetailsStageConfig as fromDetailsLoadersStageConfig,
} from './loaders-stage'

export const stageConfigs = [
	addFilesStageConfig,
	addLoadersStageConfig,
	addMcVersionsStageConfig,
	addEnvironmentStageConfig,
	addDependenciesStageConfig,
	addDetailsStageConfig, // to be renamed metadata stage
	addChangelogStageConfig, // to be renamed details stage
	// Non-progress stages for editing from details page
	fromDetailsLoadersStageConfig,
	fromDetailsMcVersionsStageConfig,
	fromDetailsEnvironmentStageConfig,
]
