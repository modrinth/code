<template>
	<div class="flex flex-col gap-4">
		<ModerationQueueToolbar
			v-model="query"
			:page="currentPage"
			:total-pages="totalPages"
			@search="goToPage(1)"
			@switch-page="goToPage"
		>
			<template #actions>
				<Combobox
					v-model="currentMessageFilter"
					class="!w-full flex-grow sm:!w-[200px] sm:flex-grow-0"
					:options="messageFilterTypes"
					:placeholder="formatMessage(commonMessages.filterByLabel)"
					trigger-type="base"
					trigger-size="lg"
					@select="goToPage(1)"
				>
					<template #selected>
						<span class="flex flex-row gap-2 align-middle font-semibold">
							<ListFilterIcon class="size-5 flex-shrink-0 text-secondary" />
							<ModerationFilterCount
								:label="currentMessageFilterName"
								:count="sortedReports.length"
								:loading="isLoading"
							/>
						</span>
					</template>
				</Combobox>

				<Combobox
					v-model="currentSortTypeSorting"
					class="!w-full flex-grow sm:!w-[150px] sm:flex-grow-0 lg:!w-[150px]"
					:options="sortTypes"
					:placeholder="formatMessage(commonMessages.sortByLabel)"
					trigger-type="base"
					trigger-size="lg"
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

				<MultiSelect
					v-model="currentReporterOrProject"
					:options="reporterOrProjectOptions"
					:max-height="500"
					dropdown-min-width="360px"
					no-options-message="no options found"
					:searchable="reporterOrProjectOptions.length > 6"
					:max-tag-rows="1"
					fit-content
					trigger-type="base"
					trigger-size="lg"
					checkbox-position="right"
					show-selection-actions
					should-show-select-all
					@update:model-value="goToPage(1)"
				>
					<template #input-content="{ isOpen, openDirection }">
						<div class="flex min-h-7 min-w-0 max-w-full flex-1 items-center gap-1.5 pr-1">
							<LayersIcon class="size-5 shrink-0 text-primary" />
							<span class="min-w-0 flex-1 truncate px-0.5 font-semibold text-inherit">
								{{
									currentReporterOrProject.length === 0
										? 'All reports'
										: `${currentReporterOrProject.length} selected`
								}}
							</span>
							<ChevronLeftIcon
								class="size-5 shrink-0 text-primary transition-transform duration-150"
								:class="
									isOpen ? (openDirection === 'down' ? 'rotate-90' : '-rotate-90') : '-rotate-90'
								"
							/>
						</div>
					</template>
					<template #top>
						<div>
							<button
								type="button"
								class="flex w-full cursor-pointer items-center gap-1.5 border-0 bg-surface-4 px-4 py-3 text-left shadow-none transition-all duration-150 hover:brightness-[115%] focus:brightness-[115%]"
								:aria-selected="currentReporterOrProject.length === 0"
								:class="currentReporterOrProject.length === 0 ? 'text-contrast' : 'text-primary'"
								role="option"
								@click="
									() => {
										currentReporterOrProject = []
										goToPage(1)
									}
								"
								@keydown.enter.stop
								@keydown.space.stop
							>
								<LayersIcon
									class="h-5 w-5 shrink-0 text-primary"
									:class="currentReporterOrProject.length === 0 ? 'text-contrast' : 'text-primary'"
								/>
								<span class="min-w-0 flex-1 font-semibold leading-tight">All reports</span>
								<span class="flex shrink-0 items-center justify-center text-brand">
									<CheckIcon
										v-if="currentReporterOrProject.length === 0"
										aria-hidden="true"
										class="size-5"
									/>
								</span>
							</button>
						</div>
					</template>
				</MultiSelect>

				<TeleportPopoutMenu label="Advanced filters" size="lg" :auto-focus="false">
					<template #trigger>
						<BlendIcon aria-hidden="true" />
						Advanced filters
					</template>
					<template #panel>
						<div class="flex min-w-64 flex-col gap-3">
							<div class="flex flex-col gap-2">
								<span class="text-sm font-semibold text-secondary">Report target</span>
								<Combobox
									v-model="currentReportTargetFilter"
									class="!w-full"
									dropdown-class="!z-[10000]"
									:options="reportTargetFilterTypes"
									:placeholder="formatMessage(commonMessages.filterByLabel)"
									@select="goToPage(1)"
								/>
							</div>
							<div class="flex min-w-64 flex-col gap-3">
								<div class="flex flex-col gap-2">
									<span class="text-sm font-semibold text-secondary">Issue type</span>
									<Combobox
										v-model="currentReportIssueFilter"
										class="!w-full"
										dropdown-class="!z-[10000]"
										:options="reportIssueFilterTypes"
										:placeholder="formatMessage(commonMessages.filterByLabel)"
										@select="goToPage(1)"
									/>
								</div>
							</div>
							<div class="flex flex-col gap-2">
								<span class="text-sm font-semibold text-secondary">Project type</span>
								<Combobox
									v-model="currentProjectTypeFilter"
									class="!w-full"
									dropdown-class="!z-[10000]"
									:options="projectTypeFilterTypes"
									:placeholder="formatMessage(commonMessages.filterByLabel)"
									@select="goToPage(1)"
								/>
							</div>
						</div>
					</template>
				</TeleportPopoutMenu>
			</template>
			<template #meta>
				<div v-if="sortedReports.length > 0">
					Showing {{ formatNumber(pageStart) }}–{{ formatNumber(pageEnd) }} of
					{{ formatNumber(sortedReports.length) }} reports
				</div>
			</template>
		</ModerationQueueToolbar>

		<ModerationQueueSkeleton v-if="isLoading" />
		<div
			v-else-if="paginatedReports.length === 0"
			class="universal-card flex h-24 items-center justify-center text-secondary"
		>
			No reports in queue.
		</div>
		<div v-else class="flex flex-col gap-4 overflow-x-clip">
			<ReportCard
				v-for="report in paginatedReports"
				:key="report.id"
				:report="report"
				:collapsed="true"
				dismiss-after-close
				@dismiss="dismissReport(report.id)"
			/>
		</div>

		<div v-if="totalPages > 1" class="flex justify-end">
			<Pagination :page="currentPage" :count="totalPages" @switch-page="goToPage" />
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	BlendIcon,
	CheckIcon,
	ChevronLeftIcon,
	LayersIcon,
	ListFilterIcon,
	SortAscIcon,
	SortDescIcon,
} from '@modrinth/assets'
import type { ExtendedReport } from '@modrinth/moderation'
import {
	Combobox,
	type ComboboxOption,
	commonMessages,
	formatReportType,
	injectModrinthClient,
	MultiSelect,
	type MultiSelectItem,
	Pagination,
	TeleportPopoutMenu,
	useDebugLogger,
	useFormatNumber,
	useVIntl,
} from '@modrinth/ui'
import Fuse from 'fuse.js'

