<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	FilterIcon,
	SearchIcon,
	ShieldAlertIcon,
	SortAscIcon,
	SortDescIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Button,
	ButtonStyled,
	Combobox,
	type ComboboxOption,
	// injectModrinthClient, // TEMPORARY: Commented out while using mock data
	Pagination,
} from '@modrinth/ui'
// import { useQuery } from '@tanstack/vue-query' // TEMPORARY: Commented out while using mock data
import { defineMessages, useVIntl } from '@vintl/vintl'
import Fuse from 'fuse.js'

import { type BatchScanProgress } from '~/components/ui/moderation/BatchScanProgressAlert.vue'
import ModerationTechRevCard from '~/components/ui/moderation/ModerationTechRevCard.vue'

// TEMPORARY: Mock data for development
import { generateMockProjectReviews } from '~/utils/mockTechReviewData'

// const client = injectModrinthClient() // TEMPORARY: Commented out while using mock data

const currentPage = ref(1)
const itemsPerPage = 15
const { formatMessage } = useVIntl()
const route = useRoute()
const router = useRouter()

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
			.flatMap((report) => report.files)
			.flatMap((file) => file.issues)
			.map((issue) => issue.kind),
	)

	const sortedTypes = Array.from(issueTypes).sort()
	return [...base, ...sortedTypes.map((type) => ({ value: type, label: type }))]
})

const currentSortType = ref('Oldest')
const sortTypes: ComboboxOption<string>[] = [
	{ value: 'Oldest', label: 'Oldest' },
	{ value: 'Newest', label: 'Newest' },
	{ value: 'Pending first', label: 'Pending first' },
	{ value: 'Severity ↑', label: 'Severity ↑' },
	{ value: 'Severity ↓', label: 'Severity ↓' },
]

const fuse = computed(() => {
	if (!reviewItems.value || reviewItems.value.length === 0) return null
	return new Fuse(reviewItems.value, {
		keys: [
			{ name: 'project.title', weight: 4 },
			{ name: 'project.slug', weight: 3 },
			{ name: 'reports.files.file_name', weight: 2 },
			{ name: 'reports.files.issues.kind', weight: 3 },
			{ name: 'project_owner.name', weight: 2 },
		],
		includeScore: true,
		threshold: 0.4,
	})
})

const searchResults = computed(() => {
	if (!query.value || !fuse.value) return null
	return fuse.value
		.search(query.value)
		.map((result) => result.item as Labrinth.TechReview.Internal.ProjectReview)
})

const baseFiltered = computed(() => {
	if (!reviewItems.value) return []
	return query.value && searchResults.value ? searchResults.value : [...reviewItems.value]
})

const typeFiltered = computed(() => {
	if (currentFilterType.value === 'All issues') return baseFiltered.value
	const type = currentFilterType.value

	return baseFiltered.value.filter((review) => {
		return review.reports.some((report) =>
			report.files.some((file) => file.issues.some((issue) => issue.kind === type)),
		)
	})
})

function getHighestSeverity(review: Labrinth.TechReview.Internal.ProjectReview): string {
	const severities = review.reports
		.flatMap((r) => r.files)
		.flatMap((f) => f.issues)
		.flatMap((i) => i.details)
		.map((d) => d.severity)

	const order = { SEVERE: 3, HIGH: 2, MEDIUM: 1, LOW: 0 } as Record<string, number>
	return severities.sort((a, b) => (order[b] ?? 0) - (order[a] ?? 0))[0] || 'LOW'
}

function hasPendingIssues(review: Labrinth.TechReview.Internal.ProjectReview): boolean {
	return review.reports.some((report) =>
		report.files.some((file) => file.issues.some((issue) => issue.status === 'pending')),
	)
}

function getEarliestDate(review: Labrinth.TechReview.Internal.ProjectReview): number {
	const dates = review.reports.map((r) => new Date(r.created_at).getTime())
	return Math.min(...dates)
}

const filteredItems = computed(() => {
	const filtered = [...typeFiltered.value]

	switch (currentSortType.value) {
		case 'Oldest':
			filtered.sort((a, b) => getEarliestDate(a) - getEarliestDate(b))
			break
		case 'Newest':
			filtered.sort((a, b) => getEarliestDate(b) - getEarliestDate(a))
			break
		case 'Pending first': {
			filtered.sort((a, b) => {
				const aPending = hasPendingIssues(a) ? 0 : 1
				const bPending = hasPendingIssues(b) ? 0 : 1
				return aPending - bPending
			})
			break
		}
		case 'Severity ↑': {
			const order = { LOW: 0, MEDIUM: 1, HIGH: 2, SEVERE: 3 } as Record<string, number>
			filtered.sort(
				(a, b) => (order[getHighestSeverity(a)] ?? 0) - (order[getHighestSeverity(b)] ?? 0),
			)
			break
		}
		case 'Severity ↓': {
			const order = { LOW: 0, MEDIUM: 1, HIGH: 2, SEVERE: 3 } as Record<string, number>
			filtered.sort(
				(a, b) => (order[getHighestSeverity(b)] ?? 0) - (order[getHighestSeverity(a)] ?? 0),
			)
			break
		}
	}

	return filtered
})

