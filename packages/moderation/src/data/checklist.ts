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
import usePostApprovalStage from './stages/post-approval'
import useReuploadsStage from './stages/reupload'
import useRuleFollowingStage from './stages/rule-following'
import useStatusAlertsStage from './stages/status-alerts'
import useSummaryStage from './stages/summary'
import useTitleSlugStage from './stages/title-slug'
import useUndefinedProjectStage from './stages/undefined-project'
import useVersionsStage from './stages/versions'

export function useStages(
	globalState: Ref<Record<string, Record<string, NodeState>>>,
): StageNodeBuilder[] {
	const mainStages: StageNodeBuilder[] = [
		useTitleSlugStage(),
		useSummaryStage(),
		useDescriptionStage(),
		useLinksStage(),
		useLicenseStage(),
		useCategoriesStage(),
		useMetadataStage(),
		useGalleryStage(),
		useVersionsStage(),
		useReuploadsStage(),
		usePermissionsStage(),
		useRuleFollowingStage(),
		useUndefinedProjectStage(),
		usePostApprovalStage(),
	]
	provide(STAGES_KEY, ref(mainStages))
	return [...mainStages, useStatusAlertsStage(mainStages, globalState)]
}

export const stages: ReadonlyArray<StageFn> = []

export default group()