import ModerationFilterCount from '~/components/ui/moderation/ModerationFilterCount.vue'
import ModerationQueueSkeleton from '~/components/ui/moderation/ModerationQueueSkeleton.vue'
import ModerationQueueToolbar from '~/components/ui/moderation/ModerationQueueToolbar.vue'
import ReportCard from '~/components/ui/moderation/ModerationReportCard.vue'
import { enrichReportBatch } from '~/helpers/moderation.ts'

useHead({ title: 'Reports queue - Modrinth' })

const { formatMessage } = useVIntl()
const formatNumber = useFormatNumber()
const route = useRoute()
const router = useRouter()
const auth = await useAuth()
const client = injectModrinthClient()
const debug = useDebugLogger('ModerationReports')

const { data: allReports, pending: reportsPending } = await useLazyAsyncData(
	'new-moderation-reports',
	async () => {
		const startTime = performance.now()
		let currentOffset = 0
		const REPORT_ENDPOINT_COUNT = 350
		const allReports: ExtendedReport[] = []

		const enrichmentPromises: Promise<ExtendedReport[]>[] = []

		let reports: Labrinth.Reports.v3.Report[]
		let hasMoreReports = true
		while (hasMoreReports) {
			reports = (await useBaseFetch(
				`report?count=${REPORT_ENDPOINT_COUNT}&offset=${currentOffset}&all=true`,
				{
					apiVersion: 3,
				},
			)) as Labrinth.Reports.v3.Report[]

			hasMoreReports = reports.length > 0
			if (!hasMoreReports) {
				break
			}

			const enrichmentPromise = enrichReportBatch(reports, client)
			enrichmentPromises.push(enrichmentPromise)

			// this is explicitly not the length of the reports array, because the API may return fewer reports due to a report in the middle not being
			// serializable if the offset is set to the reports array you can get the same report from the end multiple times.
			currentOffset += REPORT_ENDPOINT_COUNT

			if (enrichmentPromises.length >= 3) {
				const completed = await Promise.all(enrichmentPromises.splice(0, 2))
				allReports.push(...completed.flat())
			}
		}

		const remainingBatches = await Promise.all(enrichmentPromises)
		allReports.push(...remainingBatches.flat())

		const endTime = performance.now()
		const duration = endTime - startTime

		debug(
			`Reports fetched and processed in ${duration.toFixed(2)}ms (${(duration / 1000).toFixed(2)}s)`,
		)

		return allReports
	},
)