const totalPages = computed(() => Math.ceil((filteredItems.value?.length || 0) / itemsPerPage))
const paginatedItems = computed(() => {
	if (!filteredItems.value) return []
	const start = (currentPage.value - 1) * itemsPerPage
	const end = start + itemsPerPage
	return filteredItems.value.slice(start, end)
})
function goToPage(page: number) {
	currentPage.value = page
}

// TEMPORARY: Commented out while using mock data
/*
function toApiSort(label: string): Labrinth.TechReview.Internal.SearchProjectsSort {
	switch (label) {
		case 'Oldest':
			return 'CreatedAsc'
		case 'Newest':
		default:
			return 'CreatedDesc'
	}
}
*/

// TEMPORARY: Using mock data instead of API
// Uncomment below to use real API data
/*
const {
	data: reviewItems,
	isLoading,
	refetch,
} = useQuery({
	queryKey: ['tech-reviews', currentSortType],
	queryFn: async () => {
		return await client.labrinth.tech_review_internal.searchProjects({
			limit: 350,
			page: 0,
			sort_by: toApiSort(currentSortType.value),
		})
	},
	initialData: [] as Labrinth.TechReview.Internal.ProjectReview[],
})
*/

// TEMPORARY: Mock data for development (58 items to match batch scan progress)
const reviewItems = ref<Labrinth.TechReview.Internal.ProjectReview[]>(
	generateMockProjectReviews(58),
)
const isLoading = ref(false)
const refetch = () => {
	reviewItems.value = generateMockProjectReviews(58)
}

watch(currentSortType, () => {
	goToPage(1)
	refetch()
})

const batchScanProgressInformation = computed<BatchScanProgress | undefined>(() => {
	return {
		total: 58,
		complete: 20,
	}
})
</script>

<template>
	<div class="flex flex-col gap-6">
		<!-- <BatchScanProgressAlert
			v-if="batchScanProgressInformation"
			:progress="batchScanProgressInformation"
		/> -->

		<div class="flex flex-col justify-between gap-2 lg:flex-row">
			<div class="iconified-input flex-1 lg:max-w-md">
				<SearchIcon aria-hidden="true" class="text-lg" />
				<input
					v-model="query"
					class="h-[40px]"
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
						<span class="flex flex-row gap-2 align-middle font-semibold text-primary">
							<FilterIcon class="size-4 flex-shrink-0" />
							<span class="truncate">{{ currentFilterType }} ({{ filteredItems.length }})</span>
						</span>
					</template>
				</Combobox>

				<Combobox
					v-model="currentSortType"
					class="!w-full flex-grow sm:!w-[150px] sm:flex-grow-0 lg:!w-[150px]"
					:options="sortTypes"
					:placeholder="formatMessage(messages.sortBy)"
					@select="goToPage(1)"
				>
					<template #selected>
						<span class="flex flex-row gap-2 align-middle font-semibold text-primary">
							<SortAscIcon v-if="currentSortType === 'Oldest'" class="size-4 flex-shrink-0" />
							<SortDescIcon v-else class="size-4 flex-shrink-0" />
							<span class="truncate">{{ currentSortType }}</span>
						</span>
					</template>
				</Combobox>

				<ButtonStyled color="orange">
					<button><ShieldAlertIcon /> Batch scan</button>
				</ButtonStyled>
			</div>
		</div>

		<div v-if="totalPages > 1" class="flex justify-center lg:hidden">
			<Pagination :page="currentPage" :count="totalPages" @switch-page="goToPage" />
		</div>

		<div class="flex flex-col gap-4">
			<div
				v-if="isLoading || paginatedItems.length === 0"
				class="universal-card h-24 animate-pulse"
			></div>
			<div v-for="(item, idx) in paginatedItems" v-else :key="item.project.id ?? idx" class="">
				<ModerationTechRevCard :item="item" @refetch="refetch" />
			</div>
		</div>

		<div v-if="totalPages > 1" class="mt-4 flex justify-center">
			<Pagination :page="currentPage" :count="totalPages" @switch-page="goToPage" />
		</div>
	</div>
</template>
