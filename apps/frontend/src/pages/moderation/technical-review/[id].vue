<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ArrowLeftIcon, LoaderCircleIcon } from '@modrinth/assets'
import { ButtonStyled, injectModrinthClient } from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'

import MaliciousSummaryModal, {
	type UnsafeFile,
} from '~/components/ui/moderation/MaliciousSummaryModal.vue'
import ModerationTechRevCard from '~/components/ui/moderation/ModerationTechRevCard.vue'

const route = useRoute()
const router = useRouter()
const client = injectModrinthClient()
const queryClient = useQueryClient()

const projectId = computed(() => route.params.id as string)

useHead({ title: () => `Tech review - ${projectId.value} - Modrinth` })

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

function clearExpiredCache(): void {
	try {
		const now = Date.now()
		const keys = Object.keys(localStorage)

		for (const key of keys) {
			if (key.startsWith(CACHE_KEY_PREFIX)) {
				const cached = localStorage.getItem(key)
				if (cached) {
					const data: CachedSource = JSON.parse(cached)
					if (now - data.timestamp > CACHE_TTL) {
						localStorage.removeItem(key)
					}
				}
			}
		}
	} catch (error) {
		console.error('Failed to clear expired cache:', error)
	}
}

if (import.meta.client) {
	clearExpiredCache()
}

const loadingIssues = ref<Set<string>>(new Set())
const decompiledSources = ref<Map<string, string>>(new Map())

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

const {
	data: projectReportData,
	isLoading: isLoadingReport,
	isError: isReportError,
} = useQuery({
	queryKey: ['tech-review-project-report', projectId],
	queryFn: async () => {
		return await client.labrinth.tech_review_internal.getProjectReport(projectId.value)
	},
	retry: false,
})

const {
	data: projectData,
	isLoading: isLoadingProject,
	isError: isProjectError,
} = useQuery({
	queryKey: ['project', projectId],
	queryFn: async () => {
		return await client.labrinth.projects_v3.get(projectId.value)
	},
	retry: false,
})

const { data: organizationData, isLoading: isLoadingOrg } = useQuery({
	queryKey: ['project-organization', projectId],
	queryFn: async () => {
		return await client.labrinth.projects_v3.getOrganization(projectId.value)
	},
})

const { data: membersData, isLoading: isLoadingMembers } = useQuery({
	queryKey: ['project-members', projectId],
	queryFn: async () => {
		return await client.labrinth.projects_v3.getMembers(projectId.value)
	},
	enabled: computed(() => !organizationData.value && !isLoadingOrg.value),
})

const isLoading = computed(
	() =>
		isLoadingReport.value ||
		isLoadingProject.value ||
		isLoadingOrg.value ||
		(isLoadingMembers.value && !organizationData.value),
)

const hasError = computed(() => isReportError.value || isProjectError.value)

type FlattenedFileReport = Labrinth.TechReview.Internal.FileReport & {
	id: string
	version_id: string
}

const ownership = computed<Labrinth.TechReview.Internal.Ownership | null>(() => {
	if (organizationData.value) {
		return {
			kind: 'organization',
			id: organizationData.value.id,
			name: organizationData.value.name,
			icon_url: organizationData.value.icon_url ?? undefined,
		}
	}

	if (membersData.value) {
		const owner = membersData.value.find((m) => m.is_owner)
		if (owner) {
			return {
				kind: 'user',
				id: owner.user.id,
				name: owner.user.username,
				icon_url: owner.user.avatar_url ?? undefined,
			}
		}
	}

	return null
})

const reviewItem = computed(() => {
	if (!projectReportData.value || !projectData.value || !ownership.value) return null

	const { project_report, thread } = projectReportData.value

	const reports: FlattenedFileReport[] = project_report
		? project_report.versions.flatMap((version) =>
				version.files.map((file) => ({
					...file,
					id: file.report_id,
					version_id: version.version_id,
				})),
			)
		: []

	return {
		project: projectData.value,
		project_owner: ownership.value,
		thread,
		reports,
	}
})

function handleMarkComplete(_projectId: string) {
	queryClient.invalidateQueries({ queryKey: ['tech-reviews'] })
	router.push('/moderation/technical-review')
}

const maliciousSummaryModalRef = ref<InstanceType<typeof MaliciousSummaryModal>>()
const currentUnsafeFiles = ref<UnsafeFile[]>([])

function handleShowMaliciousSummary(unsafeFiles: UnsafeFile[]) {
	currentUnsafeFiles.value = unsafeFiles
	maliciousSummaryModalRef.value?.show()
}

function refetch() {
	queryClient.invalidateQueries({ queryKey: ['tech-review-project-report', projectId.value] })
}
</script>

<template>
	<div class="flex flex-col gap-4">
		<div>
			<ButtonStyled
				><nuxt-link :to="'/moderation/technical-review'">
					<ArrowLeftIcon class="size-5" />
					Back to queue
				</nuxt-link></ButtonStyled
			>
		</div>

		<div v-if="isLoading" class="flex flex-col gap-4">
			<div class="universal-card flex h-48 items-center justify-center">
				<LoaderCircleIcon class="size-8 animate-spin text-secondary" />
			</div>
		</div>
		<div
			v-else-if="hasError"
			class="universal-card flex h-24 items-center justify-center text-secondary"
		>
			Project not found in the tech review queue.
		</div>
		<div
			v-else-if="!reviewItem"
			class="universal-card flex h-24 items-center justify-center text-secondary"
		>
			No review data available for this project.
		</div>
		<ModerationTechRevCard
			v-else
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
