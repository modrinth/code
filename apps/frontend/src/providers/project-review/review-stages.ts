import useCategoriesStage from '@modrinth/moderation/src/data/stages/categories'
import useDescriptionStage from '@modrinth/moderation/src/data/stages/description'
import useDisclosuresStage from '@modrinth/moderation/src/data/stages/disclosures'
import useGalleryStage from '@modrinth/moderation/src/data/stages/gallery'
import useLicenseStage from '@modrinth/moderation/src/data/stages/license'
import useLinksStage from '@modrinth/moderation/src/data/stages/links'
import useMetadataStage from '@modrinth/moderation/src/data/stages/metadata'
import usePostApprovalStage from '@modrinth/moderation/src/data/stages/post-approval'
import useReReviewStage from '@modrinth/moderation/src/data/stages/re-review'
import useReuploadStage from '@modrinth/moderation/src/data/stages/reupload'
import useRulesStage from '@modrinth/moderation/src/data/stages/rules'
import useStatusAlertsStage from '@modrinth/moderation/src/data/stages/status-alerts'
import useSummaryStage from '@modrinth/moderation/src/data/stages/summary'
import useTitleSlugStage from '@modrinth/moderation/src/data/stages/title-slug'
import useUndefinedProjectStage from '@modrinth/moderation/src/data/stages/undefined-project'
import useVersionsStage from '@modrinth/moderation/src/data/stages/versions'
import { isShown, type StageNode } from '@modrinth/moderation/src/types/node'
import { createContext } from '@modrinth/ui'
import { computed, type Ref, ref, shallowRef } from 'vue'

import { injectProjectReviewPageContext } from './index'
import type { ReviewTarget } from './review'
import { injectReviewSession } from './review-session'

const targetStages = {
	title: 'title-slug',
	slug: 'title-slug',
	icon: 'rules',
	summary: 'summary',
	tags: 'tags',
	compatibility: 'metadata',
	reupload: 'reupload',
	're-review': 're-review',
	'post-approval': 'post-approval',
	'status-alerts': 'status-alerts',
	'undefined-project': 'undefined-project',
	link: 'links',
	license: 'license',
	'license-url': 'license',
	description: 'description',
	gallery: 'gallery',
	'gallery-image': 'gallery',
	disclosures: 'disclosures',
	versions: 'versions',
	version: 'versions',
} as const satisfies Record<ReviewTarget['kind'], string>

type StageId = (typeof targetStages)[ReviewTarget['kind']]
type ReviewStages = Record<StageId, StageNode>

export const [injectReviewStages, provideReviewStages] =
	createContext<ReturnType<typeof createReviewStages>>('ProjectReviewStages')

export function createReviewStages(projectId: Ref<string | undefined>) {
	const draft = ref('')
	const generating = ref(false)
	const registered = shallowRef<{ projectId: string; stages: ReviewStages }>()

	function register(id: string, stages: ReviewStages) {
		const entry = { projectId: id, stages }
		registered.value = entry
		return () => {
			if (registered.value === entry) registered.value = undefined
		}
	}

	function resolve(target: ReviewTarget) {
		const entry = registered.value
		if (!entry || entry.projectId !== projectId.value) return undefined
		const scope = targetStages[target.kind]
		const stage = entry.stages[scope]
		if (!isShown(stage)) return undefined
		return { projectId: entry.projectId, stage, scope }
	}

	return { register, resolve, draft, generating }
}

export function useReviewStageDefinitions(): ReviewStages {
	const { project, projectV2 } = injectProjectReviewPageContext()
	const session = injectReviewSession()
	const mainStages = {
		metadata: useMetadataStage(),
		reupload: useReuploadStage(),
		're-review': useReReviewStage(),
		'post-approval': usePostApprovalStage(),
		'undefined-project': useUndefinedProjectStage().shown(
			computed(
				() =>
					!!projectV2.value &&
					projectV2.value.versions.length === 0 &&
					!project.value?.minecraft_server,
			),
		),
		'title-slug': useTitleSlugStage(),
		rules: useRulesStage(),
		summary: useSummaryStage(),
		tags: useCategoriesStage(),
		links: useLinksStage(),
		license: useLicenseStage(),
		description: useDescriptionStage(),
		gallery: useGalleryStage(),
		disclosures: useDisclosuresStage(),
		versions: useVersionsStage(),
	}
	return {
		...mainStages,
		'status-alerts': useStatusAlertsStage(
			Object.values(mainStages),
			computed(() => session.readProject(project.value?.id ?? '')),
		),
	}
}