const isLoading = computed(() => reportsPending.value || allReports.value == null)

const SORT_VALUES = ['oldest', 'newest'] as const
const sortTypes: ComboboxOption<string>[] = [
	{ value: 'oldest', label: 'Oldest' },
	{ value: 'newest', label: 'Newest' },
]

const MESSAGE_FILTERS = [
	{ value: 'all', name: 'All' },
	{ value: 'unread', name: 'Unread' },
	{ value: 'read', name: 'Read' },
	{ value: 'involved', name: 'Involved' },
] as const
const MESSAGE_FILTER_VALUES = MESSAGE_FILTERS.map((filter) => filter.value)

const PROJECT_TYPE_FILTERS = [
	{ value: 'all', name: 'All project types' },
	{ value: 'modpack', name: 'Modpacks' },
	{ value: 'mod', name: 'Mods' },
	{ value: 'resourcepack', name: 'Resource Packs' },
	{ value: 'datapack', name: 'Data Packs' },
	{ value: 'plugin', name: 'Plugins' },
	{ value: 'shader', name: 'Shaders' },
	{ value: 'minecraft_java_server', name: 'Servers' },
	{ value: 'shared-instance', name: 'Shared instance' },
] as const
const PROJECT_TYPE_VALUES = PROJECT_TYPE_FILTERS.map((filter) => filter.value)

const REPORT_TARGET_FILTERS = [
	{ value: 'all', name: 'All' },
	{ value: 'project', name: 'Projects' },
	{ value: 'user', name: 'Users' },
	{ value: 'version', name: 'Versions' },
	{ value: 'shared-instance', name: 'Shared instances' },
] as const
const REPORT_TARGET_VALUES = REPORT_TARGET_FILTERS.map((filter) => filter.value)

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

