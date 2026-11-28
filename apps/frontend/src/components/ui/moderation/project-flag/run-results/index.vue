<template>
	<slot :unique-flag-key-count="uniqueFlagKeyCount" />
	<section class="flex min-w-0 flex-col gap-3" aria-label="Project results">
		<Tabs
			:tabs="resultTabs"
			:value="category"
			color="gray"
			wrap
			aria-label="Project results filter"
			class="bg-surface-2 [&_[role=tab][aria-selected=true]]:border-brand [&_[role=tab][aria-selected=true]]:bg-brand-highlight [&_[role=tab][aria-selected=true]]:text-brand"
			@update:value="showCategory"
		/>

		<div class="flex flex-wrap items-center gap-2">
			<Input
				v-model="search"
				:icon="SearchIcon"
				type="search"
				input-class="[&::-webkit-search-cancel-button]:hidden"
				aria-label="Search projects"
				placeholder="Search projects by name or ID…"
				wrapper-class="min-w-0 w-full max-w-sm"
				clearable
			/>
			<Button
				v-if="!data && !needsAllResults"
				:disabled="loadFilterOptions && fullResults.isFetching.value"
				@click="loadResultFilters"
			>
				{{
					loadFilterOptions && fullResults.isFetching.value
						? 'Loading rule and flag filters…'
						: 'Load rule and flag filters'
				}}
			</Button>
			<DropdownFilterBar
				v-model="selectedFilters"
				:categories="filterCategories"
				:show-label="false"
				add-label="Filters"
			>
				<template #search-actions="{ category, selectedValues }">
					<div
						v-if="category.key === 'rule' || category.key === 'flagKey'"
						class="mr-2 flex shrink-0 items-center"
					>
						<SegmentedControl
							v-model="matchMode[category.key]"
							:items="matchModes"
							:disabled="selectedValues.length < 2"
							:aria-label="`${category.label} match mode`"
						/>
					</div>
				</template>
			</DropdownFilterBar>
		</div>

		<div class="flex flex-wrap items-center gap-x-2 gap-y-1">
			<p role="status" class="m-0 text-sm text-secondary">{{ resultCountLabel }}</p>
			<Button type="quiet" size="sm" :disabled="!canCopyProjectIds" @click="copyProjectIds">
				<CopyIcon aria-hidden="true" />
				{{ copyingProjectIds ? 'Copying…' : 'Copy IDs' }}
			</Button>
		</div>

		<div class="grid min-w-0 gap-4 lg:grid-cols-[minmax(0,1.1fr)_minmax(0,1fr)]">
			<ProjectList
				:rows="projectRows"
				:selected-id="selectedRow?.project.id"
				:empty-message="emptyMessage"
				:offset="offset"
				:page-size="PAGE_SIZE"
				:category-total="categoryTotal"
				@select="selectProject"
				@change-page="changePage"
			>
				<p v-if="isPending" role="status" class="m-0 flex items-center gap-2">
					<LoaderCircleIcon class="size-4 animate-spin" /> Loading project results…
				</p>
				<div v-if="isError" role="alert" class="universal-card flex flex-col items-start gap-3">
					<Button :disabled="isFetching" @click="refetch()">Try again</Button>
				</div>
				<p
					v-if="projectTypeFilterActive && loadingProjectTypes"
					role="status"
					class="m-0 flex items-center gap-2"
				>
					<LoaderCircleIcon class="size-4 animate-spin" /> Loading project types…
				</p>
				<div
					v-if="projectTypeFilterActive && projectTypeError"
					role="alert"
					class="universal-card flex flex-col items-start gap-3"
				>
					<Button @click="refetchProjectTypes()">Try again</Button>
				</div>
			</ProjectList>

			<ProjectDetail :key="selectedRow?.project.id" :result="selectedRow" />
		</div>
	</section>
</template>

<script setup lang="ts">
import { useFlagErrorNotification } from '~/components/ui/moderation/project-flag/use-flag-error-notification'

import type { ProjectFlagRun, ProjectFlagRunResult } from '@modrinth/api-client'
import { CopyIcon, LoaderCircleIcon, SearchIcon } from '@modrinth/assets'
import {
	Button,
	DropdownFilterBar,
	type DropdownFilterBarCategory,
	injectModrinthClient,
	injectNotificationManager,
	Input,
	PROJECT_TYPE_ICONS,
	SegmentedControl,
	Tabs,
	type TabsValue,
} from '@modrinth/ui'
import { formatProjectType } from '@modrinth/utils'
import { useDebounceFn } from '@vueuse/core'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import Fuse from 'fuse.js'

