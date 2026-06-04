<template>
	<div class="flex flex-col gap-4">
		<div class="flex flex-col justify-between gap-3 lg:flex-row">
			<StyledInput
				v-model="query"
				:icon="SearchIcon"
				type="text"
				autocomplete="off"
				:placeholder="formatMessage(commonMessages.searchPlaceholder)"
				clearable
				wrapper-class="flex-1 lg:max-w-52"
				input-class="h-[40px]"
				@input="goToPage(1)"
			/>

			<div v-if="totalPages > 1" class="hidden flex-1 justify-center lg:flex">
				<Pagination :page="currentPage" :count="totalPages" @switch-page="goToPage" />
			</div>

			<div
				class="flex flex-col items-stretch justify-end gap-2 sm:flex-row sm:items-center lg:flex-shrink-0"
			>
				<Combobox
					v-model="currentMessageFilter"
					class="!w-full flex-grow sm:!w-[200px] sm:flex-grow-0"
					:options="messageFilterTypes"
					:placeholder="formatMessage(commonMessages.filterByLabel)"
					@select="goToPage(1)"
				>
					<template #selected="{ label: messageLabel }">
						<span class="flex flex-row gap-2 align-middle font-semibold">
							<ListFilterIcon class="size-5 flex-shrink-0 text-secondary" />
							<span class="truncate text-contrast"
								>{{ messageLabel }} ({{ sortedReports.length }})</span
							>
						</span>
					</template>
				</Combobox>

				<Combobox
					v-model="currentSortTypeSorting"
					class="!w-full flex-grow sm:!w-[150px] sm:flex-grow-0 lg:!w-[150px]"
					:options="sortTypes"
					:placeholder="formatMessage(commonMessages.sortByLabel)"
					@select="goToPage(1)"
				>
					<template #selected="{ label: sortingLabel }">
						<span class="flex flex-row gap-2 align-middle font-semibold">
							<SortAscIcon
								v-if="currentSortTypeSorting === 'oldest'"
								class="size-5 flex-shrink-0 text-secondary"
							/>
							<SortDescIcon v-else class="size-5 flex-shrink-0 text-secondary" />
							<span class="truncate text-contrast">{{ sortingLabel }}</span>
						</span>
					</template>
				</Combobox>

				<FloatingPanel button-class="!h-10 !shadow-none !text-contrast" :auto-focus="false">
					<BlendIcon class="size-5" /> Advanced filters
					<template #panel>
						<div class="flex min-w-64 flex-col gap-3">
							<div class="flex flex-col gap-2">
								<span class="text-sm font-semibold text-secondary">Report target</span>
								<Combobox
									v-model="currentReportTargetFilter"
									class="!w-full"
									:options="reportTargetFilterTypes"
									:placeholder="formatMessage(commonMessages.filterByLabel)"
								/>
							</div>
							<div class="flex min-w-64 flex-col gap-3">
								<div class="flex flex-col gap-2">
									<span class="text-sm font-semibold text-secondary">Issue type</span>
									<Combobox
										v-model="currentReportIssueFilter"
										class="!w-full"
										:options="reportIssueFilterTypes"
										:placeholder="formatMessage(commonMessages.filterByLabel)"
									/>
								</div>
							</div>
							<div class="flex flex-col gap-2">
								<span class="text-sm font-semibold text-secondary">Project type</span>
								<Combobox
									v-model="currentProjectTypeFilter"
									class="!w-full"
									:options="projectTypeFilterTypes"
									:placeholder="formatMessage(commonMessages.filterByLabel)"
								/>
							</div>
						</div>
					</template>
				</FloatingPanel>
			</div>
		</div>

		<div v-if="totalPages > 1" class="flex justify-center lg:hidden">
			<Pagination :page="currentPage" :count="totalPages" @switch-page="goToPage" />
		</div>

		<div class="flex flex-col gap-4">
			<div v-if="paginatedReports.length === 0" class="universal-card h-24 animate-pulse"></div>
			<ReportCard v-for="report in paginatedReports" v-else :key="report.id" :report="report" />
		</div>

		<div v-if="totalPages > 1" class="mt-4 flex justify-center">
			<Pagination :page="currentPage" :count="totalPages" @switch-page="goToPage" />
		</div>
	</div>
</template>

