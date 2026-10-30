<!-- eventually extract content from project settings to be a component so it can be shared here. -->
<script setup lang="ts">
import { Checkbox, EmptyState, sortProjectTypes, TagItem } from '@modrinth/ui'
import { capitalizeString, sortedCategories } from '@modrinth/utils'
import { computed, ref, useTemplateRef } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { categoryHeader, categoryName } from '../tag-labels'
import { projectTypeLabels } from '../type-labels'
import EditModal from './edit-modal.vue'
import { useProjectInfoEdit } from './use-project-edit'

interface Category {
	name: string
	header: string
	icon?: string
	project_type: string
}

interface CategoryGroup {
	id: string
	title: string
	description?: string
	categories: Category[]
}

const MAX_FEATURED_TAGS = 3
const SHARED_CATEGORY_PROJECT_TYPES: Record<string, string> = {
	plugin: 'mod',
	datapack: 'mod',
}

const tags = useGeneratedState()
const { projectV2 } = injectProjectReviewPageContext()
const { project, saving, beginEditing, saveProject } = useProjectInfoEdit()
const modal = useTemplateRef<InstanceType<typeof EditModal>>('modal')
const selectedTags = ref<string[]>([])
const featuredTags = ref<string[]>([])

const isServerProject = computed(() => project.value?.minecraft_server != null)
const canSelectTags = computed(() => !!projectV2.value?.versions.length || isServerProject.value)
const projectTypes = computed(() => {
	if (isServerProject.value) return ['minecraft_java_server']
	return sortProjectTypes(new Set(project.value?.project_types ?? ['mod']))
})
const formatCategoryName = (category: string) => categoryName(category)
const allCategories = computed(
	() => sortedCategories(tags.value, formatCategoryName, 'en-US') as Category[],
)
const projectTypesByCategoryList = computed(() => {
	const lists = new Map<string, string[]>()
	for (const projectType of projectTypes.value) {
		const source = SHARED_CATEGORY_PROJECT_TYPES[projectType] ?? projectType
		lists.set(source, [...(lists.get(source) ?? []), projectType])
	}
	return lists
})
const projectTypeListFormatter = computed(
	() => new Intl.ListFormat('en-US', { style: 'long', type: 'conjunction' }),
)

function formatProjectTypeName(type: string) {
	return (projectTypeLabels[type] ?? 'Project').toLowerCase()
}

function formatGroupTitle(header: string, types: string[]) {
	const title = categoryHeader(header)
	if (
		projectTypes.value.length <= 1 ||
		!['categories', 'features', 'resolutions', 'performance impact'].includes(header)
	)
		return title
	return `${capitalizeString(projectTypeListFormatter.value.format(types.map(formatProjectTypeName)))} ${header}`
}

function formatGroupDescription(header: string, types: string[]) {
	const type = formatProjectTypeName(types.length > 1 ? 'project' : types[0])
	switch (header) {
		case 'categories':
			return `Select all categories that reflect the themes or function of your ${type}.`
		case 'features':
			return `Select all of the features that your ${type} makes use of.`
		case 'resolutions':
			return `Select the resolution(s) of textures in your ${type}.`
		case 'performance impact':
			return `Select the realistic performance impact of your ${type}. Select multiple if the ${type} is configurable to different levels of performance impact.`
		default:
			return undefined
	}
}

function withPokemonFirst(categories: Category[]) {
	return categories.slice().sort((a, b) => {
		if (a.name === 'pokemon') return -1
		if (b.name === 'pokemon') return 1
		return 0
	})
}

const categoryGroups = computed<CategoryGroup[]>(() => {
	const groups: CategoryGroup[] = []
	for (const [categoryList, types] of projectTypesByCategoryList.value) {
		const byHeader = new Map<string, Category[]>()
		for (const category of allCategories.value) {
			if (category.project_type !== categoryList) continue
			byHeader.set(category.header, [...(byHeader.get(category.header) ?? []), category])
		}
		for (const [header, categories] of byHeader) {
			groups.push({
				id: `${categoryList}-${header}`,
				title: formatGroupTitle(header, types),
				description: formatGroupDescription(header, types),
				categories:
					header === 'minecraft_server_features' ? withPokemonFirst(categories) : categories,
			})
		}
	}
	return groups
})
const availableTags = computed(() => [
	...new Set(
		categoryGroups.value.flatMap((group) => group.categories.map((category) => category.name)),
	),
])

