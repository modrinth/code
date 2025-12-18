<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ListFilterIcon, SearchIcon, SortAscIcon, SortDescIcon, XIcon } from '@modrinth/assets'
import {
	Button,
	Combobox,
	type ComboboxOption,
	injectModrinthClient,
	Pagination,
} from '@modrinth/ui'
import { useInfiniteQuery, useQueryClient } from '@tanstack/vue-query'
import { defineMessages, useVIntl } from '@vintl/vintl'
import Fuse from 'fuse.js'

import MaliciousSummaryModal, {
	type UnsafeFile,
} from '~/components/ui/moderation/MaliciousSummaryModal.vue'
import ModerationTechRevCard from '~/components/ui/moderation/ModerationTechRevCard.vue'

const client = injectModrinthClient()
const queryClient = useQueryClient()

const currentPage = ref(1)
const API_PAGE_SIZE = 50
const UI_PAGE_SIZE = 4
const { formatMessage } = useVIntl()
const route = useRoute()
const router = useRouter()

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

clearExpiredCache()

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
	for (const review of reviewItems.value) {
		const report = review.reports.find((r) => r.id === reportId)
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
			return
		}
	}
}

function handleLoadFileSources(reportId: string): void {
	tryLoadCachedSourcesForFile(reportId)

	for (const review of reviewItems.value) {
		const report = review.reports.find((r) => r.id === reportId)
		if (report) {
			for (const issue of report.issues) {
				const hasUncached = issue.details.some((d) => !decompiledSources.value.has(d.id))
				if (hasUncached) {
					loadIssueSource(issue.id)
				}
			}
			return
		}
	}
}

const messages = defineMessages({
	searchPlaceholder: {
		id: 'moderation.search.placeholder',
		defaultMessage: 'Search...',
	},
	filterBy: {
		id: 'moderation.filter.by',
		defaultMessage: 'Filter by',
	},
	sortBy: {
		id: 'moderation.sort.by',
		defaultMessage: 'Sort by',
	},
})

const query = ref(route.query.q?.toString() || '')

watch(
	query,
	(newQuery) => {
		const currentQuery = { ...route.query }
		if (newQuery) {
			currentQuery.q = newQuery
		} else {
			delete currentQuery.q
		}

		router.replace({
			path: route.path,
			query: currentQuery,
		})
		goToPage(1)
	},
	{ immediate: false },
)

watch(
	() => route.query.q,
	(newQueryParam) => {
		const newValue = newQueryParam?.toString() || ''
		if (query.value !== newValue) {
			query.value = newValue
		}
	},
)

const currentFilterType = ref('All issues')

const filterTypes = computed<ComboboxOption<string>[]>(() => {
	const base: ComboboxOption<string>[] = [{ value: 'All issues', label: 'All issues' }]
	if (!reviewItems.value) return base

	const issueTypes = new Set(
		reviewItems.value
			.flatMap((review) => review.reports)
			.flatMap((report) => report.issues)
			.map((issue) => issue.issue_type),
	)

	const sortedTypes = Array.from(issueTypes).sort()
	return [...base, ...sortedTypes.map((type) => ({ value: type, label: type }))]
})

const currentSortType = ref('Severe first')
const sortTypes: ComboboxOption<string>[] = [
	{ value: 'Oldest', label: 'Oldest' },
	{ value: 'Newest', label: 'Newest' },
	{ value: 'Severe first', label: 'Severe first' },
	{ value: 'Severe last', label: 'Severe last' },
]

const fuse = computed(() => {
	if (!reviewItems.value || reviewItems.value.length === 0) return null
	return new Fuse(reviewItems.value, {
		keys: [
			{ name: 'project.title', weight: 4 },
			{ name: 'project.slug', weight: 3 },
			{ name: 'reports.file_name', weight: 2 },
			{ name: 'reports.issues.issue_type', weight: 3 },
			{ name: 'project_owner.name', weight: 2 },
		],
		includeScore: true,
		threshold: 0.4,
	})
})

const searchResults = computed(() => {
	if (!query.value || !fuse.value) return null
	return fuse.value.search(query.value).map((result) => result.item)
})

const baseFiltered = computed(() => {
	if (!reviewItems.value) return []
	return query.value && searchResults.value ? searchResults.value : [...reviewItems.value]
})

const typeFiltered = computed(() => {
	if (currentFilterType.value === 'All issues') return baseFiltered.value
	const type = currentFilterType.value

	return baseFiltered.value.filter((review) => {
		return review.reports.some((report: Labrinth.TechReview.Internal.FileReport) =>
			report.issues.some(
				(issue: Labrinth.TechReview.Internal.FileIssue) => issue.issue_type === type,
			),
		)
	})
})

