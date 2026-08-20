<template>
	<div v-if="isModpack" :class="innerPanelClass" class="flex flex-col gap-3">
		<ProjectCombobox
			:model-value="pendingProjectId"
			:project-types="selectableProjectTypes"
			:exclude-project-ids="dependencyProjectIds"
			:search-placeholder="formatMessage(messages.searchContentPlaceholder)"
			:show-chevron="false"
			:sync-with-selection="false"
			clearable
			search-input-variant="button"
			show-search-icon
			@update:model-value="addIncludedProject"
		/>
		<div v-if="loadedDependentProjects.length > 0" class="flex flex-col gap-1">
			<div
				v-for="{ projectId, project } in loadedDependentProjects"
				:key="projectId"
				class="group flex min-w-0 items-center gap-2 rounded-xl px-1 pr-0 py-1 text-primary"
			>
				<img
					v-if="project.icon_url"
					:src="project.icon_url"
					:alt="project.title"
					class="size-6 shrink-0 rounded-md object-cover"
				/>
				<PackageIcon v-else class="size-8 shrink-0 text-secondary" />
				<span class="min-w-0 flex-1 truncate font-medium text-contrast">
					{{ project.title }}
				</span>
				<div class="flex items-center gap-0.5">
					<button
						v-tooltip="
							excludedProjectIds.has(projectId)
								? formatMessage(messages.excluded)
								: formatMessage(messages.exclude)
						"
						type="button"
						class="flex shrink-0 cursor-pointer items-center justify-center rounded-xl border-0 px-2 py-2 transition-all [@media(hover:hover)]:py-1"
						:class="
							excludedProjectIds.has(projectId)
								? 'text-red opacity-100 bg-highlight-red'
								: 'text-secondary [@media(hover:hover)]:opacity-0 group-hover:opacity-100 bg-transparent hover:bg-button-bg hover:text-red active:scale-[0.96]'
						"
						:aria-label="
							excludedProjectIds.has(projectId)
								? formatMessage(messages.excluded)
								: formatMessage(messages.exclude)
						"
						@click="toggleProjectExcluded(projectId)"
					>
						<BanIcon class="size-4" aria-hidden="true" />
					</button>
					<button
						v-tooltip="formatMessage(messages.removeIncludedProjectTooltip)"
						type="button"
						class="flex shrink-0 cursor-pointer items-center justify-center rounded-xl border-0 bg-transparent px-2 py-2 text-secondary transition-all [@media(hover:hover)]:py-1 [@media(hover:hover)]:opacity-0 group-hover:opacity-100 hover:bg-button-bg hover:text-contrast active:scale-[0.96]"
						:aria-label="
							formatMessage(messages.removeIncludedProject, {
								project: project.title,
							})
						"
						@click="removeIncludedProject(projectId)"
					>
						<XIcon class="size-4" aria-hidden="true" />
					</button>
				</div>
			</div>
		</div>
	</div>
	<div v-else :class="innerPanelClass" class="flex flex-col gap-3">
		<ProjectCombobox
			v-show="!showSelectedProject"
			ref="projectCombobox"
			:model-value="selectedProjectId"
			:project-types="selectableProjectTypes"
			:search-placeholder="formatMessage(messages.searchProjectPlaceholder)"
			:show-chevron="false"
			clearable
			search-input-variant="button"
			select-search-text-on-focus
			show-search-icon
			@update:model-value="setSelectedProjectId"
		/>
		<template v-if="showSelectedProject">
			<div
				class="flex items-center gap-2 rounded-2xl p-2.5"
				:class="selectedProjectClass ?? 'bg-surface-2'"
			>
				<img
					v-if="selectedProject?.icon_url"
					:src="selectedProject.icon_url"
					:alt="selectedProject.title"
					class="size-12 shrink-0 rounded-xl object-cover"
				/>
				<PackageIcon v-else class="size-14 shrink-0 text-secondary" />
				<div class="min-w-0 flex-1">
					<div class="truncate text-base font-bold text-contrast">
						{{ selectedProject?.title ?? selectedProjectId }}
					</div>
					<div class="relative right-2.5">
						<MultiSelect
							:model-value="draftDependencyTypes"
							:options="dependencyTypeOptions"
							:clearable="false"
							fit-content
							show-chevron
							trigger-class="!rounded-none !bg-transparent !p-0"
							checkbox-position="right"
							:dropdown-min-width="220"
							@open="resetDraftDependencyTypes"
							@close="commitDependencyTypes"
							@update:model-value="setDraftDependencyTypes"
						>
							<template #input-content="{ isOpen }">
								<span class="flex items-center gap-0.5 text-sm text-secondary ml-2.5">
									{{ dependencyTypeLabel }}
									<DropdownIcon
										class="size-4 transition-transform"
										:class="{ 'rotate-180': isOpen }"
									/>
								</span>
							</template>
						</MultiSelect>
					</div>
				</div>
				<IconButton
					v-tooltip="formatMessage(messages.clearDependencyFilter)"
					type="quiet"
					size="sm"
					class="ml-auto"
					:label="formatMessage(messages.clearDependencyFilter)"
					@click="setSelectedProjectId(undefined)"
				>
					<XIcon aria-hidden="true" />
				</IconButton>
			</div>
		</template>
	</div>