function selectedValuesEqual(left: string[], right: string[]): boolean {
	if (left.length !== right.length) return false
	return left.every((value, index) => value === right[index])
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
const currentSortTypeSorting = ref(parseAllowed(route.query.sort, SORT_VALUES, 'oldest'))
const currentMessageFilter = ref(parseAllowed(route.query.messages, MESSAGE_FILTER_VALUES, 'all'))
const currentMessageFilterName = computed(
	() =>
		MESSAGE_FILTERS.find((filter) => filter.value === currentMessageFilter.value)?.name ?? 'All',
)
const currentProjectTypeFilter = ref(
	parseAllowed(route.query.projectType, PROJECT_TYPE_VALUES, 'all'),
)
const currentReportTargetFilter = ref(parseAllowed(route.query.target, REPORT_TARGET_VALUES, 'all'))
const currentReportIssueFilter = ref(queryAsStringOrEmpty(route.query.issue ?? '') || 'all')
const currentReporterOrProject = ref(queryAsStringArray(route.query.selected))
const currentPage = ref(parsePage(route.query.page))

function writeFiltersToRoute() {
	const nextQuery = { ...route.query }

	if (query.value) nextQuery.q = query.value
	else delete nextQuery.q

	if (currentSortTypeSorting.value !== 'oldest') nextQuery.sort = currentSortTypeSorting.value
	else delete nextQuery.sort

	if (currentMessageFilter.value !== 'all') nextQuery.messages = currentMessageFilter.value
	else delete nextQuery.messages

	if (currentReportTargetFilter.value !== 'all') nextQuery.target = currentReportTargetFilter.value
	else delete nextQuery.target

	if (currentReportIssueFilter.value !== 'all') nextQuery.issue = currentReportIssueFilter.value
	else delete nextQuery.issue

	if (currentProjectTypeFilter.value !== 'all') {
		nextQuery.projectType = currentProjectTypeFilter.value
	} else {
		delete nextQuery.projectType
	}

	if (currentReporterOrProject.value.length === 1) {
		nextQuery.selected = currentReporterOrProject.value[0]
	} else if (currentReporterOrProject.value.length > 1) {
		nextQuery.selected = currentReporterOrProject.value
	} else {
		delete nextQuery.selected
	}

	if (currentPage.value > 1) nextQuery.page = String(currentPage.value)
	else delete nextQuery.page

	if (serializeRouteQuery(route.query) === serializeRouteQuery(nextQuery)) return

	router.replace({
		path: route.path,
		query: nextQuery,
	})
}

function readFiltersFromRoute() {
	const nextQuery = queryAsStringOrEmpty(route.query.q ?? '')
	if (query.value !== nextQuery) query.value = nextQuery

	const nextSort = parseAllowed(route.query.sort, SORT_VALUES, 'oldest')
	if (currentSortTypeSorting.value !== nextSort) currentSortTypeSorting.value = nextSort

	const nextMessages = parseAllowed(route.query.messages, MESSAGE_FILTER_VALUES, 'all')
	if (currentMessageFilter.value !== nextMessages) currentMessageFilter.value = nextMessages

	const nextProjectType = parseAllowed(route.query.projectType, PROJECT_TYPE_VALUES, 'all')
	if (currentProjectTypeFilter.value !== nextProjectType) {
		currentProjectTypeFilter.value = nextProjectType
	}

	const nextTarget = parseAllowed(route.query.target, REPORT_TARGET_VALUES, 'all')
	if (currentReportTargetFilter.value !== nextTarget) currentReportTargetFilter.value = nextTarget

	const nextIssue = queryAsStringOrEmpty(route.query.issue ?? '') || 'all'
	if (currentReportIssueFilter.value !== nextIssue) currentReportIssueFilter.value = nextIssue

	const nextSelected = queryAsStringArray(route.query.selected)
	if (!selectedValuesEqual(currentReporterOrProject.value, nextSelected)) {
		currentReporterOrProject.value = nextSelected
	}

	const nextPage = parsePage(route.query.page)
	if (currentPage.value !== nextPage) currentPage.value = nextPage
}

watch(
	[
		query,
		currentSortTypeSorting,
		currentMessageFilter,
		currentProjectTypeFilter,
		currentReportTargetFilter,
		currentReportIssueFilter,
		currentReporterOrProject,
		currentPage,
	],
	writeFiltersToRoute,
	{ deep: true },
)

watch(() => route.query, readFiltersFromRoute, { deep: true })

type ReportedType<T> = T & { report_item_count: number }
const reporterOrProjectOptions = computed<MultiSelectItem<string>[]>(() => {
	if (!allReports.value) return []
	const options: MultiSelectItem<string>[] = []

	const uniqueProjectIds: { [id: string]: ReportedType<Labrinth.Projects.v2.Project> } = {}
	const uniqueReporterIds: { [id: string]: ReportedType<User> } = {}

	for (const report of filteredReports.value) {
		if (report.project)
			uniqueProjectIds[report.project.id] = {
				...report.project,
				report_item_count: (uniqueProjectIds[report.project.id]?.report_item_count || 0) + 1,
			}
		if (report.reporter_user)
			uniqueReporterIds[report.reporter_user.id] = {
				...report.reporter_user,
				report_item_count: (uniqueReporterIds[report.reporter_user.id]?.report_item_count || 0) + 1,
			}
	}

	if (Object.keys(uniqueProjectIds).length !== 0) {
		options.push({ type: 'section-header', label: 'Projects' })
		Object.values(uniqueProjectIds)
			.sort((a, b) =>
				a.report_item_count === b.report_item_count
					? a.title.localeCompare(b.title)
					: b.report_item_count - a.report_item_count,
			)
			.forEach((project) => {
				options.push({
					value: `project/${project.id}`,
					label: `${project.title} (${formatNumber(project.report_item_count)})`,
					icon: project.icon_url ? h('img', { src: project.icon_url }) : undefined,
				})
			})
	}

	options.push({ type: 'section-header', label: 'Reporters' })
	Object.values(uniqueReporterIds)
		.sort((a, b) =>
			a.report_item_count === b.report_item_count
				? a.username.localeCompare(b.username)
				: b.report_item_count - a.report_item_count,
		)
		.forEach((reporter) => {
			options.push({
				value: `reporter/${reporter.id}`,
				label: `${reporter.username} (${formatNumber(reporter.report_item_count)})`,
				icon: reporter.avatar_url ? h('img', { src: reporter.avatar_url }) : undefined,
			})
		})

	return options
})

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
	return baseFiltered.value.filter((report) => {
		return (
			matchesMessageFilter(report, currentMessageFilter.value) &&
			matchesProjectTypeFilter(report, currentProjectTypeFilter.value) &&
			matchesReportTargetFilter(report, currentReportTargetFilter.value) &&
			matchesReportIssueFilter(report, currentReportIssueFilter.value)
		)
	})
})

