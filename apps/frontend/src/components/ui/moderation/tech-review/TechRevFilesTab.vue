<script setup lang="ts">
import { computed } from 'vue'

import { getFileDetailCount } from './helpers'
import TechRevFileItem from './TechRevFileItem.vue'
import type { FlattenedFileReport, TechRevProjectRef } from './types'
import { injectTechReviewDecisions } from './use-tech-review-decisions'

const props = defineProps<{
	reports: FlattenedFileReport[]
	project: TechRevProjectRef
}>()

const emit = defineEmits<{
	viewFlags: [file: FlattenedFileReport]
}>()

const { getFileMarkedCount } = injectTechReviewDecisions()

const allFiles = computed(() => {
	return [...props.reports].sort((a, b) => {
		const aComplete = getFileMarkedCount(a) === getFileDetailCount(a)
		const bComplete = getFileMarkedCount(b) === getFileDetailCount(b)
		return aComplete === bComplete ? 0 : aComplete ? 1 : -1
	})
})
</script>

<template>
	<div>
		<TechRevFileItem
			v-for="file in allFiles"
			:key="file.id"
			class="bg-surface-2 last:rounded-bl-2xl last:rounded-br-2xl even:bg-surface-1.5"
			:file="file"
			:project="project"
			@view-flags="emit('viewFlags', $event)"
		/>
	</div>
</template>
