<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { BlendIcon, ListFilterIcon, SortAscIcon, SortDescIcon, SpinnerIcon } from '@modrinth/assets'
import {
	Combobox,
	type ComboboxOption,
	commonMessages,
	injectModrinthClient,
	Pagination,
	TeleportPopoutMenu,
	Toggle,
	useFormatNumber,
	useVIntl,
} from '@modrinth/ui'
import { useInfiniteQuery, useQueryClient } from '@tanstack/vue-query'
import Fuse from 'fuse.js'
import { nextTick, reactive } from 'vue'

import MaliciousSummaryModal, {
	type UnsafeFile,
} from '~/components/ui/moderation/MaliciousSummaryModal.vue'
import ModerationQueueSkeleton from '~/components/ui/moderation/ModerationQueueSkeleton.vue'
import ModerationQueueToolbar from '~/components/ui/moderation/ModerationQueueToolbar.vue'
import ModerationTechRevCard from '~/components/ui/moderation/ModerationTechRevCard.vue'
import { flattenFileReports } from '~/components/ui/moderation/tech-review/helpers'
import { useTechReviewSources } from '~/components/ui/moderation/tech-review/use-tech-review-sources'

useHead({ title: 'Tech review queue - Modrinth' })

const client = injectModrinthClient()
const queryClient = useQueryClient()
const keybinds = useModerationKeybinds()

const API_PAGE_SIZE = 50
const UI_PAGE_SIZE = 4
const { formatMessage } = useVIntl()
const formatNumber = useFormatNumber()
const route = useRoute()
const router = useRouter()

const SORT_VALUES = ['Severity highest', 'Severity lowest', 'Oldest', 'Newest'] as const
const sortTypes: ComboboxOption<string>[] = [
	{ value: 'Severity highest', label: 'Severity highest' },
	{ value: 'Severity lowest', label: 'Severity lowest' },
	{ value: 'Oldest', label: 'Oldest' },
	{ value: 'Newest', label: 'Newest' },
]

const RESPONSE_FILTER_VALUES = ['All', 'Unread', 'Read'] as const
const responseFilterTypes: ComboboxOption<string>[] = [
	{ value: 'All', label: 'All' },
	{ value: 'Unread', label: 'Unread' },
	{ value: 'Read', label: 'Read' },
]

const PROJECT_TYPE_FILTERS = [
	{ value: 'All project types', name: 'All project types' },
	{ value: 'Modpacks', name: 'Modpacks' },
	{ value: 'Mods', name: 'Mods' },
	{ value: 'Resource Packs', name: 'Resource Packs' },
	{ value: 'Data Packs', name: 'Data Packs' },
	{ value: 'Plugins', name: 'Plugins' },
	{ value: 'Shaders', name: 'Shaders' },
	{ value: 'Servers', name: 'Servers' },
] as const
const PROJECT_TYPE_VALUES = PROJECT_TYPE_FILTERS.map((filter) => filter.value)

function parseAllowed<T extends string>(value: unknown, allowed: readonly T[], fallback: T): T {
	const parsed = queryAsStringOrEmpty((value as string | string[] | null | undefined) ?? '')
	return (allowed as readonly string[]).includes(parsed) ? (parsed as T) : fallback
}

function parsePage(value: unknown): number {
	const page = Number.parseInt(
		queryAsStringOrEmpty((value as string | string[] | null | undefined) ?? ''),
		10,
	)
	return Number.isInteger(page) && page > 0 ? page : 1
}

function parseBoolean(value: unknown, fallback: boolean): boolean {
	const parsed = queryAsStringOrEmpty(
		(value as string | string[] | null | undefined) ?? '',
	).toLowerCase()
	if (parsed === 'true' || parsed === '1') return true
	if (parsed === 'false' || parsed === '0') return false
	return fallback
}

function serializeRouteQuery(query: typeof route.query): string {
	const keys = Object.keys(query).sort()
	return JSON.stringify(
		Object.fromEntries(
			keys.flatMap((key) => {
				const value = query[key]
				if (value == null || value === '') return []
				return [[key, Array.isArray(value) ? value.map(String) : String(value)]]
			}),
		),
	)
}

