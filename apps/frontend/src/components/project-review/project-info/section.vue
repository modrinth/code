<template>
	<component
		:is="target ? ReviewPanel : 'section'"
		v-bind="target ? { mode: 'anchored', as: 'section', target } : {}"
		:aria-label="heading"
		class="review-section"
		:class="{ 'editable-review-section': !!$slots.actions }"
	>
		<div class="flex items-center gap-1">
			<h3 class="m-0 text-sm font-normal text-secondary">{{ heading }}</h3>
			<slot name="actions" />
		</div>
		<slot />
	</component>
</template>

<script setup lang="ts">
import type { ReviewTarget } from '~/providers/project-review/review'

import ReviewPanel from '../review-panel/index.vue'

defineProps<{
	heading: string
	target?: ReviewTarget
}>()
</script>

<style scoped>
.review-section {
	@apply flex flex-col gap-2 border-0 border-t border-solid border-divider px-0.5 pb-3 pt-3;
}
</style>
