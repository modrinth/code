<template>
	<div v-if="allTags.length > 0" class="flex flex-col gap-3">
		<h2 class="text-lg m-0">Tags</h2>
		<div class="flex flex-wrap gap-1">
			<TagItem v-for="tag in allTags" :key="tag">
				<FormattedTag :tag="tag" />
			</TagItem>
		</div>
	</div>
</template>
<script setup lang="ts">
import { computed } from 'vue'

import FormattedTag from '../base/FormattedTag.vue'
import TagItem from '../base/TagItem.vue'

const props = defineProps<{
	project: {
		categories: string[]
		additional_categories: string[]
	}
}>()

const allTags = computed(() => [...props.project.categories, ...props.project.additional_categories])
</script>
