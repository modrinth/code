<template>
	<div v-if="isModpack" :class="innerPanelClass" class="flex flex-col gap-3">
		<ProjectCombobox
			:model-value="pendingProjectId"
			:project-types="selectableProjectTypes"
			:exclude-project-ids="dependencyProjectIds"
			:search-placeholder="formatMessage(messages.searchContentPlaceholder)"
			@update:model-value="addIncludedProject"
		/>
		<div v-if="dependencyProjectIds.length > 0" class="flex flex-col gap-1">
			<div
				v-for="projectId in dependencyProjectIds"
				:key="projectId"
				class="group flex min-w-0 items-center gap-2 rounded-xl px-1 py-1 text-primary"
			>
				<img
					v-if="dependentProjectMap.get(projectId)?.icon_url"
					:src="dependentProjectMap.get(projectId)?.icon_url ?? undefined"
					:alt="dependentProjectMap.get(projectId)?.title ?? projectId"
					class="size-6 shrink-0 rounded-md object-cover"
				/>
				<PackageIcon v-else class="size-8 shrink-0 text-secondary" />
				<span class="min-w-0 flex-1 truncate font-medium text-contrast">
					{{ dependentProjectMap.get(projectId)?.title ?? projectId }}
				</span>
				<button
					type="button"
					class="shrink-0 cursor-pointer border-0 bg-transparent p-0 text-sm transition-opacity"
					:class="
						excludedProjectIds.has(projectId)
							? 'text-secondary opacity-100'
							: 'text-secondary opacity-0 underline decoration-1 underline-offset-2 group-hover:opacity-100 group-focus-within:opacity-100 hover:text-contrast'
					"
					@click="toggleProjectExcluded(projectId)"
				>
					{{
						excludedProjectIds.has(projectId)
							? formatMessage(messages.excluded)
							: formatMessage(messages.exclude)
					}}
				</button>
				<button
					type="button"
					class="flex shrink-0 cursor-pointer items-center border-0 bg-transparent p-0 text-secondary hover:text-contrast"
					:aria-label="
						formatMessage(messages.removeIncludedProject, {
							project: dependentProjectMap.get(projectId)?.title ?? projectId,
						})
					"
					@click="removeIncludedProject(projectId)"
				>
					<XIcon class="size-5" />
				</button>
			</div>
		</div>
	</div>
	<div v-else :class="innerPanelClass" class="flex flex-col gap-3">
		<ProjectCombobox
			v-show="!selectedProjectId || refreshing"
			ref="projectCombobox"
			:model-value="selectedProjectId"
			:project-types="selectableProjectTypes"
			:search-placeholder="formatMessage(messages.searchProjectPlaceholder)"
			@update:model-value="setSelectedProjectId"
		/>
		<template v-if="selectedProjectId && !refreshing">
			<div class="flex items-center justify-between gap-3 px-2 text-secondary">
				<span>{{ formatMessage(messages.dependentCount, { count: resultCount ?? 0 }) }}</span>
				<button
					class="border-none bg-transparent p-0 text-secondary cursor-pointer hover:text-contrast"
					@click="setSelectedProjectId(undefined)"
				>
					{{ formatMessage(messages.clearFilter) }}
				</button>
			</div>
			<div class="flex items-center gap-3 rounded-2xl bg-surface-1 p-3">
				<img
					v-if="selectedProject?.icon_url"
					:src="selectedProject.icon_url"
					:alt="selectedProject.title"
					class="size-14 shrink-0 rounded-xl object-cover"
				/>
				<PackageIcon v-else class="size-14 shrink-0 text-secondary" />
				<div class="min-w-0 flex-1">
					<div class="truncate text-base font-bold text-contrast">
						{{ selectedProject?.title ?? selectedProjectId }}
					</div>
					<MultiSelect
						:model-value="dependencyTypes"
						:options="dependencyTypeOptions"
						:clearable="false"
						fit-content
						show-chevron
						trigger-class="!rounded-none !bg-transparent !p-0"
						checkbox-position="right"
						:dropdown-min-width="220"
						@update:model-value="setDependencyTypes"
					>
						<template #input-content="{ isOpen }">
							<span class="flex items-center gap-1 text-sm text-secondary">
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
		</template>
	</div>
