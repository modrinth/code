<script setup lang="ts">
import { useDroppable } from '@dnd-kit/vue'
import { ref } from 'vue'

import type { ContentTableSection } from './types'

const props = defineProps<{
	section: ContentTableSection
	dropActive: boolean
	droppable: boolean
}>()

const element = ref<HTMLElement | null>(null)

useDroppable({
	id: () => `content-section-empty:${props.section.id}`,
	element,
	disabled: () => !props.droppable,
	data: () => ({ sectionId: props.section.id }),
})
</script>

<template>
	<div
		ref="element"
		role="row"
		class="flex items-center border-0 border-t border-solid border-surface-4 px-4 text-secondary transition-colors duration-150"
		:class="dropActive ? 'bg-surface-3' : 'bg-surface-2'"
	>
		{{ section.emptyLabel }}
	</div>
</template>