const filteredItems = computed(() => typeFiltered.value)

const filteredIssuesCount = computed(() => {
	return filteredItems.value.reduce((total, review) => {
		if (currentFilterType.value === 'All issues') {
			return total + review.reports.reduce((sum, report) => sum + report.issues.length, 0)
		} else {
			return (
				total +
				review.reports.reduce((sum, report) => {
					return (
						sum +
						report.issues.filter((issue) => issue.issue_type === currentFilterType.value).length
					)
				}, 0)
			)
		}
	}, 0)
})

const totalPages = computed(() => Math.ceil((filteredItems.value?.length || 0) / UI_PAGE_SIZE))
const paginatedItems = computed(() => {
	if (!filteredItems.value) return []
	const start = (currentPage.value - 1) * UI_PAGE_SIZE
	const end = start + UI_PAGE_SIZE
	return filteredItems.value.slice(start, end)
})
function goToPage(page: number, top = false) {
	currentPage.value = page

	if (top && window) {
		window.scrollTo({
			top: 0,
			left: 0,
			behavior: 'smooth',
		})
	}
}

function toApiSort(label: string): Labrinth.TechReview.Internal.SearchProjectsSort {
	switch (label) {
		case 'Oldest':
			return 'created_asc'
		case 'Newest':
			return 'created_desc'
		case 'Severe first':
			return 'severity_desc'
		case 'Severe last':
			return 'severity_asc'
		default:
			return 'severity_desc'
	}
}

const {
	data: infiniteData,
	isLoading,
	isFetchingNextPage,
	fetchNextPage,
	hasNextPage,
	refetch,
} = useInfiniteQuery({
	queryKey: ['tech-reviews', currentSortType],
	queryFn: async ({ pageParam = 0 }) => {
		return await client.labrinth.tech_review_internal.searchProjects({
			limit: API_PAGE_SIZE,
			page: pageParam,
			sort_by: toApiSort(currentSortType.value),
		})
	},
	getNextPageParam: (lastPage, allPages) => {
		// If we got a full page, there's probably more
		return lastPage.project_reports.length >= API_PAGE_SIZE ? allPages.length : undefined
	},
	initialPageParam: 0,
})

watch(
	[() => infiniteData.value, hasNextPage],
	() => {
		if (hasNextPage.value && !isFetchingNextPage.value) {
			fetchNextPage()
		}
	},
	{ immediate: true },
)

const mergedSearchResponse = computed(() => {
	if (!infiniteData.value?.pages?.length) return null

	return infiniteData.value.pages.reduce(
		(merged, page) => ({
			project_reports: [...merged.project_reports, ...page.project_reports],
			projects: { ...merged.projects, ...page.projects },
			threads: { ...merged.threads, ...page.threads },
			ownership: { ...merged.ownership, ...page.ownership },
		}),
		{
			project_reports: [] as Labrinth.TechReview.Internal.ProjectReport[],
			projects: {} as Record<string, Labrinth.TechReview.Internal.ProjectModerationInfo>,
			threads: {} as Record<string, Labrinth.TechReview.Internal.Thread>,
			ownership: {} as Record<string, Labrinth.TechReview.Internal.Ownership>,
		},
	)
})

type FlattenedFileReport = Labrinth.TechReview.Internal.FileReport & {
	id: string
	version_id: string
}

const reviewItems = computed(() => {
	if (!mergedSearchResponse.value?.project_reports?.length) {
		return []
	}

	const response = mergedSearchResponse.value

	return response.project_reports
		.map((projectReport) => {
			const project = response.projects[projectReport.project_id]
			const thread = project?.thread_id ? response.threads[project.thread_id] : undefined

			if (!thread) return null

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
		})
		.filter(
			(
				item,
			): item is {
				project: Labrinth.TechReview.Internal.ProjectModerationInfo
				project_owner: Labrinth.TechReview.Internal.Ownership
				thread: Labrinth.TechReview.Internal.Thread
				reports: FlattenedFileReport[]
			} => item !== null,
		)
})

function handleMarkComplete(projectId: string) {
	queryClient.setQueryData(
		['tech-reviews', currentSortType],
		(
			oldData:
				| {
						pages: Labrinth.TechReview.Internal.SearchResponse[]
						pageParams: number[]
				  }
				| undefined,
		) => {
			if (!oldData) return oldData

			return {
				...oldData,
				pages: oldData.pages.map((page) => ({
					...page,
					project_reports: page.project_reports.filter((pr) => pr.project_id !== projectId),
					projects: Object.fromEntries(
						Object.entries(page.projects).filter(([id]) => id !== projectId),
					),
					ownership: Object.fromEntries(
						Object.entries(page.ownership).filter(([id]) => id !== projectId),
					),
				})),
			}
		},
	)
}

