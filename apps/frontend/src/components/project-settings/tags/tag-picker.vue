<script setup lang="ts">
import { CheckIcon, SERVER_CATEGORY_ICON_MAP, StarIcon } from '@modrinth/assets'
import { Button, formatCategory, FormattedTag, TagIcon, useVIntl } from '@modrinth/ui'

import type { Category, CategoryGroup } from './types'

defineProps<{
	groups: CategoryGroup[]
	selectedTags: string[]
	featuredTags: string[]
	disabled: boolean
	limitReached: boolean
}>()
const emit = defineEmits<{ toggle: [tag: string] }>()
const { formatMessage } = useVIntl()

function categoryIconName(category: Category) {
	return category.project_type === 'minecraft_java_server'
		? (SERVER_CATEGORY_ICON_MAP[category.name] ?? category.name)
		: category.name
}
</script>

<template>
	<div
		data-project-field="tags"
		data-project-field-scroll-only
		class="settings-content-card flex flex-col gap-6 overflow-y-auto p-6"
	>
		<section v-for="group in groups" :key="group.id" :aria-labelledby="`tags-${group.id}`">
			<h3 :id="`tags-${group.id}`" class="m-0 mb-2.5 text-lg font-semibold text-contrast">
				{{ group.title }}
			</h3>
			<p v-if="group.description" class="mb-3 mt-0 text-secondary">
				{{ group.description }}
			</p>
			<div class="flex flex-wrap gap-2">
				<Button
					v-for="category in group.categories"
					:key="`${group.id}-${category.name}`"
					class="aria-pressed:!bg-brand-highlight aria-[pressed=false]:!text-secondary aria-[pressed=false]:[&>svg]:!text-inherit"
					:aria-pressed="selectedTags.includes(category.name)"
					:aria-label="formatCategory(formatMessage, category.name)"
					:disabled="disabled || (limitReached && !selectedTags.includes(category.name))"
					:type="selectedTags.includes(category.name) ? 'outlined' : 'base'"
					:color="selectedTags.includes(category.name) ? 'brand' : undefined"
					@click="emit('toggle', category.name)"
				>
					<StarIcon v-if="featuredTags.includes(category.name)" aria-hidden="true" />
					<CheckIcon v-else-if="selectedTags.includes(category.name)" aria-hidden="true" />
					<TagIcon
						v-else
						:tag="categoryIconName(category)"
						enforce-type="category"
						aria-hidden="true"
					/>
					<FormattedTag :tag="category.name" enforce-type="category" />
				</Button>
			</div>
		</section>
	</div>
</template>
