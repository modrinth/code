<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { StarIcon } from '@modrinth/assets'
import {
	Button,
	defineMessages,
	formatCategory,
	FormattedTag,
	ProjectCard,
	useVIntl,
} from '@modrinth/ui'

defineProps<{
	project: Labrinth.Projects.v2.Project
	selectedTags: string[]
	featuredTags: string[]
	maxFeaturedTags: number
	disabled: boolean
}>()
const emit = defineEmits<{ toggle: [tag: string] }>()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	featuredTags: {
		id: 'project.settings.tags.featured-tags',
		defaultMessage: 'Featured tags',
	},
	featuredTagsDescription: {
		id: 'project.settings.tags.featured-tags-select-description',
		defaultMessage: 'Shown on your project’s card in Discover search results. Choose up to 3.',
	},
	selectAtLeastOneCategory: {
		id: 'project.settings.tags.select-at-least-one-category',
		defaultMessage: 'Select at least one category to feature.',
	},
	featureTag: { id: 'project.settings.tags.feature-tag', defaultMessage: 'Feature {tag}' },
})
</script>

<template>
	<section>
		<h3 class="settings-section-title !mb-2.5">
			{{ formatMessage(messages.featuredTags) }}
		</h3>
		<p class="mb-3 mt-0">
			{{ formatMessage(messages.featuredTagsDescription) }}
		</p>
		<div class="settings-content-card flex flex-col gap-4">
			<div class="min-h-10">
				<p v-if="selectedTags.length < 1" class="m-0 my-2 text-secondary">
					{{ formatMessage(messages.selectAtLeastOneCategory) }}
				</p>
				<div v-else class="flex flex-wrap gap-2">
					<Button
						v-for="name in selectedTags"
						:key="`featured-${name}`"
							class="aria-pressed:!bg-brand-highlight aria-[pressed=false]:!text-secondary aria-[pressed=false]:[&>svg]:!text-inherit"
						:aria-pressed="featuredTags.includes(name)"
						:aria-label="
							formatMessage(messages.featureTag, { tag: formatCategory(formatMessage, name) })
						"
						:type="featuredTags.includes(name) ? 'outlined' : 'base'"
						:color="featuredTags.includes(name) ? 'brand' : undefined"
						:disabled="
							disabled || (!featuredTags.includes(name) && featuredTags.length >= maxFeaturedTags)
						"
						@click="emit('toggle', name)"
						><StarIcon aria-hidden="true" /><FormattedTag :tag="name" enforce-type="category"
					/></Button>
				</div>
			</div>
			<ProjectCard
				class="[&>div]:bg-surface-2"
				layout="list"
				:title="project.title"
				:summary="project.description"
				:icon-url="project.icon_url ?? undefined"
				:tags="featuredTags"
				:downloads="project.downloads"
				:followers="project.followers"
			/>
		</div>
	</section>
</template>
