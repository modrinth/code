<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { injectModrinthClient } from '@modrinth/ui'
import { ref } from 'vue'

import type { UnsafeFile } from '~/components/ui/moderation/MaliciousSummaryModal.vue'
import MaliciousSummaryModal from '~/components/ui/moderation/MaliciousSummaryModal.vue'
import ModerationTechRevCard from '~/components/ui/moderation/ModerationTechRevCard.vue'

const client = injectModrinthClient()
const { params } = useRoute()
const router = useRouter()
const projectId = params.id as string

const CACHE_TTL = 24 * 60 * 60 * 1000
const CACHE_KEY_PREFIX = 'tech_review_source_'

type CachedSource = {
	source: string
	timestamp: number
}

function getCachedSource(detailId: string): string | null {
	try {
		const cached = localStorage.getItem(`${CACHE_KEY_PREFIX}${detailId}`)
		if (!cached) return null

		const data: CachedSource = JSON.parse(cached)
		const now = Date.now()

		if (now - data.timestamp > CACHE_TTL) {
			localStorage.removeItem(`${CACHE_KEY_PREFIX}${detailId}`)
			return null
		}

		return data.source
	} catch {
		return null
	}
}

function setCachedSource(detailId: string, source: string): void {
	try {
		const data: CachedSource = {
			source,
			timestamp: Date.now(),
		}
		localStorage.setItem(`${CACHE_KEY_PREFIX}${detailId}`, JSON.stringify(data))
	} catch (error) {
		console.error('Failed to cache source:', error)
	}
}

const loadingIssues = ref<Set<string>>(new Set())
const decompiledSources = ref<Map<string, string>>(new Map())

type FlattenedFileReport = Labrinth.TechReview.Internal.FileReport & {
	id: string
	version_id: string
}

const { data: reviewItem, refresh: refetch } = await useAsyncData(
	`tech-review-${projectId}`,
	async () => {
		try {
			const response = await client.labrinth.tech_review_internal.getProject(projectId)
			const projectReport = response.project_reports[0]
			if (!projectReport) {
				throw createError({ statusCode: 404, statusMessage: 'Tech review not found' })
			}

			const project = response.projects[projectReport.project_id]
			const thread = project?.thread_id ? response.threads[project.thread_id] : null
			if (!thread) {
				throw createError({ statusCode: 404, statusMessage: 'Tech review not found' })
			}

			const reports: FlattenedFileReport[] = projectReport.versions.flatMap((version) =>
				version.files.map((file) => ({
					...file,
					id: file.report_id,
					version_id: version.version_id,
				})),
			)

			return {
				project,
				project_owner: response.ownership[projectReport.project_id],
				thread,
				reports,
			}
		} catch (error) {
			console.error('Error fetching tech review:', error)
			throw createError({
				statusCode: 404,
				statusMessage: 'Tech review not found',
			})
		}
	},
)

async function loadIssueSource(issueId: string): Promise<void> {
	if (loadingIssues.value.has(issueId)) return

	loadingIssues.value.add(issueId)

	try {
		const issueData = await client.labrinth.tech_review_internal.getIssue(issueId)

		for (const detail of issueData.details) {
			if (detail.decompiled_source) {
				decompiledSources.value.set(detail.id, detail.decompiled_source)
				setCachedSource(detail.id, detail.decompiled_source)
			}
		}
	} catch (error) {
		console.error('Failed to load issue source:', error)
	} finally {
		loadingIssues.value.delete(issueId)
	}
}

function tryLoadCachedSourcesForFile(reportId: string): void {
	if (!reviewItem.value) return
	const report = reviewItem.value.reports.find((r) => r.id === reportId)
	if (report) {
		for (const issue of report.issues) {
			for (const detail of issue.details) {
				if (!decompiledSources.value.has(detail.id)) {
					const cached = getCachedSource(detail.id)
					if (cached) {
						decompiledSources.value.set(detail.id, cached)
					}
				}
			}
		}
	}
}

function handleLoadFileSources(reportId: string): void {
	tryLoadCachedSourcesForFile(reportId)

	if (!reviewItem.value) return
	const report = reviewItem.value.reports.find((r) => r.id === reportId)
	if (report) {
		for (const issue of report.issues) {
			const hasUncached = issue.details.some((d) => !decompiledSources.value.has(d.id))
			if (hasUncached) {
				loadIssueSource(issue.id)
			}
		}
	}
}

function handleMarkComplete(_projectId: string) {
	router.push('/moderation/technical-review')
}

const maliciousSummaryModalRef = ref<InstanceType<typeof MaliciousSummaryModal>>()
const currentUnsafeFiles = ref<UnsafeFile[]>([])

function handleShowMaliciousSummary(unsafeFiles: UnsafeFile[]) {
	currentUnsafeFiles.value = unsafeFiles
	maliciousSummaryModalRef.value?.show()
}
</script>

<template>
	<div class="flex flex-col gap-3">
		<ModerationTechRevCard
			v-if="reviewItem"
			:item="reviewItem"
			:loading-issues="loadingIssues"
			:decompiled-sources="decompiledSources"
			@refetch="refetch"
			@load-file-sources="handleLoadFileSources"
			@mark-complete="handleMarkComplete"
			@show-malicious-summary="handleShowMaliciousSummary"
		/>

		<MaliciousSummaryModal ref="maliciousSummaryModalRef" :unsafe-files="currentUnsafeFiles" />
	</div>
</template>
