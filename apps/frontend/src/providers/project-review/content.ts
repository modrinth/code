import { injectModrinthClient } from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type Ref, ref, watch } from 'vue'

import {
	flattenFileReports,
	getFileDetailCount,
} from '~/components/project-review/tech-review/helpers'
import { useTechReviewDecisions } from '~/components/project-review/tech-review/use-tech-review-decisions'
import { projectQueryOptions } from '~/composables/queries/project'

import { reviewQueryOptions } from './queries'

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
		computed(() => ({
			...reviewQueryOptions.versions(projectId.value, client, queryClient),
			enabled: !!projectId.value,
		})),
	)
	const reportsQuery = useQuery(
		computed(() => ({
			...reviewQueryOptions.report(projectId.value, client),
			enabled: !!projectId.value,
		})),
	)
	const versions = computed(() =>
		(versionsQuery.data.value ?? [])
			.filter((version) => version.name !== '__synthetic')
			.toSorted((a, b) => Date.parse(b.date_published) - Date.parse(a.date_published)),
	)
	const rawReports = computed(() =>
		flattenFileReports(reportsQuery.data.value?.project_report?.versions ?? []),
	)
	const selectedArchiveId = ref<string>()
	watch(projectId, () => {
		selectedArchiveId.value = undefined
	})
	const decisions = useTechReviewDecisions(rawReports)
	watch(projectId, decisions.resetDecisions, { flush: 'sync' })
	const reports = computed(() => {
		const publishedDates = new Map(
			versions.value.map((version) => [version.id, Date.parse(version.date_published) || 0]),
		)
		const pendingIds = new Set(
			rawReports.value.filter(decisions.isFilePending).map((file) => file.id),
		)
		const unsafeCompletedIds = new Set(
			rawReports.value
				.filter(
					(file) =>
						!pendingIds.has(file.id) &&
						file.issues.some((issue) =>
							issue.details.some(
								(detail) => decisions.getDetailDecision(detail.id, detail.status) === 'malware',
							),
						),
				)
				.map((file) => file.id),
		)
		return rawReports.value.toSorted(
			(a, b) =>
				Number(pendingIds.has(b.id)) - Number(pendingIds.has(a.id)) ||
				Number(unsafeCompletedIds.has(b.id)) - Number(unsafeCompletedIds.has(a.id)) ||
				(publishedDates.get(b.version_id) ?? 0) - (publishedDates.get(a.version_id) ?? 0),
		)
	})
	const pendingReports = computed(() => reports.value.filter(decisions.isFilePending))
	watch(
		[reports, pendingReports],
		([files, pending]) => {
			if (files.some((file) => file.id === selectedArchiveId.value)) return
			selectedArchiveId.value = pending[0]?.id ?? files[0]?.id
		},
		{ immediate: true },
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
		initialLoading: computed(
			() =>
				versionsV2Query.isFetching.value ||
				versionsQuery.isFetching.value ||
				reportsQuery.isFetching.value,
		),
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
		selectedArchiveId,
		decisions,
		pendingReports,
		completedReports,
		unflaggedReports,
		contentCounts,
		refreshReports,
	}
}