const query = ref(queryAsStringOrEmpty(route.query.q ?? ''))
const currentFilterType = ref(queryAsStringOrEmpty(route.query.flags ?? '') || 'All flags')
const currentSortType = ref(parseAllowed(route.query.sort, SORT_VALUES, 'Severity highest'))
const currentResponseFilter = ref(parseAllowed(route.query.response, RESPONSE_FILTER_VALUES, 'All'))
const currentProjectTypeFilter = ref(
	parseAllowed(route.query.projectType, PROJECT_TYPE_VALUES, 'All project types'),
)
const inOtherQueueFilter = ref(parseBoolean(route.query.underReview, true))
const currentPage = ref(parsePage(route.query.page))

let syncingFromRoute = false

function writeFiltersToRoute() {
	if (syncingFromRoute) return

	const nextQuery = { ...route.query }

	if (query.value) nextQuery.q = query.value
	else delete nextQuery.q

	if (currentSortType.value !== 'Severity highest') nextQuery.sort = currentSortType.value
	else delete nextQuery.sort

	if (currentResponseFilter.value !== 'All') nextQuery.response = currentResponseFilter.value
	else delete nextQuery.response

	if (currentFilterType.value !== 'All flags') nextQuery.flags = currentFilterType.value
	else delete nextQuery.flags

	if (currentProjectTypeFilter.value !== 'All project types') {
		nextQuery.projectType = currentProjectTypeFilter.value
	} else {
		delete nextQuery.projectType
	}

	if (!inOtherQueueFilter.value) nextQuery.underReview = 'false'
	else delete nextQuery.underReview

	if (currentPage.value > 1) nextQuery.page = String(currentPage.value)
	else delete nextQuery.page

	if (serializeRouteQuery(route.query) === serializeRouteQuery(nextQuery)) return

	router.replace({
		path: route.path,
		query: nextQuery,
	})
}

function readFiltersFromRoute() {
	syncingFromRoute = true

	const nextQuery = queryAsStringOrEmpty(route.query.q ?? '')
	if (query.value !== nextQuery) query.value = nextQuery

	const nextFlags = queryAsStringOrEmpty(route.query.flags ?? '') || 'All flags'
	if (currentFilterType.value !== nextFlags) currentFilterType.value = nextFlags

	const nextSort = parseAllowed(route.query.sort, SORT_VALUES, 'Severity highest')
	if (currentSortType.value !== nextSort) currentSortType.value = nextSort

	const nextResponse = parseAllowed(route.query.response, RESPONSE_FILTER_VALUES, 'All')
	if (currentResponseFilter.value !== nextResponse) currentResponseFilter.value = nextResponse

	const nextProjectType = parseAllowed(
		route.query.projectType,
		PROJECT_TYPE_VALUES,
		'All project types',
	)
	if (currentProjectTypeFilter.value !== nextProjectType) {
		currentProjectTypeFilter.value = nextProjectType
	}

	const nextUnderReview = parseBoolean(route.query.underReview, true)
	if (inOtherQueueFilter.value !== nextUnderReview) inOtherQueueFilter.value = nextUnderReview

	const nextPage = parsePage(route.query.page)
	if (currentPage.value !== nextPage) currentPage.value = nextPage

	nextTick(() => {
		syncingFromRoute = false
	})
}

watch(
	[
		query,
		currentFilterType,
		currentSortType,
		currentResponseFilter,
		currentProjectTypeFilter,
		inOtherQueueFilter,
		currentPage,
	],
	writeFiltersToRoute,
)

watch(() => route.query, readFiltersFromRoute, { deep: true })

const filterTypes = computed<ComboboxOption<string>[]>(() => {
	const issues =
		reviewItems.value?.flatMap((review) => review.reports.flatMap((report) => report.issues)) ?? []
	const counts = new Map<string, number>()
	for (const issue of issues) {
		counts.set(issue.issue_type, (counts.get(issue.issue_type) ?? 0) + 1)
	}

	const options: ComboboxOption<string>[] = [
		{
			value: 'All flags',
			label: isLoading.value ? 'All flags' : `All flags (${formatNumber(issues.length)})`,
		},
	]
	for (const type of Array.from(counts.keys()).sort()) {
		options.push({
			value: type,
			label: isLoading.value ? type : `${type} (${formatNumber(counts.get(type) ?? 0)})`,
		})
	}
	return options
})

const projectTypeFilterTypes = computed<ComboboxOption<string>[]>(() => {
	const items = reviewItems.value ?? []
	const showCounts = !isLoading.value && currentProjectTypeFilter.value === 'All project types'

	return PROJECT_TYPE_FILTERS.map((filter) => {
		if (!showCounts) {
			return { value: filter.value, label: filter.name }
		}

		const apiType = toApiProjectType(filter.value)
		const count =
			filter.value === 'All project types'
				? items.length
				: items.filter((item) => apiType && item.project.project_types.includes(apiType)).length

		return { value: filter.value, label: `${filter.name} (${formatNumber(count)})` }
	})
})

