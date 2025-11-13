<template>
	<div>
		<section class="universal-card">
			<div class="label">
				<h3>
					<span class="label__title size-card-header">Tags</span>
				</h3>
			</div>

			<div
				v-if="tooManyTagsWarning && !allTagsSelectedWarning"
				class="my-2 flex items-center gap-1.5 text-orange"
			>
				<TriangleAlertIcon class="my-auto" />
				{{ tooManyTagsWarning }}
			</div>

			<div v-if="multipleResolutionTagsWarning" class="my-2 flex items-center gap-1.5 text-orange">
				<TriangleAlertIcon class="my-auto" />
				{{ multipleResolutionTagsWarning }}
			</div>

			<div v-if="allTagsSelectedWarning" class="my-2 flex items-center gap-1.5 text-red">
				<TriangleAlertIcon class="my-auto" />
				<span>{{ allTagsSelectedWarning }}</span>
			</div>

			<p>
				Accurate tagging is important to help people find your
				{{ formatProjectType(project.project_type).toLowerCase() }}. Make sure to select all tags
				that apply.
			</p>

			<p v-if="project.versions.length === 0" class="known-errors">
				Please upload a version first in order to select tags!
			</p>
			<template v-else>
				<template v-for="header in Object.keys(categoryLists)" :key="`categories-${header}`">
					<div class="label mb-3">
						<h4>
							<span class="label__title">{{ formatCategoryHeader(header) }}</span>
						</h4>
						<span class="label__description">
							<template v-if="header === 'categories'">
								Select all categories that reflect the themes or function of your
								{{ formatProjectType(project.project_type).toLowerCase() }}.
							</template>
							<template v-else-if="header === 'features'">
								Select all of the features that your
								{{ formatProjectType(project.project_type).toLowerCase() }} makes use of.
							</template>
							<template v-else-if="header === 'resolutions'">
								Select the resolution(s) of textures in your
								{{ formatProjectType(project.project_type).toLowerCase() }}.
							</template>
							<template v-else-if="header === 'performance impact'">
								Select the realistic performance impact of your
								{{ formatProjectType(project.project_type).toLowerCase() }}. Select multiple if the
								{{ formatProjectType(project.project_type).toLowerCase() }} is configurable to
								different levels of performance impact.
							</template>
						</span>
					</div>
					<div class="category-list input-div">
						<Checkbox
							v-for="category in categoryLists[header]"
							:key="`category-${header}-${category.name}`"
							:model-value="selectedTags.includes(category)"
							:description="formatCategory(category.name)"
							class="category-selector"
							@update:model-value="toggleCategory(category)"
						>
							<div class="category-selector__label">
								<div
									v-if="header !== 'resolutions' && category.icon"
									aria-hidden="true"
									class="icon"
									v-html="category.icon"
								/>
								<span aria-hidden="true"> {{ formatCategory(category.name) }}</span>
							</div>
						</Checkbox>
					</div>
				</template>
				<div class="label">
					<h4>
						<span class="label__title"><StarIcon /> Featured tags</span>
					</h4>
					<span class="label__description">
						You can feature up to 3 of your most relevant tags. Other tags may be promoted to
						featured if you do not select all 3.
					</span>
				</div>
				<p v-if="selectedTags.length < 1">
					Select at least one category in order to feature a category.
				</p>
				<div class="category-list input-div">
					<Checkbox
						v-for="category in selectedTags"
						:key="`featured-category-${category.name}`"
						class="category-selector"
						:model-value="featuredTags.includes(category)"
						:description="formatCategory(category.name)"
						:disabled="featuredTags.length >= 3 && !featuredTags.includes(category)"
						@update:model-value="toggleFeaturedCategory(category)"
					>
						<div class="category-selector__label">
							<div
								v-if="category.header !== 'resolutions' && category.icon"
								aria-hidden="true"
								class="icon"
								v-html="category.icon"
							/>
							<span aria-hidden="true"> {{ formatCategory(category.name) }}</span>
						</div>
					</Checkbox>
				</div>
			</template>

			<div class="button-group">
				<button
					type="button"
					class="iconified-button brand-button"
					:disabled="!hasChanges"
					@click="saveChanges()"
				>
					<SaveIcon />
					Save changes
				</button>
			</div>
		</section>
	</div>
</template>

<script setup lang="ts">
import { SaveIcon, StarIcon, TriangleAlertIcon } from '@modrinth/assets'
import { Checkbox } from '@modrinth/ui'
import {
	formatCategory,
	formatCategoryHeader,
	formatProjectType,
	type Project,
	sortedCategories,
} from '@modrinth/utils'
import { computed, ref } from 'vue'

interface Category {
	name: string
	header: string
	icon?: string
	project_type: string
}

interface Props {
	project: Project & {
		actualProjectType: string
	}
	allMembers?: any[]
	currentMember?: any
	patchProject?: (data: any) => void
}

const tags = useGeneratedState()

const props = withDefaults(defineProps<Props>(), {
	allMembers: () => [],
	currentMember: null,
	patchProject: () => {
		addNotification({
			title: 'An error occurred',
			text: 'Patch project function not found',
			type: 'error',
		})
	},
})

