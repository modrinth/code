<script setup lang="ts" generic="ItemType extends StackedAdmonitionItem">
import { ChevronDownIcon, XIcon } from '@modrinth/assets'
import { AnimatePresence, Motion } from 'motion-v'
import { computed, onBeforeUnmount, onMounted, ref, useId, watch } from 'vue'

import { defineMessages, useVIntl } from '../../composables/i18n'
import ButtonStyled from './ButtonStyled.vue'

export type StackedAdmonitionType = 'info' | 'warning' | 'critical' | 'success'

/** Extend this interface to attach arbitrary per-item data consumed in the #item slot. */
export interface StackedAdmonitionItem {
	id: string
	type: StackedAdmonitionType
}

const props = withDefaults(
	defineProps<{
		items: ItemType[]
		peek?: number
		hoverPeek?: number
		expandedGap?: number
		scaleStep?: number
		hoverScaleStep?: number
		maxVisibleBehind?: number
		dismissAllEnabled?: boolean
		expanded?: boolean
	}>(),
	{
		peek: 8,
		hoverPeek: 16,
		expandedGap: 12,
		scaleStep: 0.04,
		hoverScaleStep: 0.025,
		maxVisibleBehind: 2,
		dismissAllEnabled: true,
		expanded: undefined,
	},
)

const emit = defineEmits<{
	'dismiss-all': []
	'update:expanded': [value: boolean]
	expand: []
	collapse: []
}>()

defineSlots<{
	item(props: {
		item: ItemType
		index: number
		isFront: boolean
		expanded: boolean
		/**
		 * Whether the consumer should render the Admonition's own dismiss button.
		 * False when the stack header's "Dismiss"/"Dismiss all" covers the dismiss action
		 * (e.g. single-item stack with `dismissAllEnabled`).
		 */
		dismissible: boolean
	}): unknown
	'header-label'(props: { count: number; expanded: boolean }): unknown
}>()

const { formatMessage } = useVIntl()
const stackId = useId()

const internalExpanded = ref(false)
const isHovered = ref(false)
const prefersReducedMotion = ref(false)

const heights = ref<Record<string, number>>({})
const cardEls = new Map<string, HTMLElement>()
const observers = new Map<string, ResizeObserver>()
const pendingHeights = new Map<string, number>()
let flushHandle: number | null = null

function scheduleHeightFlush() {
	if (flushHandle != null) return
	flushHandle = requestAnimationFrame(() => {
		flushHandle = null
		if (pendingHeights.size === 0) return
		const next = { ...heights.value }
		let changed = false
		for (const [id, h] of pendingHeights) {
			if (next[id] !== h) {
				next[id] = h
				changed = true
			}
		}
		pendingHeights.clear()
		if (changed) heights.value = next
	})
}

const isExpanded = computed(() => {
	if (props.items.length <= 1) return false
	return props.expanded ?? internalExpanded.value
})

const frontCardHeight = computed(() => {
	const front = props.items[0]
	return front ? (heights.value[front.id] ?? 0) : 0
})

const hasBehind = computed(() => props.items.length > 1)

const containerHeight = computed(() => {
	if (isExpanded.value) {
		return props.items.reduce(
			(acc, it, i) => acc + (heights.value[it.id] ?? 0) + (i > 0 ? props.expandedGap : 0),
			0,
		)
	}
	if (!hasBehind.value) return frontCardHeight.value
	const peek = isHovered.value ? props.hoverPeek : props.peek
	const behind = Math.min(props.items.length - 1, props.maxVisibleBehind)
	const pad = isHovered.value ? 6 : 0
	return frontCardHeight.value + peek * behind + pad
})

const springTransition = computed(() =>
	prefersReducedMotion.value
		? { duration: 0 }
		: { type: 'spring' as const, stiffness: 260, damping: 32 },
)

const exitTransition = computed(() =>
	prefersReducedMotion.value ? { duration: 0 } : { duration: 0.18 },
)

function cardPosition(index: number) {
	if (isExpanded.value) {
		let y = 0
		for (let i = 0; i < index; i++) {
			y += (heights.value[props.items[i].id] ?? 0) + props.expandedGap
		}
		return { y, scale: 1, opacity: 1 }
	}
	const peek = isHovered.value ? props.hoverPeek : props.peek
	const step = isHovered.value ? props.hoverScaleStep : props.scaleStep
	const hidden = index > props.maxVisibleBehind
	return {
		y: index * peek,
		scale: Math.max(0.8, 1 - index * step),
		opacity: hidden ? 0 : 1,
	}
}

function resolveNode(el: unknown): HTMLElement | null {
	if (!el) return null
	if (el instanceof HTMLElement) return el
	if (typeof el === 'object' && '$el' in el) {
		const node = (el as { $el: unknown }).$el
		return node instanceof HTMLElement ? node : null
	}
	return null
}

function setCardRef(id: string, el: unknown) {
	const node = resolveNode(el)
	if (!node) return
	if (cardEls.get(id) === node) return
	observers.get(id)?.disconnect()
	cardEls.set(id, node)
	const ro = new ResizeObserver(() => {
		pendingHeights.set(id, node.offsetHeight)
		scheduleHeightFlush()
	})
	ro.observe(node)
	observers.set(id, ro)
}

function setExpanded(v: boolean) {
	internalExpanded.value = v
	emit('update:expanded', v)
	if (v) emit('expand')
	else emit('collapse')
}

function toggleExpanded() {
	if (props.items.length <= 1) return
	setExpanded(!isExpanded.value)
}

function isInteractiveTarget(target: HTMLElement | null, currentTarget: EventTarget | null) {
	if (!target) return false
	const interactive = target.closest(
		'button, a, input, select, textarea, summary, [role="button"], [role="link"]',
	)
	return !!interactive && interactive !== currentTarget
}

