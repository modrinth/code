<script setup lang="ts">
import { CheckIcon, StarIcon } from '@modrinth/assets'
import { Button, formatCategory, FormattedTag, TagIcon, useVIntl } from '@modrinth/ui'

import type { CategoryGroup } from './types'

defineProps<{
	groups: CategoryGroup[]
	selectedTags: string[]
	featuredTags: string[]
	disabled: boolean
}>()
const emit = defineEmits<{ toggle: [tag: string] }>()
const { formatMessage } = useVIntl()
</script>

<template>
	<div
		data-project-field="tags"
		data-project-field-scroll-only
		class="settings-content-card flex flex-col gap-6 overflow-y-auto p-6"
	>
		<section v-for="group in groups" :key="group.id" :aria-labelledby="`tags-${group.id}`">
			<div class="mb-3 flex items-center justify-between gap-2">
				<h3 :id="`tags-${group.id}`" class="m-0 text-base font-semibold text-contrast">
					{{ group.title }}
				</h3>
				<span class="text-xs text-secondary"
					>{{
						group.categories.filter((category) => selectedTags.includes(category.name)).length
					}}
					/ {{ group.categories.length }}</span
				>
			</div>
			<p v-if="group.description" class="mb-3 mt-0 text-secondary">
				{{ group.description }}
			</p>
			<div class="flex flex-wrap gap-2">
				<Button
					v-for="category in group.categories"
					:key="`${group.id}-${category.name}`"
					class="aria-pressed:!bg-brand-highlight"
					:aria-pressed="selectedTags.includes(category.name)"
					:aria-label="formatCategory(formatMessage, category.name)"
					:disabled="disabled"
					:type="selectedTags.includes(category.name) ? 'outlined' : 'base'"
					:color="selectedTags.includes(category.name) ? 'brand' : undefined"
					@click="emit('toggle', category.name)"
				>
					<StarIcon v-if="featuredTags.includes(category.name)" aria-hidden="true" />
					<CheckIcon v-else-if="selectedTags.includes(category.name)" aria-hidden="true" />
					<TagIcon v-else :tag="category.name" enforce-type="category" aria-hidden="true" />
					<FormattedTag :tag="category.name" enforce-type="category" />
				</Button>
			</div>
		</section>
	</div>
</template>
