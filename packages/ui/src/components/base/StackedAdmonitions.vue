<script setup lang="ts" generic="ItemType extends StackedAdmonitionItem">
import { ChevronDownIcon, XIcon } from '@modrinth/assets'
import { AnimatePresence, Motion } from 'motion-v'
import { computed, onBeforeUnmount, onMounted, ref, useId, watch } from 'vue'

import { defineMessages, useVIntl } from '../../composables/i18n'
import ButtonStyled from './ButtonStyled.vue'

export type StackedAdmonitionType = 'info' | 'warning' | 'critical' | 'success'

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

const isExpanded = computed(() => {
	if (props.items.length <= 1) return false
	return props.expanded ?? internalExpanded.value
})

const collapsedInteractive = computed(() => !isExpanded.value && props.items.length >= 2)

const renderedItems = computed(() => props.items)

const frontCardHeight = computed(() => {
	const front = props.items[0]
	return front ? (heights.value[front.id] ?? 0) : 0
})

const containerHeight = computed(() => {
	if (isExpanded.value) {
		return props.items.reduce(
			(acc, it, i) => acc + (heights.value[it.id] ?? 0) + (i > 0 ? props.expandedGap : 0),
			0,
		)
	}
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
		const next = node.offsetHeight
		if (heights.value[id] !== next) {
			heights.value = { ...heights.value, [id]: next }
		}
	})
	ro.observe(node)
	observers.set(id, ro)
	heights.value = { ...heights.value, [id]: node.offsetHeight }
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

function onContainerClick(e: MouseEvent) {
	if (isExpanded.value || props.items.length <= 1) return
	const target = e.target as HTMLElement | null
	if (!target) return
	const interactive = target.closest('button, a, input, select, textarea, [role="button"]')
	if (interactive && interactive.id !== stackId) return
	setExpanded(true)
}

function onContainerKeydown(e: KeyboardEvent) {
	if (isExpanded.value || props.items.length <= 1) return
	if (e.key === 'Enter' || e.key === ' ') {
		e.preventDefault()
		setExpanded(true)
	}
}

watch(
	() => props.items.length,
	(n) => {
		if (n <= 1 && internalExpanded.value) setExpanded(false)
	},
)

watch(
	() => props.items.map((i) => i.id).join('|'),
	(joined) => {
		const ids = new Set(joined.split('|').filter(Boolean))
		for (const [id, ro] of observers) {
			if (!ids.has(id)) {
				ro.disconnect()
				observers.delete(id)
				cardEls.delete(id)
			}
		}
		const next: Record<string, number> = {}
		for (const id of ids) {
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
	expandAriaLabel: {
		id: 'ui.stacked-admonitions.expand-aria-label',
		defaultMessage: 'Expand {count, plural, one {# alert} other {# alerts}}',
	},
})
</script>

<template>
	<div v-if="items.length > 0" class="relative">
		<div v-if="items.length >= 2" class="mb-2 flex items-center justify-between">
			<ButtonStyled type="transparent">
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
			<ButtonStyled v-if="dismissAllEnabled" type="transparent">
				<button type="button" @click="$emit('dismiss-all')">
					<XIcon class="h-4 w-4" />
					{{ formatMessage(messages.dismissAll) }}
				</button>
			</ButtonStyled>
		</div>

		<Motion
			:id="stackId"
			as="div"
			class="relative"
			:animate="{ height: containerHeight }"
			:transition="springTransition"
			:role="collapsedInteractive ? 'button' : undefined"
			:tabindex="collapsedInteractive ? 0 : undefined"
			:aria-label="
				collapsedInteractive
					? formatMessage(messages.expandAriaLabel, { count: items.length })
					: undefined
			"
			@mouseenter="isHovered = true"
			@mouseleave="isHovered = false"
			@click="onContainerClick"
			@keydown="onContainerKeydown"
		>
			<AnimatePresence :initial="false">
				<Motion
					v-for="(item, index) in renderedItems"
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
					:exit="{ opacity: 0, scale: 0.9, transition: { duration: 0.18 } }"
					:transition="springTransition"
					:style="{
						zIndex: items.length - index,
						transformOrigin: 'top center',
					}"
					:inert="!isExpanded && index !== 0 ? true : undefined"
					:aria-hidden="!isExpanded && index !== 0 ? 'true' : undefined"
				>
					<template v-if="isExpanded || index === 0">
						<slot
							name="item"
							:item="item"
							:index="index"
							:is-front="index === 0"
							:expanded="isExpanded"
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
