<template>
	<slot
		:results="data"
		:scanned-count="data?.length"
		:scan-error-count="scanErrorCount"
		:flagged-count="data ? groupedRows.flagged.length : undefined"
		:approvable-count="data ? groupedRows.approvable.length : undefined"
		:unique-flag-key-count="uniqueFlagKeyCount"
	/>
	<section class="flex min-w-0 flex-col gap-3" aria-label="Project results">
		<Tabs
			:tabs="resultTabs"
			:value="category"
			color="gray"
			wrap
			aria-label="Project results filter"
			class="bg-surface-2 [&>button[aria-selected=true]]:border-brand [&>button[aria-selected=true]]:bg-brand-highlight [&>button[aria-selected=true]]:text-brand"
			@update:value="showCategory"
		/>

		<div class="flex flex-wrap items-center gap-2">
			<Input
				v-model="search"
				:icon="SearchIcon"
				type="search"
				aria-label="Search projects"
				placeholder="Search projects by name or ID…"
				wrapper-class="min-w-0 w-full max-w-sm"
				clearable
			/>
			<DropdownFilterBar
				v-model="selectedFilters"
				:categories="filterCategories"
				:show-label="false"
				add-label="Filters"
			/>
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

import type { ProjectFlagRun } from '@modrinth/api-client'
import { LoaderCircleIcon, SearchIcon } from '@modrinth/assets'
import {
	Button,
	DropdownFilterBar,
	type DropdownFilterBarCategory,
	injectModrinthClient,
	Input,
	PROJECT_TYPE_ICONS,
	Tabs,
	type TabsValue,
} from '@modrinth/ui'
import { formatProjectType } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import Fuse from 'fuse.js'

import {
	projectFlagResultCategory,
	projectFlagRunResultsOptions,
} from '~/composables/queries/project-flag-run'

import { countUniqueFlagKeys } from '../run-format'
import ProjectDetail from './project-detail.vue'
import ProjectList from './project-list.vue'
import { useResultProjects, useResultProjectOwners } from './use-result-projects'

const PAGE_SIZE = 25
const filters = [
	{ label: 'Flagged', category: 'flagged' },
	{ label: 'Approvable', category: 'approvable' },
	{ label: 'Scan errors', category: 'errors' },
] as const
type Category = (typeof filters)[number]['category']

const props = defineProps<{ run: ProjectFlagRun; category: Category; projectId?: string }>()
const emit = defineEmits<{ changeCategory: [category: Category]; selectProject: [id: string] }>()
const client = injectModrinthClient()

const search = ref('')
const selectedFilters = ref<Record<string, string[]>>({})
const selectedRules = computed(() => selectedFilters.value.rule ?? [])
const selectedFlaggedRules = computed(() => selectedFilters.value.flaggedRule ?? [])
const selectedProjectTypes = computed(() => selectedFilters.value.projectType ?? [])
const projectTypeFilterActive = computed(() => selectedProjectTypes.value.length > 0)
const hasFilters = computed(
	() =>
		selectedRules.value.length > 0 ||
		selectedFlaggedRules.value.length > 0 ||
		projectTypeFilterActive.value,
)
// TODO: Request only the current results page with server-side search and filters. Use the API's
// matching total and filter counts, and fetch project details only for the returned IDs.
const { data, isPending, isError, error, isFetching, refetch } = useQuery(
	computed(() => projectFlagRunResultsOptions(client, props.run)),
)

const uniqueFlagKeyCount = computed(() => data.value && countUniqueFlagKeys(data.value))

