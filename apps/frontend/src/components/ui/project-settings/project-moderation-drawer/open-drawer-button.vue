<script setup lang="ts">
import { ListTodoIcon, MessagesSquareIcon } from '@modrinth/assets'
import { defineMessages, useVIntl } from '@modrinth/ui'
import { computed, ref } from 'vue'

import {
	isProjectChecklistVisible,
	projectPublishingColors,
	type ProjectPublishingState,
	projectPublishingTones,
} from '~/providers/project-publishing'

const props = defineProps<{
	state: ProjectPublishingState
	remaining: number
	controls?: string
}>()
const emit = defineEmits<{ open: []; hover: [] }>()

const { formatMessage } = useVIntl()
const messages = defineMessages({
	openDrawer: {
		id: 'project.settings.review.open-drawer',
		defaultMessage:
			'Open moderation drawer{count, plural, =0 {} one { (# checklist item remaining)} other { (# checklist items remaining)}}',
	},
	openThread: {
		id: 'project.settings.review.open-thread',
		defaultMessage: 'Open moderation thread',
	},
})
const launcherColor = computed(() => projectPublishingColors[projectPublishingTones[props.state]])
const showChecklist = computed(() => isProjectChecklistVisible(props.state, props.remaining))

const trigger = ref<HTMLButtonElement>()

function onPointerMove(event: PointerEvent) {
	if (event.pointerType === 'mouse' && event.buttons === 0) emit('hover')
}

defineExpose({ element: trigger })
</script>

<template>
	<button
		ref="trigger"
		type="button"
		class="drawer-launcher flex cursor-pointer flex-col rounded-l-2xl border border-r-0 border-solid p-1 shadow-lg transition-[filter] hover:brightness-110 focus-visible:outline focus-visible:outline-brand motion-reduce:transition-none"
		:style="{ '--launcher-color': launcherColor ? `var(--color-${launcherColor})` : undefined }"
		:aria-label="
			showChecklist
				? formatMessage(messages.openDrawer, { count: remaining })
				: formatMessage(messages.openThread)
		"
		:aria-controls="controls"
		:aria-expanded="controls ? false : undefined"
		@pointermove="onPointerMove"
		@click="emit('open')"
	>
		<span v-if="showChecklist" class="relative grid size-12 place-items-center" aria-hidden="true">
			<ListTodoIcon class="size-6" />
			<span
				v-if="remaining > 0"
				class="absolute right-0 top-0 flex aspect-square h-4 min-w-4 items-center justify-center rounded-full text-[10px] leading-none"
				:class="
					launcherColor
						? 'bg-[--launcher-color] text-[var(--color-accent-contrast)]'
						: 'bg-surface-4 text-contrast'
				"
			>
				{{ remaining }}
			</span>
		</span>
		<span class="grid size-12 place-items-center" aria-hidden="true">
			<MessagesSquareIcon class="size-6" />
		</span>
	</button>
</template>

<style scoped>
.drawer-launcher {
	color: var(--launcher-color, var(--color-contrast));
	background-color: color-mix(
		in srgb,
		var(--launcher-color, var(--surface-2)) 14%,
		var(--surface-2)
	);
	border-color: color-mix(in srgb, var(--launcher-color, var(--surface-4)) 35%, var(--surface-4));
}
</style>