<script setup lang="ts">
import { BlendIcon, ListFilterIcon, SearchIcon, SortAscIcon, SortDescIcon } from '@modrinth/assets'
import type { ExtendedReport } from '@modrinth/moderation'
import {
	Combobox,
	type ComboboxOption,
	commonMessages,
	FloatingPanel,
	Pagination,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import type { Report } from '@modrinth/utils'
import Fuse from 'fuse.js'

import ReportCard from '~/components/ui/moderation/ModerationReportCard.vue'
import { enrichReportBatch } from '~/helpers/moderation.ts'

useHead({ title: 'Reports queue - Modrinth' })

const { formatMessage } = useVIntl()
const route = useRoute()
const router = useRouter()
const auth = await useAuth()

const { data: allReports } = await useLazyAsyncData('new-moderation-reports', async () => {
	const startTime = performance.now()
	let currentOffset = 0
	const REPORT_ENDPOINT_COUNT = 350
	const allReports: ExtendedReport[] = []

	const enrichmentPromises: Promise<ExtendedReport[]>[] = []

	let reports: Report[]
	let hasMoreReports = true
	while (hasMoreReports) {
		reports = (await useBaseFetch(`report?count=${REPORT_ENDPOINT_COUNT}&offset=${currentOffset}`, {
			apiVersion: 3,
		})) as Report[]

		hasMoreReports = reports.length > 0
		if (!hasMoreReports) {
			break
		}

		const enrichmentPromise = enrichReportBatch(reports)
		enrichmentPromises.push(enrichmentPromise)

		currentOffset += reports.length

		if (enrichmentPromises.length >= 3) {
			const completed = await Promise.all(enrichmentPromises.splice(0, 2))
			allReports.push(...completed.flat())
		}
	}

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

const currentSortTypeSorting = ref('oldest')
const sortTypes: ComboboxOption<string>[] = [
	{ value: 'oldest', label: 'Oldest' },
	{ value: 'newest', label: 'Newest' },
]

const currentMessageFilter = ref('all')
const messageFilterTypes: ComboboxOption<string>[] = [
	{ value: 'all', label: 'All' },
	{ value: 'unread', label: 'Unread' },
	{ value: 'read', label: 'Read' },
	{ value: 'involved', label: 'Involved' },
]

const currentProjectTypeFilter = ref('all')
const projectTypeFilterTypes: ComboboxOption<string>[] = [
	{ value: 'all', label: 'All project types' },
	{ value: 'modpack', label: 'Modpacks' },
	{ value: 'mod', label: 'Mods' },
	{ value: 'resourcepack', label: 'Resource Packs' },
	{ value: 'datapack', label: 'Data Packs' },
	{ value: 'plugin', label: 'Plugins' },
	{ value: 'shader', label: 'Shaders' },
	{ value: 'minecraft_java_server', label: 'Servers' },
]

const currentReportTargetFilter = ref('all')
const reportTargetFilterTypes: ComboboxOption<string>[] = [
	{ value: 'all', label: 'All' },
	{ value: 'project', label: 'Projects' },
	{ value: 'user', label: 'Users' },
	{ value: 'version', label: 'Versions' },
]

const currentReportIssueFilter = ref('all')
const reportIssueFilterTypes = computed<ComboboxOption<string>[]>(() => {
	const base: ComboboxOption<string>[] = [{ value: 'all', label: 'All' }]
	if (!allReports.value) return base

	const issueTypes = new Set(allReports.value.map((report) => report.report_type))

	const sortedTypes = Array.from(issueTypes).sort()
	return [...base, ...sortedTypes.map((type) => ({ value: type, label: type }))]
})

const currentPage = ref(1)
const itemsPerPage = 15
const totalPages = computed(() => Math.ceil((sortedReports.value?.length || 0) / itemsPerPage))

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

const filteredReports = computed(() => {
	const messageFilter = currentMessageFilter.value
	const projectTypeFilter = currentProjectTypeFilter.value
	const reportTargetFilter = currentReportTargetFilter.value
	const reportIssueFilter = currentReportIssueFilter.value

	if (
		messageFilter === 'all' &&
		projectTypeFilter === 'all' &&
		reportTargetFilter === 'all' &&
		reportIssueFilter === 'all'
	) {
		return baseFiltered.value
	}

	const messageFilterPredicate = (report: ExtendedReport) => {
		const messages = report.thread?.messages || []

		if (messageFilter === 'all') return true
		if (messages.length === 0) return messageFilter === 'Unread'
		if (!messages[messages.length - 1].author_id) return false

		if (messageFilter === 'involved') {
			const userId = (auth.value.user as any)?.id
			return userId && messages.some((message) => message.author_id === userId)
		}

		const roleMap = memberRoleMap.value.get(report.id)
		if (!roleMap) return false

		const authorRole = roleMap.get(messages[messages.length - 1].author_id)
		const isModeratorMessage = authorRole === 'moderator' || authorRole === 'admin'

		return messageFilter === 'Read' ? isModeratorMessage : !isModeratorMessage
	}

	const projectTypeFilterPredicate = (report: ExtendedReport) => {
		return projectTypeFilter === 'all' || report.project?.project_type === projectTypeFilter
	}

	const reportTargetFilterPredicate = (report: ExtendedReport) => {
		return reportTargetFilter === 'all' || report.item_type === reportTargetFilter
	}

	const reportIssueFilterPredicate = (report: ExtendedReport) => {
		return reportIssueFilter === 'all' || report.report_type === reportIssueFilter
	}

	return baseFiltered.value.filter((report) => {
		return (
			messageFilterPredicate(report) &&
			projectTypeFilterPredicate(report) &&
			reportTargetFilterPredicate(report) &&
			reportIssueFilterPredicate(report)
		)
	})
})

const sortedReports = computed(() => {
	const filtered = [...filteredReports.value]

	if (currentSortTypeSorting.value === 'oldest') {
		filtered.sort((a, b) => new Date(a.created).getTime() - new Date(b.created).getTime())
	} else {
		filtered.sort((a, b) => new Date(b.created).getTime() - new Date(a.created).getTime())
	}

	return filtered
})

const paginatedReports = computed(() => {
	if (!sortedReports.value) return []
	const start = (currentPage.value - 1) * itemsPerPage
	const end = start + itemsPerPage
	return sortedReports.value.slice(start, end)
})

function goToPage(page: number) {
	currentPage.value = page
}
</script>
