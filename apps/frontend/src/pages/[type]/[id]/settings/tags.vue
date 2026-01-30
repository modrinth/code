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
							:model-value="current.selectedTags.includes(category)"
							:description="
								typeof getTagMessageOrDefault(category.name, 'category') === 'string'
									? getTagMessageOrDefault(category.name, 'category')
									: formatMessage(getTagMessageOrDefault(category.name, 'category'))
							"
							class="category-selector"
							@update:model-value="toggleCategory(category)"
						>
							<div class="category-selector__label">
								<component
									:is="getCategoryIcon(category.name)"
									v-if="header !== 'resolutions' && getCategoryIcon(category.name)"
									aria-hidden="true"
									class="icon"
								/>
								<span aria-hidden="true">
									<FormattedTag :tag="category.name" enforce-type="category" />
								</span>
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
				<p v-if="current.selectedTags.length < 1">
					Select at least one category in order to feature a category.
				</p>
				<div class="category-list input-div">
					<Checkbox
						v-for="category in current.selectedTags"
						:key="`featured-category-${category.name}`"
						class="category-selector"
						:model-value="current.featuredTags.includes(category)"
						:description="
							typeof getTagMessageOrDefault(category.name, 'category') === 'string'
								? getTagMessageOrDefault(category.name, 'category')
								: formatMessage(getTagMessageOrDefault(category.name, 'category'))
						"
						:disabled="current.featuredTags.length >= 3 && !current.featuredTags.includes(category)"
						@update:model-value="toggleFeaturedCategory(category)"
					>
						<div class="category-selector__label">
							<component
								:is="getCategoryIcon(category.name)"
								v-if="category.header !== 'resolutions' && getCategoryIcon(category.name)"
								aria-hidden="true"
								class="icon"
							/>
							<span aria-hidden="true">
								<FormattedTag :tag="category.name" enforce-type="category" />
							</span>
						</div>
					</Checkbox>
				</div>
			</template>
		</section>
		<UnsavedChangesPopup
			:original="saved"
			:modified="current"
			:saving="saving"
			@reset="reset"
			@save="save"
		/>
	</div>
</template>

<script setup lang="ts">
import { getCategoryIcon, StarIcon, TriangleAlertIcon } from '@modrinth/assets'
import {
	Checkbox,
	FormattedTag,
	getTagMessageOrDefault,
	injectProjectPageContext,
	UnsavedChangesPopup,
	useSavable,
	useVIntl,
} from '@modrinth/ui'
import { formatCategoryHeader, formatProjectType, sortedCategories } from '@modrinth/utils'
import { computed } from 'vue'

interface Category {
	name: string
	header: string
	icon?: string
	project_type: string
}

const tags = useGeneratedState()
const { formatMessage } = useVIntl()

const { projectV2: project, patchProject } = injectProjectPageContext()

const { saved, current, saving, reset, save } = useSavable(
	() => ({
		selectedTags: sortedCategories(tags.value).filter(
			(x: Category) =>
				x.project_type === project.value.actualProjectType &&
				(project.value.categories.includes(x.name) ||
					project.value.additional_categories.includes(x.name)),
		) as Category[],
		featuredTags: sortedCategories(tags.value).filter(
			(x: Category) =>
				x.project_type === project.value.actualProjectType &&
				project.value.categories.includes(x.name),
		) as Category[],
	}),
	async () => {
		// Promote selected categories to featured if there are less than 3 featured
		const newFeaturedTags = current.value.featuredTags.slice()
		if (newFeaturedTags.length < 1 && current.value.selectedTags.length > newFeaturedTags.length) {
			const nonFeaturedCategories = current.value.selectedTags.filter(
				(x) => !newFeaturedTags.includes(x),
			)
			nonFeaturedCategories
				.slice(0, Math.min(nonFeaturedCategories.length, 3 - newFeaturedTags.length))
				.forEach((x) => newFeaturedTags.push(x))
		}

		// Convert selected and featured categories to backend-usable arrays
		const categories = newFeaturedTags.map((x) => x.name)
		const additionalCategories = current.value.selectedTags
			.filter((x) => !newFeaturedTags.includes(x))
			.map((x) => x.name)

		const data: Record<string, string[]> = {}

		if (
			categories.length !== project.value.categories.length ||
			categories.some((value) => !project.value.categories.includes(value))
		) {
			data.categories = categories
		}

		if (
			additionalCategories.length !== project.value.additional_categories.length ||
			additionalCategories.some((value) => !project.value.additional_categories.includes(value))
		) {
			data.additional_categories = additionalCategories
		}

		await patchProject(data)
	},
)

const categoryLists = computed(() => {
	const lists: Record<string, Category[]> = {}
	sortedCategories(tags.value).forEach((x: Category) => {
		if (x.project_type === project.value.actualProjectType) {
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
	const tagCount = current.value.selectedTags.length
	if (tagCount > 8) {
		return `You've selected ${tagCount} tags. Consider reducing to 8 or fewer to keep your project focused and easier to discover.`
	}
	return null
})

const multipleResolutionTagsWarning = computed(() => {
	if (project.value.actualProjectType !== 'resourcepack') return null

	const resolutionTags = current.value.selectedTags.filter((tag) =>
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
		(x: Category) => x.project_type === project.value.actualProjectType,
	)
	const totalSelectedTags = current.value.selectedTags.length

	if (
		totalSelectedTags === categoriesForProjectType.length &&
		categoriesForProjectType.length > 0
	) {
		return `You've selected all ${categoriesForProjectType.length} available tags. Please select only the tags that truly apply to your project.`
	}
	return null
})

const toggleCategory = (category: Category) => {
	if (current.value.selectedTags.includes(category)) {
		current.value.selectedTags = current.value.selectedTags.filter((x) => x !== category)
		if (current.value.featuredTags.includes(category)) {
			current.value.featuredTags = current.value.featuredTags.filter((x) => x !== category)
		}
	} else {
		current.value.selectedTags = [...current.value.selectedTags, category]
	}
}

const toggleFeaturedCategory = (category: Category) => {
	if (current.value.featuredTags.includes(category)) {
		current.value.featuredTags = current.value.featuredTags.filter((x) => x !== category)
	} else {
		current.value.featuredTags = [...current.value.featuredTags, category]
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
				width: 1rem;
				margin-right: 0.25rem;
				display: flex;
				align-items: center;

				svg {
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
