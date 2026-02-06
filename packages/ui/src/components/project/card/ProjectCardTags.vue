<script setup lang="ts">
import { computed } from 'vue'

import { getTagMessage, sortTagsForDisplay } from '../../../utils'
import { TagTagItem } from '../../base'
import TagsOverflow from '../TagsOverflow.vue'

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
</script>

<template>
	<TagTagItem
		v-for="tag in visibleTags"
		:key="'visible-tag-' + tag"
		hide-non-loader-icon
		:tag="tag"
	/>
	<TagsOverflow
		v-if="overflowTags"
		:tags="overflowTags"
		class="smart-clickable:allow-pointer-events"
	/>
</template>