function reset() {
	selectedTags.value = availableTags.value.filter(
		(tag) =>
			project.value?.categories.includes(tag) || project.value?.additional_categories.includes(tag),
	)
	featuredTags.value = availableTags.value.filter((tag) => project.value?.categories.includes(tag))
}

function show() {
	beginEditing()
	reset()
	modal.value?.show()
}

function toggleTagRaw(selection: string[], tag: string) {
	if (selection.includes(tag)) return selection.filter((selected) => selected !== tag)
	return availableTags.value.filter(
		(available) => available === tag || selection.includes(available),
	)
}

function toggleTag(tag: string) {
	selectedTags.value = toggleTagRaw(selectedTags.value, tag)
	if (!selectedTags.value.includes(tag)) {
		featuredTags.value = featuredTags.value.filter((featured) => featured !== tag)
	}
}

function toggleFeatured(tag: string) {
	featuredTags.value = toggleTagRaw(featuredTags.value, tag)
}

function hasSameTags(a: string[], b: string[]) {
	return a.length === b.length && a.every((tag) => b.includes(tag))
}

const additionalCategories = computed(() =>
	selectedTags.value.filter((tag) => !featuredTags.value.includes(tag)),
)
const hasChanges = computed(
	() =>
		!hasSameTags(featuredTags.value, project.value?.categories ?? []) ||
		!hasSameTags(additionalCategories.value, project.value?.additional_categories ?? []),
)
const canSave = computed(
	() =>
		canSelectTags.value &&
		hasChanges.value &&
		featuredTags.value.length > 0 &&
		featuredTags.value.length <= MAX_FEATURED_TAGS,
)

async function save() {
	if (!canSave.value) return
	if (
		await saveProject({
			categories: featuredTags.value,
			additional_categories: additionalCategories.value,
		})
	) {
		modal.value?.hide()
	}
}

defineExpose({ show })
</script>

<template>
	<EditModal
		ref="modal"
		section="Tags"
		:saving="saving"
		:can-save="canSave"
		width="60rem"
		@cancel="reset"
		@save="save"
	>
		<EmptyState
			v-if="!canSelectTags"
			type="no-documents"
			heading="Upload versions before adding tags"
			description="Upload a version before selecting project tags."
		/>
		<div v-else class="flex flex-col gap-4">
			<div
				v-for="group in categoryGroups"
				:key="group.id"
				class="rounded-2xl border border-solid border-surface-4 bg-surface-3 p-4"
			>
				<h3 class="mb-1 mt-0 text-lg font-semibold text-contrast">
					{{ group.title }}
				</h3>
				<p v-if="group.description" class="mb-3 mt-0 text-sm text-secondary">
					{{ group.description }}
				</p>
				<div class="grid grid-cols-2 gap-x-4 gap-y-2 sm:grid-cols-4">
					<Checkbox
						v-for="category in group.categories"
						:key="`${group.id}-${category.name}`"
						:model-value="selectedTags.includes(category.name)"
						:disabled="saving"
						:description="formatCategoryName(category.name)"
						@update:model-value="toggleTag(category.name)"
					>
						<span aria-hidden="true">
							{{ categoryName(category.name) }}
						</span>
					</Checkbox>
				</div>
			</div>

			<div class="rounded-2xl border border-solid border-surface-4 bg-surface-3 p-4">
				<div class="mb-1 flex items-center gap-2">
					<h3 class="m-0 text-lg font-semibold text-contrast">Featured tags</h3>
					<TagItem> {{ featuredTags.length }}/{{ MAX_FEATURED_TAGS }} </TagItem>
				</div>
				<p class="mb-3 mt-0 text-sm text-secondary">
					Select your most relevant tags. These are displayed before the rest of your tags.
				</p>
				<p v-if="selectedTags.length === 0" class="m-0 text-secondary">
					Select at least one category in order to feature a category.
				</p>
				<div v-else class="grid grid-cols-2 gap-x-4 gap-y-2 sm:grid-cols-4">
					<Checkbox
						v-for="name in selectedTags"
						:key="`featured-${name}`"
						:model-value="featuredTags.includes(name)"
						:disabled="
							saving || (!featuredTags.includes(name) && featuredTags.length >= MAX_FEATURED_TAGS)
						"
						:description="formatCategoryName(name)"
						@update:model-value="toggleFeatured(name)"
					>
						<span aria-hidden="true">
							{{ categoryName(name) }}
						</span>
					</Checkbox>
				</div>
				<p v-if="featuredTags.length === 0" class="mb-0 mt-3 text-sm text-red" role="alert">
					You must have at least one featured tag.
				</p>
			</div>
		</div>
	</EditModal>
</template>
