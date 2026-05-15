import {
	fromDetailsStageConfig as fromDetailsFilesStageConfig,
	stageConfig as addFilesStageConfig,
} from './add-files-stage'
import { fromDetailsStageConfig as fromDetailsDependenciesStageConfig } from './dependencies-stage'
import { stageConfig as detailsStageConfig } from './details-stage'
import {
	fromDetailsStageConfig as fromDetailsEnvironmentStageConfig,
	stageConfig as environmentStageConfig,
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
	metadataStageConfig,
	detailsStageConfig,

	// Non-progress stages for editing from metadata/details pages
	fromDetailsLoadersStageConfig,
	fromDetailsMcVersionsStageConfig,
	fromDetailsEnvironmentStageConfig,
	fromDetailsFilesStageConfig,
	fromDetailsDependenciesStageConfig,
]