function onContainerClick(e: MouseEvent) {
	if (isExpanded.value || props.items.length <= 1) return
	const target = e.target as HTMLElement | null
	if (isInteractiveTarget(target, e.currentTarget)) return
	setExpanded(true)
}

function onCardClick(e: MouseEvent) {
	if (!isExpanded.value) return
	const target = e.target as HTMLElement | null
	if (isInteractiveTarget(target, e.currentTarget)) return
	e.stopPropagation()
	setExpanded(false)
}

watch(
	() => props.items.length,
	(n) => {
		if (n <= 1 && internalExpanded.value) setExpanded(false)
	},
)

watch(
	() => props.items.map((i) => i.id),
	(ids) => {
		const idSet = new Set(ids)
		for (const [id, ro] of observers) {
			if (!idSet.has(id)) {
				ro.disconnect()
				observers.delete(id)
				cardEls.delete(id)
			}
		}
		const next: Record<string, number> = {}
		for (const id of idSet) {
			if (heights.value[id] != null) next[id] = heights.value[id]
		}
		heights.value = next
	},
)

let mql: MediaQueryList | null = null
function syncRM(e: MediaQueryListEvent | MediaQueryList) {
	prefersReducedMotion.value = 'matches' in e ? e.matches : false
}

onMounted(() => {
	if (typeof window === 'undefined' || !window.matchMedia) return
	mql = window.matchMedia('(prefers-reduced-motion: reduce)')
	prefersReducedMotion.value = mql.matches
	mql.addEventListener('change', syncRM)
})

onBeforeUnmount(() => {
	mql?.removeEventListener('change', syncRM)
	for (const ro of observers.values()) ro.disconnect()
	observers.clear()
	cardEls.clear()
	pendingHeights.clear()
	if (flushHandle != null) cancelAnimationFrame(flushHandle)
})

const placeholderClasses: Record<StackedAdmonitionType, string> = {
	info: 'border-brand-blue bg-bg-blue',
	warning: 'border-brand-orange bg-bg-orange',
	critical: 'border-brand-red bg-bg-red',
	success: 'border-brand-green bg-bg-green',
}

const messages = defineMessages({
	alertCount: {
		id: 'ui.stacked-admonitions.alert-count',
		defaultMessage: '{count, plural, one {# alert} other {# alerts}}',
	},
	dismissAll: {
		id: 'ui.stacked-admonitions.dismiss-all',
		defaultMessage: 'Dismiss all',
	},
	dismiss: {
		id: 'ui.stacked-admonitions.dismiss',
		defaultMessage: 'Dismiss',
	},
})
</script>

<template>
	<div v-if="items.length > 0" class="relative">
		<div v-if="items.length >= 1" class="mb-2 flex items-center justify-between">
			<ButtonStyled v-if="items.length >= 2" type="transparent" hover-color-fill="none">
				<button
					type="button"
					:aria-expanded="isExpanded"
					:aria-controls="stackId"
					@click="toggleExpanded"
				>
					<Motion
						as="span"
						class="inline-flex"
						:animate="{ rotate: isExpanded ? 0 : -90 }"
						:transition="{ type: 'spring', stiffness: 350, damping: 30 }"
					>
						<ChevronDownIcon class="h-4 w-4" />
					</Motion>
					<slot name="header-label" :count="items.length" :expanded="isExpanded">
						{{ formatMessage(messages.alertCount, { count: items.length }) }}
					</slot>
				</button>
			</ButtonStyled>
			<span v-else class="px-3 py-1.5 text-sm font-medium text-secondary">
				<slot name="header-label" :count="items.length" :expanded="isExpanded">
					{{ formatMessage(messages.alertCount, { count: items.length }) }}
				</slot>
			</span>
			<ButtonStyled v-if="dismissAllEnabled" type="transparent" hover-color-fill="none">
				<button type="button" @click="$emit('dismiss-all')">
					<XIcon class="h-4 w-4" />
					{{
						items.length >= 2 ? formatMessage(messages.dismissAll) : formatMessage(messages.dismiss)
					}}
				</button>
			</ButtonStyled>
		</div>

		<Motion
			:id="stackId"
			as="div"
			class="relative"
			:animate="{ height: containerHeight }"
			:transition="springTransition"
			@mouseenter="isHovered = true"
			@mouseleave="isHovered = false"
			@click="onContainerClick"
		>
			<AnimatePresence :initial="false">
				<Motion
					v-for="(item, index) in items"
					:key="item.id"
					:ref="(el: unknown) => setCardRef(item.id, el)"
					as="div"
					class="absolute inset-x-0 top-0 rounded-2xl bg-bg will-change-transform"
					:initial="{
						opacity: 0,
						y: cardPosition(index).y - 8,
						scale: cardPosition(index).scale,
					}"
					:animate="cardPosition(index)"
					:exit="{ opacity: 0, scale: 0.9, transition: exitTransition }"
					:transition="springTransition"
					:style="{
						zIndex: items.length - index,
						transformOrigin: 'top center',
					}"
					:aria-hidden="!isExpanded && index !== 0 ? 'true' : undefined"
					@click="onCardClick"
				>
					<template v-if="isExpanded || index === 0">
						<slot
							name="item"
							:item="item"
							:index="index"
							:is-front="index === 0"
							:expanded="isExpanded"
							:dismissible="!dismissAllEnabled"
						/>
					</template>
					<div
						v-else
						:class="['rounded-2xl border border-solid', placeholderClasses[item.type]]"
						:style="{ height: `${frontCardHeight}px` }"
						aria-hidden="true"
					/>
				</Motion>
			</AnimatePresence>
		</Motion>
	</div>
</template>
