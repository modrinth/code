<template>
	<div
		ref="container"
		class="flex h-full min-h-0 w-full min-w-0"
		:class="vertical ? 'flex-col' : 'flex-row'"
	>
		<template v-for="(member, index) in members" :key="member.id">
			<div
				v-if="index"
				role="separator"
				tabindex="0"
				:aria-orientation="vertical ? 'horizontal' : 'vertical'"
				:aria-label="formatMessage(messages.resize)"
				class="workbench-link-sash relative z-[1] shrink-0 touch-none bg-surface-5"
				:class="vertical ? 'h-px w-full cursor-row-resize' : 'h-full w-px cursor-col-resize'"
				@pointerdown="startResize(index, $event)"
				@pointermove="resize"
				@pointerup="stopResize"
				@pointercancel="stopResize"
				@dblclick="evenOut"
			/>
			<LinkedMember
				:member="member"
				class="min-h-0 min-w-0"
				:style="{ flex: `${shares[index] ?? 1} 1 0` }"
			/>
		</template>
	</div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'

import { injectWorkbench } from '../../providers/workbench'
import type { WorkbenchPanel } from '../../types'
import LinkedMember from './linked-member.vue'

/** A linked tab's content: its panels split along the link's orientation, resized by dragging. */
const props = defineProps<{ tab: WorkbenchPanel }>()

const { formatMessage } = useVIntl()
const workbench = injectWorkbench()

const messages = defineMessages({
	resize: { id: 'workbench.link.resize', defaultMessage: 'Resize linked panels' },
})

/** A panel's share can't shrink below this part of the tab. */
const MINIMUM_SHARE = 0.1

const members = computed(() => props.tab.members ?? [])
const link = computed(() =>
	workbench.links.value.find((candidate) => candidate.id === props.tab.id),
)
const vertical = computed(() => link.value?.orientation !== 'horizontal')

const container = ref<HTMLElement | null>(null)
const shares = ref<number[]>([])

watch(
	[() => link.value?.sizes, () => members.value.length],
	([sizes, count]) => {
		shares.value =
			sizes && sizes.length === count ? [...sizes] : Array.from({ length: count }, () => 1 / count)
	},
	{ immediate: true },
)

let drag: { index: number; start: number; extent: number; shares: number[] } | null = null

function position(event: PointerEvent) {
	return vertical.value ? event.clientY : event.clientX
}

function startResize(index: number, event: PointerEvent) {
	if (event.button !== 0 || !container.value) return
	event.preventDefault()
	const target = event.currentTarget as HTMLElement
	target.setPointerCapture(event.pointerId)
	const bounds = container.value.getBoundingClientRect()
	drag = {
		index,
		start: position(event),
		extent: vertical.value ? bounds.height : bounds.width,
		shares: [...shares.value],
	}
}

function resize(event: PointerEvent) {
	if (!drag || drag.extent <= 0) return
	const { index, shares: initial } = drag
	const total = initial[index - 1] + initial[index]
	const moved = ((position(event) - drag.start) / drag.extent) * initial.reduce((a, b) => a + b, 0)
	const before = Math.min(
		Math.max(initial[index - 1] + moved, MINIMUM_SHARE),
		total - MINIMUM_SHARE,
	)
	const next = [...initial]
	next[index - 1] = before
	next[index] = total - before
	shares.value = next
}

function stopResize(event: PointerEvent) {
	if (!drag) return
	drag = null
	const target = event.currentTarget as HTMLElement
	if (target.hasPointerCapture(event.pointerId)) target.releasePointerCapture(event.pointerId)
	workbench.setLinkSizes(props.tab.id, shares.value)
}

function evenOut() {
	const count = members.value.length
	workbench.setLinkSizes(
		props.tab.id,
		Array.from({ length: count }, () => 1 / count),
	)
}
</script>

<style scoped>
.workbench-link-sash::before {
	content: '';
	position: absolute;
	inset: -3px;
}

.workbench-link-sash:hover,
.workbench-link-sash:focus-visible {
	background-color: var(--color-brand);
}
</style>