</template>

<script setup lang="ts">
import { DropdownIcon, PackageIcon, XIcon } from '@modrinth/assets'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

import { defineMessages, useVIntl } from '../../composables/i18n'
import { injectModrinthClient } from '../../providers'
import type { FilterValue } from '../../utils/search'
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
const dependencyProjectIds = computed(() =>
	selectedFilters.value
		.filter((filter) => filter.type === FILTER_TYPE_ID)
		.map((filter) => filter.option),
)
const pendingProjectId = ref<string>()
const excludedProjectIds = ref(new Set<string>())

watch(dependencyProjectIds, (projectIds) => {
	const selectedProjectIds = new Set(projectIds)
	excludedProjectIds.value = new Set(
		[...excludedProjectIds.value].filter((projectId) => selectedProjectIds.has(projectId)),
	)
})

const { data: dependentProjects } = useQuery({
	queryKey: computed(() => [
		'search-depends-on-filter',
		'dependent-projects',
		dependencyProjectIds.value,
	]),
	queryFn: () => labrinth.projects_v2.getMultiple(dependencyProjectIds.value),
	enabled: computed(() => isModpack.value && dependencyProjectIds.value.length > 0),
	placeholderData: [],
	refetchOnWindowFocus: false,
})

const dependentProjectMap = computed(
	() => new Map(dependentProjects.value?.map((project) => [project.id, project]) ?? []),
)
const projectCombobox = ref<{ selectedProject: SearchHit | null } | null>(null)
const selectedProject = computed(() => projectCombobox.value?.selectedProject ?? null)
const selectedProjectId = computed(
	() => selectedFilters.value.find((filter) => filter.type === FILTER_TYPE_ID)?.option,
)

type DependencyType = 'required' | 'optional' | 'embedded'

const dependencyTypes = ref<DependencyType[]>(['required'])
const dependencyTypeOptions = computed<MultiSelectOption<DependencyType>[]>(() => [
	{ value: 'required', label: formatMessage(messages.required) },
	{ value: 'optional', label: formatMessage(messages.optional) },
	{ value: 'embedded', label: formatMessage(messages.embedded) },
])
const dependencyTypeLabel = computed(() => {
	const selectedTypes = new Set(dependencyTypes.value)
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
	return dependencyTypeOptions.value.find((option) => selectedTypes.has(option.value))?.label ?? ''
})

function setSelectedProjectId(projectId: string | undefined) {
	const otherFilters = selectedFilters.value.filter((filter) => filter.type !== FILTER_TYPE_ID)
	selectedFilters.value = projectId
		? [...otherFilters, { type: FILTER_TYPE_ID, option: projectId }]
		: otherFilters
	dependencyTypes.value = ['required']
}

function addIncludedProject(projectId: string | undefined) {
	if (projectId && !dependencyProjectIds.value.includes(projectId)) {
		selectedFilters.value = [...selectedFilters.value, { type: FILTER_TYPE_ID, option: projectId }]
	}
	pendingProjectId.value = undefined
}

function removeIncludedProject(projectId: string) {
	selectedFilters.value = selectedFilters.value.filter(
		(filter) => filter.type !== FILTER_TYPE_ID || filter.option !== projectId,
	)
	const nextExcludedProjectIds = new Set(excludedProjectIds.value)
	nextExcludedProjectIds.delete(projectId)
	excludedProjectIds.value = nextExcludedProjectIds
}

function toggleProjectExcluded(projectId: string) {
	const nextExcludedProjectIds = new Set(excludedProjectIds.value)
	if (nextExcludedProjectIds.has(projectId)) {
		nextExcludedProjectIds.delete(projectId)
	} else {
		nextExcludedProjectIds.add(projectId)
	}
	excludedProjectIds.value = nextExcludedProjectIds
}

function setDependencyTypes(types: DependencyType[]) {
	if (types.length > 0) {
		dependencyTypes.value = types
	}
}

const messages = defineMessages({
	clearFilter: {
		id: 'search.filter.clear',
		defaultMessage: 'Clear',
	},
	dependentCount: {
		id: 'search.filter.dependent_count',
		defaultMessage: '{count, plural, one {# dependent} other {# dependents}}',
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