</template>

<script setup lang="ts">
import { BanIcon, DropdownIcon, PackageIcon, XIcon } from '@modrinth/assets'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

import { defineMessages, useVIntl } from '../../composables/i18n'
import { injectModrinthClient } from '../../providers'
import {
	type DependencyType,
	type FilterValue,
	formatDependencyProjectFilterOption,
	parseDependencyProjectFilterOption,
} from '../../utils/search'
import { IconButton } from '../base/buttons'
import MultiSelect, { type MultiSelectOption } from '../base/MultiSelect.vue'
import ProjectCombobox, {
	type ProjectType as ProjectComboboxProjectType,
	type SearchHit,
} from '../project/ProjectCombobox.vue'

const FILTER_TYPE_ID = 'compatible_dependency_project_ids'

const { formatMessage } = useVIntl()
const { labrinth } = injectModrinthClient()

const selectedFilters = defineModel<FilterValue[]>('selectedFilters', { required: true })

const props = defineProps<{
	projectType: string
	innerPanelClass?: string
	selectedProjectClass?: string
	resultCount?: number
	refreshing?: boolean
}>()

const selectableProjectTypes: ProjectComboboxProjectType[] = [
	'mod',
	'resourcepack',
	'shader',
	'datapack',
	'plugin',
]
const isModpack = computed(() => props.projectType === 'modpack')
const dependencyProjectIds = computed(() => [
	...new Set(
		selectedFilters.value
			.filter((filter) => filter.type === FILTER_TYPE_ID)
			.map((filter) => parseDependencyProjectFilterOption(filter.option).projectId),
	),
])
const pendingProjectId = ref<string>()
const excludedProjectIds = computed(
	() =>
		new Set(
			selectedFilters.value
				.filter((filter) => filter.type === FILTER_TYPE_ID && filter.negative)
				.map((filter) => parseDependencyProjectFilterOption(filter.option).projectId),
		),
)

const { data: dependentProjects } = useQuery({
	queryKey: computed(() => [
		'search-depends-on-filter',
		'dependent-projects',
		dependencyProjectIds.value,
	]),
	queryFn: () => labrinth.projects_v2.getMultiple(dependencyProjectIds.value),
	enabled: computed(() => isModpack.value && dependencyProjectIds.value.length > 0),
	placeholderData: (previousData) => previousData,
	refetchOnWindowFocus: false,
})

const dependentProjectMap = computed(
	() => new Map(dependentProjects.value?.map((project) => [project.id, project]) ?? []),
)
const loadedDependentProjects = computed(() =>
	dependencyProjectIds.value.flatMap((projectId) => {
		const project = dependentProjectMap.value.get(projectId)
		return project ? [{ projectId, project }] : []
	}),
)
const projectCombobox = ref<{ selectedProject: SearchHit | null } | null>(null)
const selectedProject = computed(() => projectCombobox.value?.selectedProject ?? null)
const selectedDependencyFilter = computed(() =>
	selectedFilters.value.find((filter) => filter.type === FILTER_TYPE_ID),
)
const selectedProjectId = computed(() =>
	selectedDependencyFilter.value
		? parseDependencyProjectFilterOption(selectedDependencyFilter.value.option).projectId
		: undefined,
)
const showSelectedProject = computed(() =>
	Boolean(selectedProjectId.value && selectedProject.value),
)

const dependencyTypes = computed<DependencyType[]>(() =>
	selectedDependencyFilter.value
		? parseDependencyProjectFilterOption(selectedDependencyFilter.value.option).dependencyTypes
		: ['required'],
)
const draftDependencyTypes = ref<DependencyType[]>([...dependencyTypes.value])

watch(
	dependencyTypes,
	(types) => {
		draftDependencyTypes.value = [...types]
	},
	{ immediate: true },
)

