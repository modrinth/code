import type { Ref } from 'vue'
import { provide, ref } from 'vue'

import type { NodeState, StageFn, StageNodeBuilder } from '../types/node'
import { group, STAGES_KEY } from '../types/node'
import useCategoriesStage from './stages/categories'
import useDescriptionStage from './stages/description'
import useGalleryStage from './stages/gallery'
import useLicenseStage from './stages/license'
import useLinksStage from './stages/links'
import useMetadataStage from './stages/metadata'
import usePermissionsStage from './stages/permissions'
import usePostApprovalStage from './stages/postApproval'
import useReReviewStage from './stages/reReview'
import useReuploadsStage from './stages/reupload'
import useOtherRulesStage from './stages/otherRules'
import useStatusAlertsStage from './stages/statusAlerts'
import useSummaryStage from './stages/summary'
import useTitleSlugStage from './stages/titleSlug'
import useUndefinedProjectStage from './stages/undefinedProject'
import useVersionsStage from './stages/versions'

export function useStages(
	globalState: Ref<Record<string, Record<string, NodeState>>>,
): StageNodeBuilder[] {
	const mainStages: StageNodeBuilder[] = [
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
		useVersionsStage(),
		useMetadataStage(),
		useReuploadsStage(),
		usePermissionsStage(),
		useOtherRulesStage(),
	]
	provide(STAGES_KEY, ref(mainStages))
	return [...mainStages, useStatusAlertsStage(mainStages, globalState)]
}

export const stages: ReadonlyArray<StageFn> = []

export default group()
