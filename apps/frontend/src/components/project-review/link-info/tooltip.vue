<template>
	<span
		ref="anchor"
		class="inline-flex w-fit min-w-0 max-w-full"
		@pointerenter="enterLink"
		@pointerleave="leaveLink"
		@pointerover.stop
		@pointermove.stop
		@focusin="show"
		@focusout="leaveFocus"
	>
		<slot :tooltip-id="open ? tooltipId : undefined" />
	</span>
	<Teleport v-if="open && anchor" :to="anchor.ownerDocument.body">
		<div
			:id="tooltipId"
			ref="panel"
			role="tooltip"
			:style="floatingStyles"
			class="z-[101] box-border w-96 max-w-[calc(100vw-1rem)] overflow-y-auto rounded-xl border border-solid border-surface-5 bg-surface-2 shadow-lg"
			@pointerenter="cancelClose"
			@pointerleave="scheduleClose"
			@pointerover.stop
			@pointermove.stop
			@focusin="cancelClose"
			@focusout="leaveFocus"
		>
			<LinkInfo :url="url" :enabled="hovered" :validation-messages="validationMessages" />
		</div>
	</Teleport>
</template>

<script setup lang="ts">
import { autoUpdate, flip, offset, shift, size, useFloating } from '@floating-ui/vue'
import { type FieldValidationMessage, moderationSettings } from '@modrinth/moderation'
import { computed, onScopeDispose, ref, useId, watch } from 'vue'

import { useModerationSettings } from '~/composables/moderation'

import { useReviewEventListener } from '../shortcuts'
import LinkInfo from './index.vue'

const props = defineProps<{ url: string; validationMessages?: FieldValidationMessage[] }>()
const emit = defineEmits<{ hover: [hovered: boolean] }>()
const settings = useModerationSettings()
const anchor = ref<HTMLElement>()
const panel = ref<HTMLElement>()
const open = ref(false)
const hovered = ref(false)
const tooltipId = useId()
let openTimer: ReturnType<typeof setTimeout> | undefined
let closeTimer: ReturnType<typeof setTimeout> | undefined

const { floatingStyles } = useFloating(anchor, panel, {
	placement: computed(() =>
		settings.value.get(moderationSettings.General.LinkInfoPlacement) === 'above'
			? 'top-start'
			: 'bottom-start',
	),
	strategy: 'fixed',
	whileElementsMounted: autoUpdate,
	middleware: [
		offset(8),
		flip({ padding: 8 }),
		shift({ padding: 8 }),
		size({
			padding: 8,
			apply: ({ availableHeight, elements }) => {
				elements.floating.style.maxHeight = `${Math.max(0, availableHeight)}px`
			},
		}),
	],
})

function cancelClose() {
	clearTimeout(closeTimer)
}

function show() {
	cancelClose()
	open.value = true
}

function enterLink(event: PointerEvent) {
	if (event.pointerType === 'touch') return
	hovered.value = true
	emit('hover', true)
	cancelClose()
	clearTimeout(openTimer)
	openTimer = setTimeout(show, 200)
}

function leaveLink() {
	hovered.value = false
	emit('hover', false)
	clearTimeout(openTimer)
	scheduleClose()
}

function contains(target: EventTarget | null) {
	if (!target || !('nodeType' in target)) return false
	return !!(anchor.value?.contains(target as Node) || panel.value?.contains(target as Node))
}

function leaveFocus(event: FocusEvent) {
	if (!contains(event.relatedTarget)) scheduleClose()
}

function scheduleClose() {
	cancelClose()
	closeTimer = setTimeout(() => {
		if (contains(anchor.value?.ownerDocument.activeElement ?? null)) return
		close()
	}, 200)
}

function close() {
	clearTimeout(openTimer)
	cancelClose()
	open.value = false
	hovered.value = false
	emit('hover', false)
}

useReviewEventListener('keydown', (event) => {
	if (!open.value || event.key !== 'Escape') return
	if ((event.target as Node | null)?.ownerDocument !== anchor.value?.ownerDocument) return
	event.preventDefault()
	event.stopPropagation()
	close()
})
useReviewEventListener('pointerdown', (event) => {
	if (open.value && !contains(event.target)) close()
})
watch(() => props.url, close)
onScopeDispose(close)
</script>
