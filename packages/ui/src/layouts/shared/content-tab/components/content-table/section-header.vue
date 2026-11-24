<script setup lang="ts">
import { useDroppable } from '@dnd-kit/vue'
import { DropdownIcon } from '@modrinth/assets'
import { ref } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'

import type { ContentTableSection } from './types'

const props = defineProps<{
	section: ContentTableSection
	dropActive: boolean
	droppable: boolean
}>()

const emit = defineEmits<{
	toggle: []
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	collapse: { id: 'content.table.section.collapse', defaultMessage: 'Collapse group' },
	expand: { id: 'content.table.section.expand', defaultMessage: 'Expand group' },
})

const element = ref<HTMLElement | null>(null)

useDroppable({
	id: () => `content-section:${props.section.id}`,
	element,
	disabled: () => !props.droppable,
	data: () => ({ sectionId: props.section.id }),
})

function toggle() {
	emit('toggle')
}
</script>

<template>
	<div
		ref="element"
		role="row"
		class="group/section-header flex h-12 items-center gap-1.5 border-0 border-t border-solid border-surface-4 px-4 transition-colors duration-150"
		:class="dropActive ? 'bg-surface-3' : ''"
	>
		<slot :toggle="toggle" :collapsed="!!section.collapsed">
			<button
				type="button"
				class="flex shrink-0 cursor-pointer items-center border-0 bg-transparent p-0 text-secondary"
				:aria-expanded="!section.collapsed"
				:aria-label="formatMessage(section.collapsed ? messages.expand : messages.collapse)"
				@click="toggle"
			>
				<DropdownIcon
					aria-hidden="true"
					class="size-5 transition-transform duration-200"
					:class="{ 'rotate-180': !section.collapsed }"
				/>
			</button>
			<span role="rowheader" class="truncate font-semibold text-primary">
				{{ section.label }}
			</span>
		</slot>
	</div>
</template>