const selectedTags = ref<Category[]>(
	sortedCategories(tags.value).filter(
		(x: Category) =>
			x.project_type === props.project.actualProjectType &&
			(props.project.categories.includes(x.name) ||
				props.project.additional_categories.includes(x.name)),
	),
)

const featuredTags = ref<Category[]>(
	sortedCategories(tags.value).filter(
		(x: Category) =>
			x.project_type === props.project.actualProjectType &&
			props.project.categories.includes(x.name),
	),
)

const categoryLists = computed(() => {
	const lists: Record<string, Category[]> = {}
	sortedCategories(tags.value).forEach((x: Category) => {
		if (x.project_type === props.project.actualProjectType) {
			const header = x.header
			if (!lists[header]) {
				lists[header] = []
			}
			lists[header].push(x)
		}
	})
	return lists
})

const tooManyTagsWarning = computed(() => {
	const tagCount = selectedTags.value.length
	if (tagCount > 8) {
		return `You've selected ${tagCount} tags. Consider reducing to 8 or fewer to keep your project focused and easier to discover.`
	}
	return null
})

const multipleResolutionTagsWarning = computed(() => {
	if (props.project.project_type !== 'resourcepack') return null

	const resolutionTags = selectedTags.value.filter((tag) =>
		['8x-', '16x', '32x', '48x', '64x', '128x', '256x', '512x+'].includes(tag.name),
	)

	if (resolutionTags.length > 1) {
		return `You've selected ${resolutionTags.length} resolution tags (${resolutionTags
			.map((t) => t.name)
			.join(', ')
			.replace('8x-', '8x or lower')
			.replace(
				'512x+',
				'512x or higher',
			)}). Resource packs should typically only have one resolution tag.`
	}
	return null
})

const allTagsSelectedWarning = computed(() => {
	const categoriesForProjectType = sortedCategories(tags.value).filter(
		(x: Category) => x.project_type === props.project.actualProjectType,
	)
	const totalSelectedTags = selectedTags.value.length

	if (
		totalSelectedTags === categoriesForProjectType.length &&
		categoriesForProjectType.length > 0
	) {
		return `You've selected all ${categoriesForProjectType.length} available tags. Please select only the tags that truly apply to your project.`
	}
	return null
})

const patchData = computed(() => {
	const data: Record<string, string[]> = {}

	// Promote selected categories to featured if there are less than 3 featured
	const newFeaturedTags = featuredTags.value.slice()
	if (newFeaturedTags.length < 1 && selectedTags.value.length > newFeaturedTags.length) {
		const nonFeaturedCategories = selectedTags.value.filter((x) => !newFeaturedTags.includes(x))

		nonFeaturedCategories
			.slice(0, Math.min(nonFeaturedCategories.length, 3 - newFeaturedTags.length))
			.forEach((x) => newFeaturedTags.push(x))
	}

	// Convert selected and featured categories to backend-usable arrays
	const categories = newFeaturedTags.map((x) => x.name)
	const additionalCategories = selectedTags.value
		.filter((x) => !newFeaturedTags.includes(x))
		.map((x) => x.name)

	if (
		categories.length !== props.project.categories.length ||
		categories.some((value) => !props.project.categories.includes(value))
	) {
		data.categories = categories
	}

	if (
		additionalCategories.length !== props.project.additional_categories.length ||
		additionalCategories.some((value) => !props.project.additional_categories.includes(value))
	) {
		data.additional_categories = additionalCategories
	}

	return data
})

const hasChanges = computed(() => {
	return Object.keys(patchData.value).length > 0
})

const toggleCategory = (category: Category) => {
	if (selectedTags.value.includes(category)) {
		selectedTags.value = selectedTags.value.filter((x) => x !== category)
		if (featuredTags.value.includes(category)) {
			featuredTags.value = featuredTags.value.filter((x) => x !== category)
		}
	} else {
		selectedTags.value.push(category)
	}
}

const toggleFeaturedCategory = (category: Category) => {
	if (featuredTags.value.includes(category)) {
		featuredTags.value = featuredTags.value.filter((x) => x !== category)
	} else {
		featuredTags.value.push(category)
	}
}

const saveChanges = () => {
	if (hasChanges.value) {
		props.patchProject(patchData.value)
	}
}
</script>

<style lang="scss" scoped>
.label__title {
	display: flex;
	align-items: center;
	gap: var(--spacing-card-xs);
	margin-top: var(--spacing-card-bg);

	svg {
		vertical-align: top;
	}
}

.button-group {
	justify-content: flex-start;
}

.category-list {
	column-count: 4;
	column-gap: var(--spacing-card-lg);
	margin-bottom: var(--spacing-card-md);

	:deep(.category-selector) {
		margin-bottom: 0.75rem;

		.category-selector__label {
			display: flex;
			align-items: center;

			.icon {
				height: 1rem;

				svg {
					margin-right: 0.25rem;
					width: 1rem;
					height: 1rem;
				}
			}
		}

		span {
			user-select: none;
		}
	}

	@media only screen and (max-width: 1250px) {
		column-count: 3;
	}
	@media only screen and (max-width: 1024px) {
		column-count: 4;
	}
	@media only screen and (max-width: 960px) {
		column-count: 3;
	}
	@media only screen and (max-width: 750px) {
		column-count: 2;
	}
	@media only screen and (max-width: 530px) {
		column-count: 1;
	}
}
</style>