const resultRows = computed(() =>
	(data.value ?? []).map((result) => {
		const rules = Object.entries(result.rules)
		return {
			...result,
			flagCount: rules.reduce((count, [, rule]) => count + rule.flags.length, 0),
			category: projectFlagResultCategory(result),
		}
	}),
)
const groupedRows = computed(() => {
	const groups: Record<Category, typeof resultRows.value> = {
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
				: label,
		})),
)
const ruleOptions = computed(() => {
	const rules = new Map<string, string>()
	for (const row of resultRows.value) {
		for (const [id, rule] of Object.entries(row.rules)) {
			rules.set(id, rule.name ?? `Deleted rule (${id})`)
		}
	}
	return [...rules]
		.map(([value, label]) => ({ value, label }))
		.sort((a, b) => a.label.localeCompare(b.label, 'en'))
})
const projectTypeOptions = computed(() => {
	const counts = new Map<string, number>()
	for (const project of filterProjects.value ?? []) {
		for (const type of new Set(project.project_types)) {
			counts.set(type, (counts.get(type) ?? 0) + 1)
		}
	}
	const countsAvailable =
		filterProjects.value !== undefined || (data.value && !groupedRows.value[props.category].length)
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
const flaggedRuleOptions = computed(() => {
	const rules = new Map<string, { name: string; count: number }>()
	for (const row of groupedRows.value[props.category]) {
		for (const [id, rule] of Object.entries(row.rules)) {
			if (!rule.flags.length) continue
			const entry = rules.get(id)
			if (entry) entry.count++
			else rules.set(id, { name: rule.name ?? `Deleted rule (${id})`, count: 1 })
		}
	}
	return [...rules]
		.sort(([, a], [, b]) => a.name.localeCompare(b.name, 'en'))
		.map(([value, { name, count }]) => ({
			value,
			label: `${name} (${count.toLocaleString('en-US')})`,
		}))
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
				key: 'flaggedRule',
				label: 'Flagged rules',
				options: flaggedRuleOptions.value,
				searchable: true,
				searchPlaceholder: 'Search flagged rules…',
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
const ruleFilteredRows = computed(() => {
	const term = search.value.trim()
	const candidates = term
		? projectSearch.value.search(term).map(({ item }) => item)
		: groupedRows.value[props.category]
	return candidates.filter((row) => {
		if (
			selectedFlaggedRules.value.length &&
			!selectedFlaggedRules.value.some((id) => (row.rules[id]?.flags.length ?? 0) > 0)
		) {
			return false
		}
		return (
			selectedRules.value.length === 0 ||
			selectedRules.value.some((id) => Object.hasOwn(row.rules, id))
		)
	})
})
const {
	filterProjects,
	filterProjectsById,
	loadingProjectTypes,
	projectTypeError,
	refetchProjectTypes,
} = useResultProjects(() => groupedRows.value[props.category].map((row) => row.project.id))
const categoryRows = computed(() => {
	if (!projectTypeFilterActive.value) return ruleFilteredRows.value
	return ruleFilteredRows.value.filter((row) =>
		filterProjectsById.value
			.get(row.project.id)
			?.project_types.some((type) => selectedProjectTypes.value.includes(type)),
	)
})
const categoryTotal = computed(() => categoryRows.value.length)
const selectedProjectIndex = computed(() =>
	categoryRows.value.findIndex((row) => row.project.id === props.projectId),
)
const offset = computed(
	() => Math.floor(Math.max(0, selectedProjectIndex.value) / PAGE_SIZE) * PAGE_SIZE,
)
const rows = computed(() => categoryRows.value.slice(offset.value, offset.value + PAGE_SIZE))
const selectedRow = computed(() => categoryRows.value[selectedProjectIndex.value] ?? rows.value[0])

const { projectsById, ownersByProject } = useResultProjectOwners(
	() => rows.value.map((row) => row.project.id),
	() => filterProjects.value !== undefined,
	(id) => filterProjectsById.value.get(id),
)

function showCategory(value: TabsValue) {
	const nextTab = filters.find((filter) => filter.category === value)
	if (!nextTab || nextTab.category === props.category) return
	selectedFilters.value.flaggedRule = []
	emit('changeCategory', nextTab.category)
}

function selectProject(projectId: string) {
	emit('selectProject', projectId)
}

function changePage(page: number) {
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
		!data.value ||
		(projectTypeFilterActive.value && loadingProjectTypes.value) ||
		(projectTypeFilterActive.value && projectTypeError.value)
	)
		return
	if (search.value.trim() || hasFilters.value) return 'No projects match your search or filters.'
	return props.run.status === 'done'
		? 'No projects in this group.'
		: 'No projects in this group yet. Results refresh automatically.'
})
useFlagErrorNotification(error, 'Could not load project results')
useFlagErrorNotification(projectTypeError, 'Could not load project types')
</script>
