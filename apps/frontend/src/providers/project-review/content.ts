import { injectModrinthClient } from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type Ref, watch } from 'vue'

import {
	flattenFileReports,
	getFileDetailCount,
} from '~/components/ui/moderation/tech-review/helpers'
import { useTechReviewDecisions } from '~/components/ui/moderation/tech-review/use-tech-review-decisions'
import { projectQueryOptions } from '~/composables/queries/project'

export function useReviewContent(projectId: Ref<string>) {
	const client = injectModrinthClient()
	const queryClient = useQueryClient()
	const versionsV2Query = useQuery(
		computed(() => ({
			...projectQueryOptions.versionsV2(projectId.value, client),
			enabled: !!projectId.value,
		})),
	)
	const versionsQuery = useQuery(
		computed(() => {
			const id = projectId.value
			const options = projectQueryOptions.versionsV3(id, client)
			return {
				...options,
				enabled: !!id,
				queryFn: async () => {
					const [versions, project] = await Promise.all([
						options.queryFn(),
						queryClient.fetchQuery(projectQueryOptions.v3(id, client)),
					])
					const loadedIds = new Set(versions.map((version) => version.id))
					const missingIds = project.versions.filter((versionId) => !loadedIds.has(versionId))
					/** The list endpoint can transiently omit versions that are still available by ID. */
					for (let index = 0; index < missingIds.length; index += 100) {
						const recovered = await client.labrinth.versions_v3.getVersions(
							missingIds.slice(index, index + 100),
						)
						for (const version of recovered) {
							if (loadedIds.has(version.id)) continue
							loadedIds.add(version.id)
							versions.push(version)
						}
					}
					if (missingIds.some((versionId) => !loadedIds.has(versionId))) {
						throw new Error('Could not load all project versions')
					}
					return versions
				},
			}
		}),
	)
	const reportsQuery = useQuery(
		computed(() => ({
			queryKey: ['tech-review-project-report', projectId.value],
			queryFn: () => client.labrinth.tech_review_internal.getProjectReport(projectId.value),
			enabled: !!projectId.value,
		})),
	)
	const versions = computed(() =>
		(versionsQuery.data.value ?? [])
			.filter((version) => version.name !== '__synthetic')
			.toSorted((a, b) => Date.parse(b.date_published) - Date.parse(a.date_published)),
	)
	const reports = computed(() =>
		flattenFileReports(reportsQuery.data.value?.project_report?.versions ?? []),
	)
	const decisions = useTechReviewDecisions(reports)
	watch(projectId, decisions.resetDecisions, { flush: 'sync' })
	const pendingReports = computed(() =>
		reports.value.filter((file) => decisions.getFileMarkedCount(file) < getFileDetailCount(file)),
	)
	const completedReports = computed(() =>
		reports.value.filter(
			(file) =>
				getFileDetailCount(file) > 0 &&
				decisions.getFileMarkedCount(file) === getFileDetailCount(file),
		),
	)
	const unflaggedReports = computed(() =>
		reports.value.filter((file) => getFileDetailCount(file) === 0),
	)
	const contentCounts = computed(() => ({
		versions: versionsQuery.isSuccess.value ? versions.value.length : undefined,
		'tech-review': reportsQuery.isSuccess.value ? pendingReports.value.length : undefined,
	}))

	async function refreshReports() {
		await Promise.all([
			queryClient.invalidateQueries({
				queryKey: ['tech-review-project-report'],
			}),
			queryClient.invalidateQueries({ queryKey: ['tech-reviews'] }),
		])
	}

	return {
		versionsV2Query,
		versionsV2: computed(() =>
			(versionsV2Query.data.value ?? [])
				.filter((version) => version.name !== '__synthetic')
				.toSorted((a, b) => Date.parse(b.date_published) - Date.parse(a.date_published)),
		),
		versionsQuery,
		versions,
		reportsQuery,
		reports,
		decisions,
		pendingReports,
		completedReports,
		unflaggedReports,
		contentCounts,
		refreshReports,
	}
}