const dependencyTypeOptions = computed<MultiSelectOption<DependencyType>[]>(() => [
	{ value: 'required', label: formatMessage(messages.required) },
	{ value: 'optional', label: formatMessage(messages.optional) },
	{ value: 'embedded', label: formatMessage(messages.embedded) },
])
const dependencyTypeLabel = computed(() => {
	const selectedTypes = new Set(draftDependencyTypes.value)
	if (selectedTypes.size === dependencyTypeOptions.value.length) {
		return formatMessage(messages.anyDependencyType)
	}
	if (selectedTypes.has('required') && selectedTypes.has('optional')) {
		return formatMessage(messages.requiredOrOptional)
	}
	if (selectedTypes.has('required') && selectedTypes.has('embedded')) {
		return formatMessage(messages.requiredOrEmbedded)
	}
	if (selectedTypes.has('optional') && selectedTypes.has('embedded')) {
		return formatMessage(messages.optionalOrEmbedded)
	}
	return (
		dependencyTypeOptions.value.find((option) => selectedTypes.has(option.value))?.label ??
		'Select type'
	)
})

function setSelectedProjectId(projectId: string | undefined) {
	const otherFilters = selectedFilters.value.filter((filter) => filter.type !== FILTER_TYPE_ID)
	selectedFilters.value = projectId
		? [
				...otherFilters,
				{
					type: FILTER_TYPE_ID,
					option: formatDependencyProjectFilterOption(projectId, ['required']),
				},
			]
		: otherFilters
}

function addIncludedProject(projectId: string | undefined) {
	if (projectId && !dependencyProjectIds.value.includes(projectId)) {
		selectedFilters.value = [...selectedFilters.value, { type: FILTER_TYPE_ID, option: projectId }]
	}
	pendingProjectId.value = undefined
}

function removeIncludedProject(projectId: string) {
	selectedFilters.value = selectedFilters.value.filter(
		(filter) =>
			filter.type !== FILTER_TYPE_ID ||
			parseDependencyProjectFilterOption(filter.option).projectId !== projectId,
	)
}

function toggleProjectExcluded(projectId: string) {
	selectedFilters.value = selectedFilters.value.map((filter) =>
		filter.type === FILTER_TYPE_ID &&
		parseDependencyProjectFilterOption(filter.option).projectId === projectId
			? { ...filter, negative: !filter.negative }
			: filter,
	)
}

function resetDraftDependencyTypes() {
	draftDependencyTypes.value = [...dependencyTypes.value]
}

function setDraftDependencyTypes(types: DependencyType[]) {
	draftDependencyTypes.value = types
}

function commitDependencyTypes() {
	const projectId = selectedProjectId.value
	const types: DependencyType[] =
		draftDependencyTypes.value.length > 0 ? draftDependencyTypes.value : ['required']
	draftDependencyTypes.value = types
	const selectedTypes = new Set(dependencyTypes.value)
	const changed =
		types.length !== selectedTypes.size || types.some((type) => !selectedTypes.has(type))

	if (changed && projectId) {
		selectedFilters.value = selectedFilters.value.map((filter) =>
			filter.type === FILTER_TYPE_ID
				? {
						...filter,
						option: formatDependencyProjectFilterOption(projectId, types),
					}
				: filter,
		)
	}
}

const messages = defineMessages({
	clearDependencyFilter: {
		id: 'search.filter.dependent_project.clear',
		defaultMessage: 'Clear dependency filter',
	},
	searchContentPlaceholder: {
		id: 'search.filter.included_content.search_placeholder',
		defaultMessage: 'Search content...',
	},
	searchProjectPlaceholder: {
		id: 'search.filter.dependent_project.search_placeholder',
		defaultMessage: 'Search for a project...',
	},
	exclude: {
		id: 'search.filter.included_content.exclude',
		defaultMessage: 'Exclude',
	},
	excluded: {
		id: 'search.filter.included_content.excluded',
		defaultMessage: 'Excluded',
	},
	removeIncludedProject: {
		id: 'search.filter.included_content.remove_project',
		defaultMessage: 'Remove {project}',
	},
	removeIncludedProjectTooltip: {
		id: 'search.filter.included_content.remove_project.tooltip',
		defaultMessage: 'Remove project',
	},
	required: {
		id: 'search.filter.dependency_type.required',
		defaultMessage: 'Required',
	},
	optional: {
		id: 'search.filter.dependency_type.optional',
		defaultMessage: 'Optional',
	},
	embedded: {
		id: 'search.filter.dependency_type.embedded',
		defaultMessage: 'Embedded',
	},
	anyDependencyType: {
		id: 'search.filter.dependency_type.any',
		defaultMessage: 'Any dependency type',
	},
	requiredOrOptional: {
		id: 'search.filter.dependency_type.required_or_optional',
		defaultMessage: 'Required or optional',
	},
	requiredOrEmbedded: {
		id: 'search.filter.dependency_type.required_or_embedded',
		defaultMessage: 'Required or embedded',
	},
	optionalOrEmbedded: {
		id: 'search.filter.dependency_type.optional_or_embedded',
		defaultMessage: 'Optional or embedded',
	},
})
</script>
