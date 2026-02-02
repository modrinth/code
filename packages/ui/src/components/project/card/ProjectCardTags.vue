<script setup lang="ts">
import { Menu } from 'floating-vue'
import { computed } from 'vue'

import { getTagMessage, sortTagsForDisplay } from '../../../utils'
import { TagItem, TagTagItem } from '../../base'

function isLoader(tag: string) {
	return getTagMessage(tag, 'loader') !== undefined
}

const props = withDefaults(
	defineProps<{
		tags: string[]
		extraTags?: string[]
		selectedTags?: string[]
		excludeLoaders?: boolean
		maxTags?: number
	}>(),
	{
		maxTags: 5,
		excludeLoaders: false,
		extraTags: () => [],
		selectedTags: () => [],
	},
)

const sortedTags = computed(() => (props.tags ? sortTagsForDisplay(props.tags) : undefined))
const sortedExtraTags = computed(() =>
	props.extraTags ? sortTagsForDisplay(props.extraTags) : undefined,
)
const filteredTags = computed(() => {
	if (!sortedTags.value) {
		return undefined
	}
	return sortedTags.value.filter(
		(tag) => !props.selectedTags.includes(tag) && (!props.excludeLoaders || !isLoader(tag)),
	)
})

const visibleTags = computed(() => filteredTags.value?.slice(0, props.maxTags))
const overflowTags = computed(() => [
	...(props.tags.filter((x) => !visibleTags.value?.includes(x)) ?? []),
	...(sortedExtraTags.value ?? []),
])
const overflowTagCount = computed(() => overflowTags.value?.length ?? 0)
</script>

<template>
	<TagTagItem
		v-for="tag in visibleTags"
		:key="'visible-tag-' + tag"
		hide-non-loader-icon
		:tag="tag"
	/>
	<Menu :delay="{ hide: 50, show: 0 }" no-auto-focus>
		<TagItem
			v-if="overflowTagCount > 0"
			class="project-card__overflow-pill smart-clickable:allow-pointer-events"
		>
			+{{ overflowTagCount }}
		</TagItem>
		<template #popper>
			<div class="flex gap-1 flex-wrap max-w-[20rem]">
				<TagTagItem
					v-for="tag in overflowTags"
					:key="'overflow-tag-' + tag"
					hide-non-loader-icon
					:tag="tag"
				/>
			</div>
		</template>
	</Menu>
</template>
