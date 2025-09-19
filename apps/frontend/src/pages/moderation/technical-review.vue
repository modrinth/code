<script setup lang="ts">
import { FilterIcon, SearchIcon, SortAscIcon, SortDescIcon, XIcon } from '@modrinth/assets'
import { Button, DropdownSelect, Pagination } from '@modrinth/ui'
import { defineMessages, useVIntl } from '@vintl/vintl'
import Fuse from 'fuse.js'
import BatchScanProgressAlert, {
	type BatchScanProgress,
} from '~/components/ui/moderation/BatchScanProgressAlert.vue'
import ModerationTechRevCard from '~/components/ui/moderation/ModerationTechRevCard.vue'
import { fetchDelphiIssues, fetchIssueTypeSchema, type OrderBy } from '~/helpers/tech-review'

// Data from backend helper (with dummy fallback)
type TechReviewItem = Awaited<ReturnType<typeof fetchDelphiIssues>>[number]
const reviewItems = ref<TechReviewItem[]>([])

// Basic pagination state (mirrors moderation pages)
const currentPage = ref(1)
const itemsPerPage = 15
// Search/filter/sort UI state
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
const rawIssueTypes = ref<string[] | null>(null)
const filterTypes = computed<readonly string[]>(() => {
	const base: string[] = ['All issues']
	if (rawIssueTypes.value && rawIssueTypes.value.length) base.push(...rawIssueTypes.value)
	return base
})

const currentSortType = ref('Oldest')
const sortTypes: readonly string[] = readonly([
	'Oldest',
	'Newest',
	'Pending first',
	'Severity ↑',
	'Severity ↓',
])

const fuse = computed(() => {
	if (!reviewItems.value || reviewItems.value.length === 0) return null
	return new Fuse(reviewItems.value, {
		keys: [
			{ name: 'issue.issue_type', weight: 3 },
			{ name: 'report.artifact_url', weight: 2 },
			{ name: 'java_classes.internal_class_name', weight: 2 },
		],
		includeScore: true,
		threshold: 0.4,
	})
})

const searchResults = computed(() => {
	if (!query.value || !fuse.value) return null
	return fuse.value.search(query.value).map((result) => result.item as TechReviewItem)
})

const baseFiltered = computed(() => {
	if (!reviewItems.value) return []
	return query.value && searchResults.value ? searchResults.value : [...reviewItems.value]
})

const typeFiltered = computed(() => {
	if (currentFilterType.value === 'All issues') return baseFiltered.value
	const type = currentFilterType.value
	return baseFiltered.value.filter((it) => it.issue.issue_type === type)
})

const filteredItems = computed(() => {
	const filtered = [...typeFiltered.value]

	switch (currentSortType.value) {
		case 'Oldest':
			filtered.sort(
				(a, b) => new Date(a.report.created).getTime() - new Date(b.report.created).getTime(),
			)
			break
		case 'Newest':
			filtered.sort(
				(a, b) => new Date(b.report.created).getTime() - new Date(a.report.created).getTime(),
			)
			break
		case 'Pending first': {
			const p = (s: string) => (s === 'pending' ? 0 : 1)
			filtered.sort((a, b) => p(a.issue.status) - p(b.issue.status))
			break
		}
		case 'Severity ↑': {
			const order = { LOW: 0, MEDIUM: 1, HIGH: 2, SEVERE: 3 } as Record<string, number>
			filtered.sort((a, b) => (order[a.report.severity] ?? 0) - (order[b.report.severity] ?? 0))
			break
		}
		case 'Severity ↓': {
			const order = { LOW: 0, MEDIUM: 1, HIGH: 2, SEVERE: 3 } as Record<string, number>
			filtered.sort((a, b) => (order[b.report.severity] ?? 0) - (order[a.report.severity] ?? 0))
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

// Map sort label to backend order_by param
function toOrderBy(label: string): OrderBy | null {
	switch (label) {
		case 'Oldest':
			return 'created_asc'
		case 'Newest':
			return 'created_desc'
		case 'Pending first':
			return 'pending_status_first'
		case 'Severity ↑':
			return 'severity_asc'
		case 'Severity ↓':
			return 'severity_desc'
		default:
			return null
	}
}

// Initial fetch and reactive refetch on filter/sort changes
onMounted(async () => {
	rawIssueTypes.value = await fetchIssueTypeSchema()
	const order_by = toOrderBy(currentSortType.value)
	reviewItems.value = await fetchDelphiIssues({ count: 350, offset: 0, order_by })
})

watch(currentFilterType, async (val) => {
	const type = val === 'All issues' ? null : val
	const order_by = toOrderBy(currentSortType.value)
	reviewItems.value = await fetchDelphiIssues({ type, count: 350, offset: 0, order_by })
	goToPage(1)
})

watch(currentSortType, async (val) => {
	const type = currentFilterType.value === 'All issues' ? null : currentFilterType.value
	const order_by = toOrderBy(val)
	// If you prefer server-side sorting only, keep this; otherwise client-side above already reorders
	reviewItems.value = await fetchDelphiIssues({ type, count: 350, offset: 0, order_by })
	goToPage(1)
})

// TODO: Live way to update this via the backend, polling?
const batchScanProgressInformation = computed<BatchScanProgress | undefined>(() => {
	return {
		total: 58,
		complete: 20,
	}
})
</script>

<template>
	<div class="flex flex-col gap-6">
		<BatchScanProgressAlert
			v-if="batchScanProgressInformation"
			:progress="batchScanProgressInformation"
		/>

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
				<DropdownSelect
					v-slot="{ selected }"
					v-model="currentFilterType"
					class="!w-full flex-grow sm:!w-[280px] sm:flex-grow-0 lg:!w-[280px]"
					:name="formatMessage(messages.filterBy)"
					:options="filterTypes as unknown[]"
					@change="goToPage(1)"
				>
					<span class="flex flex-row gap-2 align-middle font-semibold text-secondary">
						<FilterIcon class="size-4 flex-shrink-0" />
						<span class="truncate">{{ selected }} ({{ filteredItems.length }})</span>
					</span>
				</DropdownSelect>

				<DropdownSelect
					v-slot="{ selected }"
					v-model="currentSortType"
					class="!w-full flex-grow sm:!w-[150px] sm:flex-grow-0 lg:!w-[150px]"
					:name="formatMessage(messages.sortBy)"
					:options="sortTypes as unknown[]"
					@change="goToPage(1)"
				>
					<span class="flex flex-row gap-2 align-middle font-semibold text-secondary">
						<SortAscIcon v-if="selected === 'Oldest'" class="size-4 flex-shrink-0" />
						<SortDescIcon v-else class="size-4 flex-shrink-0" />
						<span class="truncate">{{ selected }}</span>
					</span>
				</DropdownSelect>
			</div>
		</div>

		<div v-if="totalPages > 1" class="flex justify-center lg:hidden">
			<Pagination :page="currentPage" :count="totalPages" @switch-page="goToPage" />
		</div>

		<div class="flex flex-col gap-2">
			<div v-if="paginatedItems.length === 0" class="universal-card h-24 animate-pulse"></div>
			<div
				v-else
				v-for="(item, idx) in paginatedItems"
				:key="item.issue.id ?? idx"
				class=""
			>
				<ModerationTechRevCard :item="item" />
			</div>
		</div>

		<!-- Bottom pagination -->
		<div v-if="totalPages > 1" class="mt-4 flex justify-center">
			<Pagination :page="currentPage" :count="totalPages" @switch-page="goToPage" />
		</div>
	</div>
</template>
