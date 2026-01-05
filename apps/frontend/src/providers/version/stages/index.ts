import {
	stageConfig as addFilesStageConfig,
	fromDetailsStageConfig as fromDetailsFilesStageConfig,
} from './add-files-stage'
import {
	stageConfig as dependenciesStageConfig,
	fromDetailsStageConfig as fromDetailsDependenciesStageConfig,
} from './dependencies-stage'
import { stageConfig as detailsStageConfig } from './details-stage'
import {
	stageConfig as environmentStageConfig,
	fromDetailsStageConfig as fromDetailsEnvironmentStageConfig,
} from './environment-stage'
import {
	fromDetailsStageConfig as fromDetailsLoadersStageConfig,
	stageConfig as loadersStageConfig,
} from './loaders-stage'
import {
	fromDetailsStageConfig as fromDetailsMcVersionsStageConfig,
	stageConfig as mcVersionsStageConfig,
} from './mc-versions-stage'
import { stageConfig as metadataStageConfig } from './metadata-stage'

export const stageConfigs = [
	addFilesStageConfig,
	loadersStageConfig,
	mcVersionsStageConfig,
	environmentStageConfig,
	dependenciesStageConfig,
	metadataStageConfig,
	detailsStageConfig,

	// Non-progress stages for editing from details page
	fromDetailsLoadersStageConfig,
	fromDetailsMcVersionsStageConfig,
	fromDetailsEnvironmentStageConfig,
	fromDetailsFilesStageConfig,
	fromDetailsDependenciesStageConfig,
]
