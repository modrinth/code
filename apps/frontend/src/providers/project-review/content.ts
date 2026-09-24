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
	const versionsQuery = useQuery(
		computed(() => ({
			...projectQueryOptions.versionsV3(projectId.value, client),
			enabled: !!projectId.value,
		})),
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