function matchesMessageFilter(
	report: ExtendedReport,
	messageFilter: (typeof MESSAGE_FILTERS)[number]['value'] | string,
): boolean {
	if (messageFilter === 'all') return true

	const messages = report.thread?.messages || []
	if (messages.length === 0) return messageFilter === 'unread'
	if (!messages[messages.length - 1].author_id) return false

	if (messageFilter === 'involved') {
		const userId = (auth.value.user as any)?.id
		return !!userId && messages.some((message) => message.author_id === userId)
	}

	const roleMap = memberRoleMap.value.get(report.id)
	if (!roleMap) return false

	const authorRole = roleMap.get(messages[messages.length - 1].author_id)
	const isModeratorMessage = authorRole === 'moderator' || authorRole === 'admin'

	return messageFilter === 'read' ? isModeratorMessage : !isModeratorMessage
}

function matchesProjectTypeFilter(
	report: ExtendedReport,
	projectTypeFilter: (typeof PROJECT_TYPE_FILTERS)[number]['value'] | string,
): boolean {
	if (projectTypeFilter === 'all') return true
	if (projectTypeFilter === 'shared-instance') return report.item_type === 'shared-instance'
	return report.project?.project_type === projectTypeFilter
}

function matchesReportTargetFilter(
	report: ExtendedReport,
	reportTargetFilter: (typeof REPORT_TARGET_FILTERS)[number]['value'] | string,
): boolean {
	return reportTargetFilter === 'all' || report.item_type === reportTargetFilter
}

function matchesReportIssueFilter(report: ExtendedReport, reportIssueFilter: string): boolean {
	return reportIssueFilter === 'all' || report.report_type === reportIssueFilter
}

function labelWithCount(name: string, count: number): string {
	return `${name} (${formatNumber(count)})`
}