const techReviewQueryKey = computed(
	() =>
		[
			'tech-reviews',
			currentSortType.value,
			currentResponseFilter.value,
			inOtherQueueFilter.value,
			currentFilterType.value,
			currentProjectTypeFilter.value,
		] as const,
)

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

const filteredItems = computed(() => {
	if (!reviewItems.value) return []
	return query.value && searchResults.value ? searchResults.value : [...reviewItems.value]
})

const filteredIssuesCount = computed(() => {
	return filteredItems.value.reduce((total, review) => {
		return total + review.reports.reduce((sum, report) => sum + report.issues.length, 0)
	}, 0)
})

const totalPages = computed(() => Math.ceil((filteredItems.value?.length || 0) / UI_PAGE_SIZE))
const paginatedItems = computed(() => {
	if (!filteredItems.value) return []
	const start = (currentPage.value - 1) * UI_PAGE_SIZE
	const end = start + UI_PAGE_SIZE
	return filteredItems.value.slice(start, end)
})
const pageStart = computed(() =>
	filteredItems.value.length === 0 ? 0 : (currentPage.value - 1) * UI_PAGE_SIZE + 1,
)
const pageEnd = computed(() =>
	Math.min(
		(currentPage.value - 1) * UI_PAGE_SIZE + paginatedItems.value.length,
		filteredItems.value.length,
	),
)
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
		case 'Severity highest':
			return 'severity_desc'
		case 'Severity lowest':
			return 'severity_asc'
		default:
			return 'severity_desc'
	}
}

function toApiProjectType(label: string): string | undefined {
	switch (label) {
		case 'Modpacks':
			return 'modpack'
		case 'Mods':
			return 'mod'
		case 'Resource Packs':
			return 'resourcepack'
		case 'Data Packs':
			return 'datapack'
		case 'Plugins':
			return 'plugin'
		case 'Shaders':
			return 'shader'
		case 'Servers':
			return 'minecraft_java_server'
		default:
			return undefined
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
	enabled: true,
	queryKey: techReviewQueryKey,
	queryFn: async ({ pageParam = 0 }) => {
		const filter: Labrinth.TechReview.Internal.SearchProjectsFilter = {
			project_type: [],
			replied_to: undefined,
			project_status: [],
			issue_type: [],
		}

		if (currentResponseFilter.value === 'Unread') {
			filter.replied_to = 'unreplied'
		} else if (currentResponseFilter.value === 'Read') {
			filter.replied_to = 'replied'
		}

		if (inOtherQueueFilter.value) {
			filter.project_status = ['processing']
		}

		if (currentFilterType.value !== 'All flags') {
			filter.issue_type = [currentFilterType.value]
		}

		const projectType = toApiProjectType(currentProjectTypeFilter.value)
		if (projectType) {
			filter.project_type = [projectType]
		}

		return await client.labrinth.tech_review_internal.searchProjects({
			limit: API_PAGE_SIZE,
			page: pageParam,
			sort_by: toApiSort(currentSortType.value),
			filter,
		})
	},
	getNextPageParam: (lastPage, allPages) => {
		// full page = maybe more
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

const reviewItems = computed(() => {
	if (!mergedSearchResponse.value?.project_reports?.length) {
		return []
	}

	const response = mergedSearchResponse.value

	return response.project_reports.flatMap((projectReport) => {
		const project = response.projects[projectReport.project_id]
		const thread = project?.thread_id ? response.threads[project.thread_id] : undefined
		if (!thread) return []

		return [
			{
				project,
				project_owner: response.ownership[projectReport.project_id],
				thread,
				reports: flattenFileReports(projectReport.versions),
			},
		]
	})
})

const { loadingIssues, decompiledSources, handleLoadIssueSources } = useTechReviewSources(() =>
	reviewItems.value.flatMap((review) => review.reports.flatMap((report) => report.issues)),
)

function handleMarkComplete(projectId: string) {
	const currentIndex = paginatedItems.value.findIndex((item) => item.project.id === projectId)
	const threadId = reviewItems.value.find((item) => item.project.id === projectId)?.thread?.id

	queryClient.setQueryData(
		techReviewQueryKey.value,
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
					// leave this as-is so getNextPageParam still sees a full page
					project_reports: page.project_reports,
					projects: Object.fromEntries(
						Object.entries(page.projects).filter(([id]) => id !== projectId),
					),
					threads: Object.fromEntries(
						Object.entries(page.threads).filter(([id]) => id !== threadId),
					),
					ownership: Object.fromEntries(
						Object.entries(page.ownership).filter(([id]) => id !== projectId),
					),
				})),
			}
		},
	)

	queryClient.invalidateQueries({
		queryKey: ['tech-reviews'],
		refetchType: 'none',
	})

	nextTick(() => {
		const nextItem = paginatedItems.value[currentIndex]
		if (nextItem) {
			cardRefs.get(nextItem.project.id)?.scrollIntoView({
				behavior: 'smooth',
				block: 'start',
			})
		}
	})
}