import {
	projectFlagResultCategory,
	projectFlagRunResultsOptions,
	projectFlagRunResultsPageOptions,
} from '~/composables/queries/project-flag-run'

import { countUniqueFlagKeys } from '../run-format'
import ProjectDetail from './project-detail.vue'
import ProjectList from './project-list.vue'
import { useResultProjects, useResultProjectOwners } from './use-result-projects'

const PAGE_SIZE = 25
const filters = [
	{ label: 'All results', category: 'all' },
	{ label: 'Flagged', category: 'flagged' },
	{ label: 'Approvable', category: 'approvable' },
	{ label: 'Scan errors', category: 'errors' },
] as const
type Category = (typeof filters)[number]['category']

const props = defineProps<{
	run: Pick<ProjectFlagRun, 'id' | 'status'> & Partial<ProjectFlagRun>
	category: Category
	projectId?: string
}>()
const emit = defineEmits<{ changeCategory: [category: Category]; selectProject: [id: string] }>()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()

const search = ref('')
const searchTerm = ref('')
const updateSearch = useDebounceFn((value: string) => {
	searchTerm.value = value.trim()
}, 250)
watch(search, updateSearch)
const loadFilterOptions = ref(false)
const pageOffset = ref(0)
const locateProject = ref(Boolean(props.projectId))
const selectedFilters = ref<Record<string, string[]>>({})
const selectedRules = computed(() => selectedFilters.value.rule ?? [])
const selectedFlagKeys = computed(() => selectedFilters.value.flagKey ?? [])
type MatchMode = 'Match any' | 'Match all'
const matchModes: MatchMode[] = ['Match any', 'Match all']
const matchMode = ref<Record<'rule' | 'flagKey', MatchMode>>({
	rule: 'Match any',
	flagKey: 'Match any',
})
const selectedProjectTypes = computed(() => selectedFilters.value.projectType ?? [])
const projectTypeFilterActive = computed(() => selectedProjectTypes.value.length > 0)
const hasFilters = computed(
	() =>
		selectedRules.value.length > 0 ||
		selectedFlagKeys.value.length > 0 ||
		projectTypeFilterActive.value,
)
const needsAllResults = computed(
	() =>
		props.category !== 'all' ||
		Boolean(searchTerm.value) ||
		hasFilters.value ||
		locateProject.value,
)
// TODO: Use server-side search, filters, and counts when the results API supports them.
const fullResults = useQuery(
	computed(() => ({
		...projectFlagRunResultsOptions(client, props.run),
		enabled: needsAllResults.value || loadFilterOptions.value,
	})),
)
const pagedResults = useQuery(
	computed(() => ({
		...projectFlagRunResultsPageOptions(client, props.run, PAGE_SIZE, pageOffset.value),
		enabled: !needsAllResults.value,
	})),
)
const data = fullResults.data
const resultPage = pagedResults.data
const activeResults = computed(() => (needsAllResults.value ? fullResults : pagedResults))
const isPending = computed(() => activeResults.value.isPending.value)
const isError = computed(() => activeResults.value.isError.value)
const error = computed(() => activeResults.value.error.value)
const isFetching = computed(() => activeResults.value.isFetching.value)
const refetch = () => activeResults.value.refetch()

function loadResultFilters() {
	if (loadFilterOptions.value) void fullResults.refetch()
	else loadFilterOptions.value = true
}

const uniqueFlagKeyCount = computed(() => data.value && countUniqueFlagKeys(data.value))

function resultRow(result: ProjectFlagRunResult) {
	return {
		...result,
		flagCount: result.num_flags,
		category: projectFlagResultCategory(result),
	}
}