const reportsForMessageCounts = computed(() =>
	baseFiltered.value.filter(
		(report) =>
			matchesProjectTypeFilter(report, currentProjectTypeFilter.value) &&
			matchesReportTargetFilter(report, currentReportTargetFilter.value) &&
			matchesReportIssueFilter(report, currentReportIssueFilter.value),
	),
)
const reportsForProjectTypeCounts = computed(() =>
	baseFiltered.value.filter(
		(report) =>
			matchesMessageFilter(report, currentMessageFilter.value) &&
			matchesReportTargetFilter(report, currentReportTargetFilter.value) &&
			matchesReportIssueFilter(report, currentReportIssueFilter.value),
	),
)
const reportsForTargetCounts = computed(() =>
	baseFiltered.value.filter(
		(report) =>
			matchesMessageFilter(report, currentMessageFilter.value) &&
			matchesProjectTypeFilter(report, currentProjectTypeFilter.value) &&
			matchesReportIssueFilter(report, currentReportIssueFilter.value),
	),
)
const reportsForIssueCounts = computed(() =>
	baseFiltered.value.filter(
		(report) =>
			matchesMessageFilter(report, currentMessageFilter.value) &&
			matchesProjectTypeFilter(report, currentProjectTypeFilter.value) &&
			matchesReportTargetFilter(report, currentReportTargetFilter.value),
	),
)

const messageFilterTypes = computed<ComboboxOption<string>[]>(() =>
	MESSAGE_FILTERS.map((filter) => ({
		value: filter.value,
		label: isLoading.value
			? filter.name
			: labelWithCount(
					filter.name,
					reportsForMessageCounts.value.filter((report) =>
						matchesMessageFilter(report, filter.value),
					).length,
				),
	})),
)

const projectTypeFilterTypes = computed<ComboboxOption<string>[]>(() =>
	PROJECT_TYPE_FILTERS.map((filter) => ({
		value: filter.value,
		label: isLoading.value
			? filter.name
			: labelWithCount(
					filter.name,
					reportsForProjectTypeCounts.value.filter((report) =>
						matchesProjectTypeFilter(report, filter.value),
					).length,
				),
	})),
)

const reportTargetFilterTypes = computed<ComboboxOption<string>[]>(() =>
	REPORT_TARGET_FILTERS.map((filter) => ({
		value: filter.value,
		label: isLoading.value
			? filter.name
			: labelWithCount(
					filter.name,
					reportsForTargetCounts.value.filter((report) =>
						matchesReportTargetFilter(report, filter.value),
					).length,
				),
	})),
)

const reportIssueFilterTypes = computed<ComboboxOption<string>[]>(() => {
	const issueTypes = new Set((allReports.value ?? []).map((report) => report.report_type))
	const options = [
		{ value: 'all', name: 'All' },
		...Array.from(issueTypes)
			.sort()
			.map((type) => ({
				value: type,
				name: formatReportType(formatMessage, type),
			})),
	]

	return options.map((filter) => ({
		value: filter.value,
		label: isLoading.value
			? filter.name
			: labelWithCount(
					filter.name,
					reportsForIssueCounts.value.filter((report) =>
						matchesReportIssueFilter(report, filter.value),
					).length,
				),
	}))
})

const sortedReports = computed(() => {
	const reporterOrProjectFilter = currentReporterOrProject.value
	const filtered =
		reporterOrProjectFilter.length === 0
			? [...filteredReports.value]
			: filteredReports.value.filter((report) => {
					const reporterOrProjectFilterLookup = new Set(reporterOrProjectFilter)
					const reporterValue = report.reporter_user ? `reporter/${report.reporter_user.id}` : null
					const projectValue = report.project ? `project/${report.project.id}` : null
					return (
						(reporterValue && reporterOrProjectFilterLookup.has(reporterValue)) ||
						(projectValue && reporterOrProjectFilterLookup.has(projectValue))
					)
				})

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

const pageStart = computed(() =>
	sortedReports.value.length === 0 ? 0 : itemsPerPage * (currentPage.value - 1) + 1,
)
const pageEnd = computed(
	() =>
		itemsPerPage * (currentPage.value - 1) + Math.min(itemsPerPage, paginatedReports.value.length),
)

function goToPage(page: number) {
	currentPage.value = page
}

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

function dismissReport(reportId: string) {
	if (!allReports.value) return

	allReports.value = allReports.value.filter((report) => report.id !== reportId)
	if (currentPage.value > totalPages.value) {
		currentPage.value = Math.max(1, totalPages.value)
	}
}
</script>