const maliciousSummaryModalRef = ref<InstanceType<typeof MaliciousSummaryModal>>()
const currentUnsafeFiles = ref<UnsafeFile[]>([])

function handleShowMaliciousSummary(unsafeFiles: UnsafeFile[]) {
	currentUnsafeFiles.value = unsafeFiles
	maliciousSummaryModalRef.value?.show()
}

watch(currentSortType, () => {
	goToPage(1)
})

// TODO: Reimpl when backend is available
// const batchScanProgressInformation = computed<BatchScanProgress | undefined>(() => {
// 	return {
// 		total: 58,
// 		complete: 20,
// 	}
// })
</script>

<template>
	<div class="flex flex-col gap-4">
		<!-- TODO: Reimpl when backend is available -->
		<!-- <BatchScanProgressAlert
			v-if="batchScanProgressInformation"
			:progress="batchScanProgressInformation"
		/> -->

		<div class="flex flex-col justify-between gap-2 lg:flex-row">
			<div class="iconified-input flex-1 lg:max-w-md">
				<SearchIcon aria-hidden="true" class="text-lg" />
				<input
					v-model="query"
					class="!h-10"
					autocomplete="off"
					spellcheck="false"
					type="text"
					:placeholder="formatMessage(messages.searchPlaceholder)"
					@input="goToPage(1)"
				/>
				<Button v-if="query" class="r-btn" @click="() => (query = '')">
					<XIcon />
				</Button>
			</div>

			<div v-if="totalPages > 1" class="hidden flex-1 justify-center lg:flex">
				<Pagination :page="currentPage" :count="totalPages" @switch-page="goToPage" />
			</div>

			<div class="flex flex-col justify-end gap-2 sm:flex-row lg:flex-shrink-0">
				<Combobox
					v-model="currentFilterType"
					class="!w-full flex-grow sm:!w-[280px] sm:flex-grow-0 lg:!w-[280px]"
					:options="filterTypes"
					:placeholder="formatMessage(messages.filterBy)"
					searchable
					@select="goToPage(1)"
				>
					<template #selected>
						<span class="flex flex-row gap-2 align-middle font-semibold">
							<ListFilterIcon class="size-5 flex-shrink-0 text-secondary" />
							<span class="truncate text-contrast"
								>{{ currentFilterType }} ({{ filteredIssuesCount }})</span
							>
						</span>
					</template>
				</Combobox>

				<Combobox
					v-model="currentSortType"
					class="!w-full flex-grow sm:!w-[150px] sm:flex-grow-0 lg:!w-[175px]"
					:options="sortTypes"
					:placeholder="formatMessage(messages.sortBy)"
					@select="goToPage(1)"
				>
					<template #selected>
						<span class="flex flex-row gap-2 align-middle font-semibold">
							<SortAscIcon
								v-if="currentSortType === 'Oldest'"
								class="size-5 flex-shrink-0 text-secondary"
							/>
							<SortDescIcon v-else class="size-5 flex-shrink-0 text-secondary" />
							<span class="truncate text-contrast">{{ currentSortType }}</span>
						</span>
					</template>
				</Combobox>

				<!-- <ButtonStyled color="orange">
					<button class="!h-10"><ShieldAlertIcon /> Batch scan</button>
				</ButtonStyled> -->
			</div>
		</div>

		<div v-if="totalPages > 1" class="flex justify-center lg:hidden">
			<Pagination :page="currentPage" :count="totalPages" @switch-page="goToPage" />
		</div>

		<div class="flex flex-col gap-4">
			<div v-if="isLoading || isFetchingNextPage" class="universal-card h-24 animate-pulse"></div>
			<div
				v-else-if="paginatedItems.length === 0"
				class="universal-card flex h-24 items-center justify-center text-secondary"
			>
				No projects in queue.
			</div>
			<div v-for="(item, idx) in paginatedItems" :key="item.project.id ?? idx">
				<ModerationTechRevCard
					:item="item"
					:loading-issues="loadingIssues"
					:decompiled-sources="decompiledSources"
					@refetch="refetch"
					@load-file-sources="handleLoadFileSources"
					@mark-complete="handleMarkComplete"
					@show-malicious-summary="handleShowMaliciousSummary"
				/>
			</div>
		</div>

		<div v-if="totalPages > 1" class="mt-4 flex justify-center">
			<Pagination
				:page="currentPage"
				:count="totalPages"
				@switch-page="(num) => goToPage(num, true)"
			/>
		</div>

		<MaliciousSummaryModal ref="maliciousSummaryModalRef" :unsafe-files="currentUnsafeFiles" />
	</div>
</template>