const resultRows = computed(() => (data.value ?? []).map(resultRow))
const groupedRows = computed(() => {
	const groups: Record<Category, typeof resultRows.value> = {
		all: resultRows.value,
		flagged: [],
		approvable: [],
		errors: [],
	}
	for (const row of resultRows.value) {
		groups[row.category].push(row)
	}
	return groups
})
const scanErrorCount = computed(() =>
	data.value ? groupedRows.value.errors.length : (props.run.num_errors ?? 0),
)
const resultTabs = computed(() =>
	filters
		.filter(
			(filter) =>
				filter.category !== 'errors' || scanErrorCount.value > 0 || props.category === 'errors',
		)
		.map(({ category, label }) => ({
			value: category,
			label: data.value
				? `${label} (${groupedRows.value[category].length.toLocaleString('en-US')})`
				: category === 'all' && resultPage.value
					? `${label} (${resultPage.value.total.toLocaleString('en-US')})`
					: label,
		})),
)
const ruleOptions = computed(() => {
	const rules = new Map<string, { name: string; count: number }>()
	for (const row of resultRows.value) {
		const inCategory = props.category === 'all' || row.category === props.category
		for (const [id, rule] of Object.entries(row.rules)) {
			const entry = rules.get(id) ?? { name: rule.name ?? `Deleted rule (${id})`, count: 0 }
			if (inCategory && rule.flags.length > 0) entry.count++
			rules.set(id, entry)
		}
	}
	return [...rules]
		.sort(([, a], [, b]) => a.name.localeCompare(b.name, 'en'))
		.map(([value, { name, count }]) => ({
			value,
			label: `${name} (${count.toLocaleString('en-US')})`,
		}))
})
const flagKeyOptions = computed(() => {
	const counts = new Map<string, number>()
	for (const row of resultRows.value) {
		const inCategory = props.category === 'all' || row.category === props.category
		for (const { key } of row.effective_flags) {
			counts.set(key, (counts.get(key) ?? 0) + (inCategory ? 1 : 0))
		}
	}
	return [...counts]
		.sort(([a], [b]) => a.localeCompare(b, 'en'))
		.map(([value, count]) => ({ value, label: `${value} (${count.toLocaleString('en-US')})` }))
})
const projectTypeOptions = computed(() => {
	const counts = new Map<string, number>()
	for (const project of filterProjects.value ?? []) {
		for (const type of new Set(project.project_types)) {
			counts.set(type, (counts.get(type) ?? 0) + 1)
		}
	}
	const countsAvailable = filterProjects.value !== undefined
	return Object.keys(PROJECT_TYPE_ICONS)
		.filter((type) => type !== 'project')
		.map((value) => {
			const name = formatProjectType(value)
			return {
				value,
				label: countsAvailable
					? `${name} (${(counts.get(value) ?? 0).toLocaleString('en-US')})`
					: name,
			}
		})
})
const filterCategories = computed(() =>
	(
		[
			{
				key: 'rule',
				label: 'Rules',
				options: ruleOptions.value,
				searchable: true,
				searchPlaceholder: 'Search rules…',
				submenuClass: 'w-[500px]',
			},
			{
				key: 'flagKey',
				label: 'Flag keys',
				options: flagKeyOptions.value,
				searchable: true,
				searchPlaceholder: 'Search flag keys…',
				submenuClass: 'w-[500px]',
			},
			{
				key: 'projectType',
				label: 'Project types',
				options: projectTypeOptions.value,
				submenuClass: 'w-[500px]',
			},
		] satisfies DropdownFilterBarCategory[]
	).filter((category) => category.options.length > 0),
)
const projectSearch = computed(
	() =>
		new Fuse(groupedRows.value[props.category], {
			keys: ['project.title', 'project.slug', 'project.id'],
			threshold: 0.3,
			ignoreLocation: true,
		}),
)
function matchesSelection(values: string[], mode: MatchMode, matches: (value: string) => boolean) {
	return values.length === 0 || (mode === 'Match all' ? values.every(matches) : values.some(matches))
}

const flagFilteredRows = computed(() => {
	const term = searchTerm.value
	const candidates = term
		? projectSearch.value.search(term).map(({ item }) => item)
		: groupedRows.value[props.category]
	const rules = selectedRules.value
	const keys = selectedFlagKeys.value
	if (rules.length === 0 && keys.length === 0) return candidates
	const { rule: ruleMode, flagKey: flagKeyMode } = matchMode.value
	return candidates.filter(
		(row) =>
			matchesSelection(rules, ruleMode, (id) => (row.rules[id]?.flags.length ?? 0) > 0) &&
			matchesSelection(keys, flagKeyMode, (key) =>
				row.effective_flags.some((flag) => flag.key === key),
			),
	)
})
const {
	filterProjects,
	filterProjectsById,
	loadingProjectTypes,
	projectTypeError,
	refetchProjectTypes,
} = useResultProjects(
	() => groupedRows.value[props.category].map((row) => row.project.id),
	() => projectTypeFilterActive.value,
)
const categoryRows = computed(() => {
	if (!projectTypeFilterActive.value) return flagFilteredRows.value
	return flagFilteredRows.value.filter((row) =>
		filterProjectsById.value
			.get(row.project.id)
			?.project_types.some((type) => selectedProjectTypes.value.includes(type)),
	)
})
const categoryTotal = computed(() =>
	needsAllResults.value ? categoryRows.value.length : (resultPage.value?.total ?? 0),
)
const copyingProjectIds = ref(false)
const resultsReady = computed(
	() =>
		!isPending.value &&
		!isError.value &&
		search.value.trim() === searchTerm.value &&
		!(projectTypeFilterActive.value && (loadingProjectTypes.value || projectTypeError.value)),
)
const resultCountLabel = computed(() => {
	if (!resultsReady.value) return ''
	const count = categoryTotal.value
	const noun = count === 1 ? 'project' : 'projects'
	const matching = hasFilters.value || searchTerm.value ? ' matching' : ''
	return `${count.toLocaleString('en-US')}${matching} ${noun}`
})
const canCopyProjectIds = computed(
	() => resultsReady.value && categoryTotal.value > 0 && !copyingProjectIds.value,
)