const maliciousSummaryModalRef = ref<InstanceType<typeof MaliciousSummaryModal>>()
const currentUnsafeFiles = ref<UnsafeFile[]>([])
const cardRefs = reactive<Map<string, HTMLElement>>(new Map())

function handleShowMaliciousSummary(unsafeFiles: UnsafeFile[]) {
	currentUnsafeFiles.value = unsafeFiles
	maliciousSummaryModalRef.value?.show()
}

watch(
	[
		currentSortType,
		currentResponseFilter,
		inOtherQueueFilter,
		currentFilterType,
		currentProjectTypeFilter,
	],
	() => {
		if (syncingFromRoute) return
		goToPage(1)
	},
)

watch(totalPages, (pages) => {
	if (isLoading.value) return

	if (pages === 0) {
		if (currentPage.value !== 1) goToPage(1)
		return
	}

	if (currentPage.value > pages) {
		goToPage(pages)
	}
})

// TODO: Reimpl when backend is available
// const batchScanProgressInformation = computed<BatchScanProgress | undefined>(() => {
// 	return {
// 		total: 58,
// 		complete: 20,
// 	}
// })

const CARD_BOTTOM_OFFSET = 210

function handleKeybinds(event: KeyboardEvent) {
	keybinds.value.handle(event, {
		scope: 'tech-review',
		actions: {
			goToTop: () => {
				Array.from(cardRefs.values())
					.filter((card) => card.getBoundingClientRect().top <= 0)
					.reduce((prev, curr) =>
						curr.getBoundingClientRect().top > prev.getBoundingClientRect().top ? curr : prev,
					)
					?.scrollIntoView({ behavior: 'smooth', block: 'start' })
			},
			goToBottom: () => {
				const nearestCard = Array.from(cardRefs.values())
					.filter((card) => card.getBoundingClientRect().bottom >= window.innerHeight)
					.reduce((prev, curr) =>
						curr.getBoundingClientRect().top < prev.getBoundingClientRect().top ? curr : prev,
					)

				if (nearestCard) {
					window.scrollTo({
						behavior: 'smooth',
						top:
							nearestCard.getBoundingClientRect().bottom +
							window.scrollY -
							window.innerHeight +
							CARD_BOTTOM_OFFSET,
					})
				}
			},
		},
	})
}

onMounted(() => {
	window.addEventListener('keydown', handleKeybinds)
})

onUnmounted(() => {
	window.removeEventListener('keydown', handleKeybinds)
})
</script>

