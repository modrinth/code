import type { Ref } from 'vue'

import type { NodeState, StageNode } from '../types/node'
import useCategoriesStage from './stages/categories'
import useDescriptionStage from './stages/description'
import useGalleryStage from './stages/gallery'
import useLicenseStage from './stages/license'
import useLinksStage from './stages/links'
import useMetadataStage from './stages/metadata'
import usePermissionsStage from './stages/permissions'
import usePostApprovalStage from './stages/post-approval'
import useReReviewStage from './stages/re-review'
import useReuploadsStage from './stages/reupload'
import useRulesStage from './stages/rules'
import useStatusAlertsStage from './stages/status-alerts'
import useSummaryStage from './stages/summary'
import useTitleSlugStage from './stages/title-slug'
import useUndefinedProjectStage from './stages/undefined-project'
import useVersionsStage from './stages/versions'

export function useStages(globalState: Ref<Record<string, Record<string, NodeState>>>): StageNode[] {
	const mainStages: StageNode[] = [
		usePostApprovalStage(),
		useUndefinedProjectStage(),
		useReReviewStage(),
		useTitleSlugStage(),
		useSummaryStage(),
		useDescriptionStage(),
		useGalleryStage(),
		useLinksStage(),
		useLicenseStage(),
		useCategoriesStage(),
		useMetadataStage(),
		useVersionsStage(),
		useReuploadsStage(),
		usePermissionsStage(),
		useRulesStage(),
	]
	return [...mainStages, useStatusAlertsStage(mainStages, globalState)]
}