function formatProjectIdCount(count: number) {
	return `${count.toLocaleString('en-US')} project ${count === 1 ? 'ID' : 'IDs'}`
}

async function copyProjectIds(event: MouseEvent) {
	if (!canCopyProjectIds.value) return
	const view = (event.currentTarget as HTMLElement | null)?.ownerDocument.defaultView
	const clipboard = view?.navigator.clipboard
	copyingProjectIds.value = true
	try {
		if (!view || !clipboard) throw new Error('Clipboard unavailable')
		let count: number
		if (needsAllResults.value || data.value?.length === categoryTotal.value) {
			const ids = categoryRows.value.map((row) => row.project.id)
			await clipboard.writeText(ids.join(', '))
			count = ids.length
		} else {
			const ids = queryClient
				.fetchQuery(projectFlagRunResultsOptions(client, props.run))
				.then((results) => results.map((result) => result.project.id))
			if (view.ClipboardItem && clipboard.write) {
				const text = ids.then((values) => new Blob([values.join(', ')], { type: 'text/plain' }))
				await Promise.all([text, clipboard.write([new view.ClipboardItem({ 'text/plain': text })])])
			} else {
				await clipboard.writeText((await ids).join(', '))
			}
			count = (await ids).length
		}
		addNotification({ title: `Copied ${formatProjectIdCount(count)}`, type: 'success' })
	} catch {
		addNotification({ title: 'Could not copy project IDs', type: 'error' })
	} finally {
		copyingProjectIds.value = false
	}
}

const selectedProjectIndex = computed(() =>
	categoryRows.value.findIndex((row) => row.project.id === props.projectId),
)
const offset = computed(() =>
	needsAllResults.value
		? Math.floor(Math.max(0, selectedProjectIndex.value) / PAGE_SIZE) * PAGE_SIZE
		: pageOffset.value,
)
const rows = computed(() =>
	needsAllResults.value
		? categoryRows.value.slice(offset.value, offset.value + PAGE_SIZE)
		: (resultPage.value?.results ?? []).map(resultRow),
)
const selectedRow = computed(
	() => rows.value.find((row) => row.project.id === props.projectId) ?? rows.value[0],
)

const { projectsById, ownersByProject } = useResultProjectOwners(
	() => rows.value.map((row) => row.project.id),
	() => projectTypeFilterActive.value && filterProjects.value !== undefined,
	(id) => filterProjectsById.value.get(id),
)

function showCategory(value: TabsValue) {
	const nextTab = filters.find((filter) => filter.category === value)
	if (!nextTab || nextTab.category === props.category) return
	selectedFilters.value.rule = []
	selectedFilters.value.flagKey = []
	emit('changeCategory', nextTab.category)
}

function selectProject(projectId: string) {
	emit('selectProject', projectId)
}

function changePage(page: number) {
	if (!needsAllResults.value) {
		pageOffset.value = (page - 1) * PAGE_SIZE
		return
	}
	const firstRow = categoryRows.value[(page - 1) * PAGE_SIZE]
	if (firstRow) return selectProject(firstRow.project.id)
}

const projectRows = computed(() =>
	rows.value.map((row) => ({
		...row,
		details: projectsById.value.get(row.project.id),
		owner: ownersByProject.value.get(row.project.id),
	})),
)
const emptyMessage = computed(() => {
	if (
		(needsAllResults.value ? !data.value : !resultPage.value) ||
		(projectTypeFilterActive.value && loadingProjectTypes.value) ||
		(projectTypeFilterActive.value && projectTypeError.value)
	)
		return
	if (search.value.trim() || hasFilters.value) return 'No projects match your search or filters.'
	if (props.category === 'all') return 'No project results yet.'
	return typeof props.run.status === 'string'
		? 'No projects in this group.'
		: 'No projects in this group yet. Results refresh automatically.'
})
useFlagErrorNotification(error, 'Could not load project results')
useFlagErrorNotification(
	() => (!needsAllResults.value && loadFilterOptions.value ? fullResults.error.value : null),
	'Could not load rule and flag filters',
)
useFlagErrorNotification(projectTypeError, 'Could not load project types')
</script>