<template>
	<div class="flex flex-col gap-4">
		<!-- TODO: Reimpl when backend is available -->
		<!-- <BatchScanProgressAlert
			v-if="batchScanProgressInformation"
			:progress="batchScanProgressInformation"
		/> -->

		<ModerationQueueToolbar
			v-model="query"
			:page="currentPage"
			:total-pages="totalPages"
			@search="goToPage(1)"
			@switch-page="goToPage"
		>
			<template #actions>
				<Combobox
					v-model="currentResponseFilter"
					class="!w-full flex-grow sm:!w-[120px] sm:flex-grow-0"
					:options="responseFilterTypes"
					trigger-type="base"
					trigger-size="lg"
				>
					<template #selected>
						<span class="flex flex-row gap-2 align-middle font-semibold">
							<ListFilterIcon class="size-5 flex-shrink-0 text-secondary" />
							<span class="truncate text-contrast">{{ currentResponseFilter }}</span>
						</span>
					</template>
				</Combobox>

				<Combobox
					v-model="currentSortType"
					class="!w-full flex-grow sm:!w-[215px] sm:flex-grow-0"
					:options="sortTypes"
					:placeholder="formatMessage(commonMessages.sortByLabel)"
					trigger-type="base"
					trigger-size="lg"
				>
					<template #selected>
						<span class="flex flex-row gap-2 align-middle font-semibold">
							<SortAscIcon
								v-if="currentSortType === 'Oldest' || currentSortType === 'Severity lowest'"
								class="size-5 flex-shrink-0 text-secondary"
							/>
							<SortDescIcon v-else class="size-5 flex-shrink-0 text-secondary" />
							<span class="truncate text-contrast">{{ currentSortType }}</span>
						</span>
					</template>
				</Combobox>

				<TeleportPopoutMenu label="Advanced filters" size="lg">
					<template #trigger>
						<BlendIcon aria-hidden="true" />
						Advanced filters
					</template>
					<template #panel>
						<div class="flex min-w-64 flex-col gap-3">
							<label class="flex cursor-pointer items-center justify-between gap-2 text-sm">
								<span class="whitespace-nowrap font-semibold">Only under review</span>
								<Toggle v-model="inOtherQueueFilter" />
							</label>
							<div class="flex flex-col gap-2">
								<span class="flex items-center gap-1.5 text-sm font-semibold text-secondary">
									Flag type
									<SpinnerIcon v-if="isLoading" class="size-3.5 animate-spin" aria-hidden="true" />
									<template v-else>({{ formatNumber(filteredIssuesCount) }})</template>
								</span>
								<Combobox
									v-model="currentFilterType"
									class="!w-full"
									dropdown-class="!z-[10000]"
									:options="filterTypes"
									:placeholder="formatMessage(commonMessages.filterByLabel)"
									searchable
								>
									<template #selected>
										<span class="flex flex-row gap-2 align-middle font-semibold">
											<ListFilterIcon class="size-5 flex-shrink-0 text-secondary" />
											<span class="truncate text-contrast">{{ currentFilterType }}</span>
										</span>
									</template>
								</Combobox>
							</div>
							<div class="flex flex-col gap-2">
								<span class="text-sm font-semibold text-secondary">Project type</span>
								<Combobox
									v-model="currentProjectTypeFilter"
									class="!w-full"
									dropdown-class="!z-[10000]"
									:options="projectTypeFilterTypes"
									:placeholder="formatMessage(commonMessages.filterByLabel)"
									searchable
								>
									<template #selected>
										<span class="flex flex-row gap-2 align-middle font-semibold">
											<ListFilterIcon class="size-5 flex-shrink-0 text-secondary" />
											<span class="truncate text-contrast">{{ currentProjectTypeFilter }}</span>
										</span>
									</template>
								</Combobox>
							</div>
						</div>
					</template>
				</TeleportPopoutMenu>
			</template>
			<template #meta>
				<div v-if="filteredItems.length > 0" class="flex items-center gap-2">
					<SpinnerIcon
						v-if="isFetchingNextPage"
						v-tooltip="`Pages are still being fetched...`"
						aria-hidden="true"
						class="size-4 animate-spin"
					/>
					Showing {{ formatNumber(pageStart) }}–{{ formatNumber(pageEnd) }} of
					{{ formatNumber(filteredItems.length) }} projects
				</div>
			</template>
		</ModerationQueueToolbar>

		<ModerationQueueSkeleton v-if="isLoading" />
		<div
			v-else-if="paginatedItems.length === 0"
			class="universal-card flex h-24 items-center justify-center text-secondary"
		>
			No projects in queue.
		</div>
		<div v-else class="flex flex-col gap-4">
			<div
				v-for="item in paginatedItems"
				:key="item.project.id"
				:ref="
					(el) => {
						if (el) {
							cardRefs.set(item.project.id, el as HTMLElement)
						} else {
							cardRefs.delete(item.project.id)
						}
					}
				"
			>
				<ModerationTechRevCard
					:item="item"
					:loading-issues="loadingIssues"
					:decompiled-sources="decompiledSources"
					:collapsed="true"
					@refetch="refetch"
					@load-issue-sources="handleLoadIssueSources"
					@mark-complete="handleMarkComplete"
					@show-malicious-summary="handleShowMaliciousSummary"
				/>
			</div>
		</div>

		<div v-if="totalPages > 1" class="flex justify-end">
			<Pagination
				:page="currentPage"
				:count="totalPages"
				@switch-page="(num) => goToPage(num, true)"
			/>
		</div>

		<MaliciousSummaryModal ref="maliciousSummaryModalRef" :unsafe-files="currentUnsafeFiles" />
	</div>
</template>
