import type { Labrinth } from '@modrinth/api-client'

import type { UpdateChannelPolicy } from '#ui/layouts/shared/content-tab/utils/update-channels'

import {
	type BrowseInstallContentType,
	type BrowseInstallPlanSource,
	type BrowseInstallPreferences,
	getInstallCandidates,
	getLatestMatchingInstallVersion,
	getSelectedInstallPreferences,
	normalizeInstallPreferences,
	type SelectedInstallPreferencesOptions,
} from './preferences'

export interface BrowseResolvedInstallContent {
	projectId: string
	versionId: string
}

export interface BrowseInstallProject {
	project_id: string
	latest_version?: string | null
	version_id?: string | null
	title?: string
	name?: string
	icon_url?: string | null
}

export interface BrowseInstallPlan<TProject extends BrowseInstallProject = BrowseInstallProject> {
	project: TProject
	projectId: string
	versionId: string
	contentType: BrowseInstallContentType
	preferences: BrowseInstallPreferences
	source: BrowseInstallPlanSource
	resolvedContent?: BrowseResolvedInstallContent[]
}

export interface BrowseInstallQueue<TProject extends BrowseInstallProject = BrowseInstallProject> {
	get: () => Map<string, BrowseInstallPlan<TProject>>
	set: (plans: Map<string, BrowseInstallPlan<TProject>>) => void
}

export interface ResolveInstallPlanOptions<
	TProject extends BrowseInstallProject,
> extends SelectedInstallPreferencesOptions {
	project: TProject
	contentType: BrowseInstallContentType
	targetPreferences?: BrowseInstallPreferences
	updateChannel?: UpdateChannelPolicy
	getProjectVersions: (projectId: string) => Promise<Labrinth.Versions.v2.Version[]>
}

export interface RequestInstallOptions<
	TProject extends BrowseInstallProject,
> extends ResolveInstallPlanOptions<TProject> {
	mode: 'queue' | 'immediate'
	queue?: BrowseInstallQueue<TProject>
	install?: (plan: BrowseInstallPlan<TProject>) => void | Promise<void>
}

export interface ResolveServerAddonInstallPlansOptions<TProject extends BrowseInstallProject> {
	plans: readonly BrowseInstallPlan<TProject>[]
	existingProjectIds: Iterable<string>
	resolvePlan: (
		plan: BrowseInstallPlan<TProject>,
		existingProjectIds: string[],
	) => Promise<BrowseResolvedInstallContent[]>
}

export async function resolveInstallPlan<TProject extends BrowseInstallProject>(
	options: ResolveInstallPlanOptions<TProject>,
): Promise<BrowseInstallPlan<TProject>> {
	const projectId = options.project.project_id
	if (!projectId) {
		throw new Error('No project is available for install.')
	}

	const selectedPreferences = getSelectedInstallPreferences(options)
	const targetPreferences = normalizeInstallPreferences(options.targetPreferences)
	const candidates = getInstallCandidates(selectedPreferences, targetPreferences)
	const versions = await options.getProjectVersions(projectId)
	let lastError: Error | null = null

	for (const candidate of candidates) {
		const version = getLatestMatchingInstallVersion(
			versions,
			candidate.preferences,
			options.updateChannel,
		)

		if (version) {
			return {
				project: options.project,
				projectId,
				versionId: version.id,
				contentType: options.contentType,
				preferences: candidate.preferences,
				source: candidate.source,
			}
		}

		lastError = createNoCompatibleVersionError(options.contentType, candidate.preferences)
	}

	throw lastError ?? new Error('No version found for this project.')
}

export async function requestInstall<TProject extends BrowseInstallProject>(
	options: RequestInstallOptions<TProject>,
) {
	const plan = await resolveInstallPlan(options)

	if (options.mode === 'queue') {
		if (!options.queue) {
			throw new Error('No install queue is available.')
		}

		const nextPlans = new Map(options.queue.get())
		nextPlans.set(plan.projectId, plan)
		options.queue.set(nextPlans)
		return plan
	}

	await options.install?.(plan)
	return plan
}

export function getServerAddonInstallPlanProjectIds<TProject extends BrowseInstallProject>(
	plans: Iterable<BrowseInstallPlan<TProject>>,
) {
	const projectIds = new Set<string>()
	for (const plan of plans) {
		projectIds.add(plan.projectId)
		for (const item of plan.resolvedContent ?? []) {
			projectIds.add(item.projectId)
		}
	}
	return projectIds
}

export async function resolveServerAddonInstallPlans<TProject extends BrowseInstallProject>({
	plans,
	existingProjectIds,
	resolvePlan,
}: ResolveServerAddonInstallPlansOptions<TProject>) {
	const installedProjectIds = new Set(existingProjectIds)
	const resolutionExistingProjectIds = Array.from(installedProjectIds)
	const resolvedPlans = await Promise.all(
		plans.map(
			async (plan) =>
				plan.resolvedContent ?? (await resolvePlan(plan, resolutionExistingProjectIds)),
		),
	)
	const explicitProjectIds = new Set(plans.map((plan) => plan.projectId))
	const addons = new Map<string, { project_id: string; version_id: string }>()

	for (const plan of plans) {
		if (installedProjectIds.has(plan.projectId)) continue
		addons.set(plan.projectId, {
			project_id: plan.projectId,
			version_id: plan.versionId,
		})
	}

	for (const content of resolvedPlans) {
		for (const item of content) {
			if (
				installedProjectIds.has(item.projectId) ||
				explicitProjectIds.has(item.projectId) ||
				addons.has(item.projectId)
			) {
				continue
			}
			addons.set(item.projectId, {
				project_id: item.projectId,
				version_id: item.versionId,
			})
		}
	}

	return Array.from(addons.values())
}

function createNoCompatibleVersionError(
	contentType: BrowseInstallContentType,
	preferences: BrowseInstallPreferences,
) {
	const versionLabel = preferences.gameVersions?.length
		? preferences.gameVersions.join(', ')
		: 'any game version'
	const loaderLabel = preferences.loaders?.length ? preferences.loaders.join(', ') : 'any loader'

	return new Error(
		contentType === 'datapack'
			? `No compatible version found for ${versionLabel}.`
			: `No compatible version found for ${versionLabel} / ${loaderLabel}.`,
	)
}
