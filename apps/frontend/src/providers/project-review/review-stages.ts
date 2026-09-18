import useDescriptionStage from '@modrinth/moderation/src/data/stages/description'
import useDisclosuresStage from '@modrinth/moderation/src/data/stages/disclosures'
import useGalleryStage from '@modrinth/moderation/src/data/stages/gallery'
import useLicenseStage from '@modrinth/moderation/src/data/stages/license'
import useLinksStage from '@modrinth/moderation/src/data/stages/links'
import useRulesStage from '@modrinth/moderation/src/data/stages/rules'
import useSummaryStage from '@modrinth/moderation/src/data/stages/summary'
import useTitleSlugStage from '@modrinth/moderation/src/data/stages/title-slug'
import useVersionsStage from '@modrinth/moderation/src/data/stages/versions'
import { isShown, type StageNode } from '@modrinth/moderation/src/types/node'
import { createContext } from '@modrinth/ui'
import { type Ref, shallowRef } from 'vue'

import type { ReviewTarget } from './review'

const targetStages = {
	title: 'title-slug',
	slug: 'title-slug',
	icon: 'rules',
	summary: 'summary',
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

	return { register, resolve }
}

export function useReviewStageDefinitions(): ReviewStages {
	return {
		'title-slug': useTitleSlugStage(),
		rules: useRulesStage(),
		summary: useSummaryStage(),
		links: useLinksStage(),
		license: useLicenseStage(),
		description: useDescriptionStage(),
		gallery: useGalleryStage(),
		disclosures: useDisclosuresStage(),
		versions: useVersionsStage(),
	}
}
