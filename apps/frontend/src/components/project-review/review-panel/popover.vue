<template>
	<Teleport to="body">
		<div
			:id="panelId"
			ref="element"
			role="dialog"
			:aria-label="formatMessage(messages.reviewSection, { section: anchor.label })"
			:data-review-panel="anchor.id"
			tabindex="-1"
			class="z-[99990] box-border w-[28rem] max-w-[calc(100vw-1rem)] overflow-y-auto rounded-xl border border-solid border-surface-5 bg-surface-3 p-4 text-sm text-primary"
			:style="floatingStyles"
			@pointerenter="cancelClose"
			@pointerleave="leave(anchor.id)"
		>
			<div class="flex items-center justify-between gap-3">
				<div class="text-lg font-semibold text-contrast">{{ anchor.label }}</div>
				<IconButton
					:label="formatMessage(messages.closeReview)"
					@click="dismiss(true)"
					type="quiet"
				>
					<XIcon />
				</IconButton>
			</div>
			<div class="space-y-4 break-words [&_a]:break-all">
				<slot />
			</div>
		</div>
	</Teleport>
</template>

<script setup lang="ts">
import { autoUpdate, flip, offset, shift, size, useFloating } from '@floating-ui/vue'
import { XIcon } from '@modrinth/assets'
import { IconButton, useVIntl } from '@modrinth/ui'
import { useEventListener } from '@vueuse/core'
import { computed, nextTick, onBeforeUnmount, shallowRef, watch } from 'vue'

import { projectReviewMessages as messages } from '../messages'
import { injectReviewContext, type ReviewAnchor } from './context'

const { formatMessage } = useVIntl()
const props = defineProps<{ anchor: ReviewAnchor }>()
const { active, panel, panelId, pinned, close, leave, cancelClose } = injectReviewContext()
const element = shallowRef<HTMLElement | null>(null)
const reference = computed(() => props.anchor.trigger ?? props.anchor.element)
const { floatingStyles, isPositioned } = useFloating(reference, element, {
	placement: 'right-start',
	strategy: 'fixed',
	whileElementsMounted: autoUpdate,
	middleware: [
		offset(10),
		flip(),
		shift({ padding: 8 }),
		size({
			padding: 8,
			apply: ({ availableHeight, elements }) => {
				elements.floating.style.maxHeight = `${Math.max(0, availableHeight)}px`
			},
		}),
	],
})

function dismiss(restoreFocus = false) {
	if (active.value?.id === props.anchor.id) close(restoreFocus)
}

watch(element, (value, previous) => {
	if (active.value?.id === props.anchor.id) panel.value = value
	else if (panel.value === previous) panel.value = null
})
onBeforeUnmount(() => {
	if (panel.value === element.value) panel.value = null
})

watch([isPositioned, pinned], () => {
	if (!isPositioned.value || !pinned.value) return
	void nextTick(() => {
		if (pinned.value && active.value?.id === props.anchor.id) element.value?.focus()
	})
})
useEventListener('pointerdown', (event) => {
	if (active.value?.id !== props.anchor.id || !(event.target instanceof Node)) return
	if (props.anchor.element.contains(event.target) || element.value?.contains(event.target)) return
	const focused = document.activeElement
	if (
		focused instanceof HTMLElement &&
		(props.anchor.element.contains(focused) || element.value?.contains(focused))
	) {
		focused.blur()
	}
	dismiss()
})
</script>
