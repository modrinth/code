<template>
	<div class="flex flex-col gap-3">
		<div class="flex flex-col justify-between gap-3 lg:flex-row">
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
						<span class="truncate">{{ selected }} ({{ filteredReports.length }})</span>
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

		<div class="mt-4 flex flex-col gap-2">
			<div v-if="paginatedReports.length === 0" class="universal-card h-24 animate-pulse"></div>
			<ReportCard v-for="report in paginatedReports" v-else :key="report.id" :report="report" />
		</div>

		<div v-if="totalPages > 1" class="mt-4 flex justify-center">
			<Pagination :page="currentPage" :count="totalPages" @switch-page="goToPage" />
		</div>
	</div>
</template>

<script setup lang="ts">
import { FilterIcon, SearchIcon, SortAscIcon, SortDescIcon, XIcon } from '@modrinth/assets'
import type { ExtendedReport } from '@modrinth/moderation'
import { Button, DropdownSelect, Pagination } from '@modrinth/ui'
import type { Report } from '@modrinth/utils'
import { defineMessages, useVIntl } from '@vintl/vintl'
import Fuse from 'fuse.js'

import ReportCard from '~/components/ui/moderation/ModerationReportCard.vue'
import { enrichReportBatch } from '~/helpers/moderation.ts'

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

const { data: allReports } = await useLazyAsyncData('new-moderation-reports', async () => {
	const startTime = performance.now()
	let currentOffset = 0
	const REPORT_ENDPOINT_COUNT = 350
	const allReports: ExtendedReport[] = []

	const enrichmentPromises: Promise<ExtendedReport[]>[] = []

	let reports: Report[]
	do {
		reports = (await useBaseFetch(`report?count=${REPORT_ENDPOINT_COUNT}&offset=${currentOffset}`, {
			apiVersion: 3,
		})) as Report[]

		if (reports.length === 0) break

		const enrichmentPromise = enrichReportBatch(reports)
		enrichmentPromises.push(enrichmentPromise)

		currentOffset += reports.length

		if (enrichmentPromises.length >= 3) {
			const completed = await Promise.all(enrichmentPromises.splice(0, 2))
			allReports.push(...completed.flat())
		}
	} while (reports.length === REPORT_ENDPOINT_COUNT)

	const remainingBatches = await Promise.all(enrichmentPromises)
	allReports.push(...remainingBatches.flat())

	const endTime = performance.now()
	const duration = endTime - startTime

	console.debug(
		`Reports fetched and processed in ${duration.toFixed(2)}ms (${(duration / 1000).toFixed(2)}s)`,
	)

	return allReports
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

const currentFilterType = ref('All')
const filterTypes: readonly string[] = readonly(['All', 'Unread', 'Read'])

const currentSortType = ref('Oldest')
const sortTypes: readonly string[] = readonly(['Oldest', 'Newest'])

const currentPage = ref(1)
const itemsPerPage = 15
const totalPages = computed(() => Math.ceil((filteredReports.value?.length || 0) / itemsPerPage))

const fuse = computed(() => {
	if (!allReports.value || allReports.value.length === 0) return null
	return new Fuse(allReports.value, {
		keys: [
			{
				name: 'id',
				weight: 3,
			},
			{
				name: 'body',
				weight: 3,
			},
			{
				name: 'report_type',
				weight: 3,
			},
			{
				name: 'item_id',
				weight: 2,
			},
			{
				name: 'reporter_user.username',
				weight: 2,
			},
			'project.name',
			'project.slug',
			'user.username',
			'version.name',
			'target.name',
			'target.slug',
		],
		includeScore: true,
		threshold: 0.4,
	})
})

const memberRoleMap = computed(() => {
	if (!allReports.value?.length) return new Map()

	const map = new Map()
	for (const report of allReports.value) {
		if (report.thread?.members?.length) {
			const roleMap = new Map()
			for (const member of report.thread.members) {
				roleMap.set(member.id, member.role)
			}
			map.set(report.id, roleMap)
		}
	}
	return map
})

const searchResults = computed(() => {
	if (!query.value || !fuse.value) return null
	return fuse.value.search(query.value).map((result) => result.item)
})

const baseFiltered = computed(() => {
	if (!allReports.value) return []
	return query.value && searchResults.value ? searchResults.value : [...allReports.value]
})

const typeFiltered = computed(() => {
	if (currentFilterType.value === 'All') return baseFiltered.value

	return baseFiltered.value.filter((report) => {
		const messages = report.thread?.messages || []

		if (messages.length === 0) {
			return currentFilterType.value === 'Unread'
		}

		const lastMessage = messages[messages.length - 1]
		if (!lastMessage.author_id) return false

		const roleMap = memberRoleMap.value.get(report.id)
		if (!roleMap) return false

		const authorRole = roleMap.get(lastMessage.author_id)
		const isModeratorMessage = authorRole === 'moderator' || authorRole === 'admin'

		return currentFilterType.value === 'Read' ? isModeratorMessage : !isModeratorMessage
	})
})

const filteredReports = computed(() => {
	const filtered = [...typeFiltered.value]

	if (currentSortType.value === 'Oldest') {
		filtered.sort((a, b) => new Date(a.created).getTime() - new Date(b.created).getTime())
	} else {
		filtered.sort((a, b) => new Date(b.created).getTime() - new Date(a.created).getTime())
	}

	return filtered
})

const paginatedReports = computed(() => {
	if (!filteredReports.value) return []
	const start = (currentPage.value - 1) * itemsPerPage
	const end = start + itemsPerPage
	return filteredReports.value.slice(start, end)
})

function goToPage(page: number) {
	currentPage.value = page
}
</script>
