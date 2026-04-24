<script setup lang="ts" generic="ItemType extends StackedAdmonitionItem">
import { ChevronDownIcon, XIcon } from '@modrinth/assets'
import { AnimatePresence, Motion } from 'motion-v'
import { computed, onBeforeUnmount, onMounted, ref, useAttrs, useId, watch } from 'vue'

import { defineMessages, useVIntl } from '../../composables/i18n'
import ButtonStyled from './ButtonStyled.vue'

defineOptions({
	inheritAttrs: false,
})

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
		/** Whether the consumer should render the Admonition's own dismiss button. */
		dismissible: boolean
	}): unknown
	'header-label'(props: { count: number; expanded: boolean }): unknown
}>()

const { formatMessage } = useVIntl()
const stackId = useId()
const attrs = useAttrs()

const internalExpanded = ref(false)
const isHovered = ref(false)
const prefersReducedMotion = ref(false)
const initialMeasurementSettled = ref(false)
const enteringItemIds = ref<Set<string>>(new Set())
const actionBarHeight = ref(0)

const heights = ref<Record<string, number>>({})
const cardEls = new Map<string, HTMLElement>()
const observers = new Map<string, ResizeObserver>()
const pendingHeights = new Map<string, number>()
let flushHandle: number | null = null
let initialMeasurementHandle: number | null = null
let enteringHandle: number | null = null
let actionBarObserver: ResizeObserver | null = null

// Slot content may run effects, so measure the one real tree instead of mounting
// hidden duplicates just to discover natural card heights.
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
		if (changed) {
			heights.value = next
			if (!initialMeasurementSettled.value && initialMeasurementHandle == null) {
				initialMeasurementHandle = requestAnimationFrame(() => {
					initialMeasurementHandle = null
					initialMeasurementSettled.value = true
				})
			}
		}
	})
}

const isExpanded = computed(() => {
	if (props.items.length <= 1) return false
	return props.expanded ?? internalExpanded.value
})
const hasActionBar = computed(() => props.items.length >= 2)
const itemDismissible = computed(() => true)

type StackPhase = 'collapsed' | 'expanding' | 'expanded' | 'collapsing'

const phase = ref<StackPhase>(isExpanded.value ? 'expanded' : 'collapsed')
const isSettledCollapsed = computed(() => phase.value === 'collapsed')
const containerHeightSettled = ref(true)
const singleItemEntrance = ref(false)

// Behind cards morph between a collapsed placeholder and real content. The shell
// height owns that morph so mixed-height cards do not swap DOM midway through motion.
function measuredCardHeight(index: number) {
	const item = props.items[index]
	return item ? (heights.value[item.id] ?? 0) : 0
}

function hasMeasuredCard(index: number) {
	const item = props.items[index]
	return !!item && heights.value[item.id] != null
}

const frontCardHeight = computed(() => measuredCardHeight(0))

const hasBehind = computed(() => props.items.length > 1)

function currentPeek() {
	return isHovered.value ? props.hoverPeek : props.peek
}

function currentScaleStep() {
	return isHovered.value ? props.hoverScaleStep : props.scaleStep
}

function targetCardHeight(index: number) {
	if (index === 0) return measuredCardHeight(0)

	const measured = measuredCardHeight(index) || frontCardHeight.value
	return isExpanded.value ? measured : frontCardHeight.value
}

const containerHeight = computed(() => {
	if (isExpanded.value) {
		return props.items.reduce((acc, _, i) => {
			return acc + measuredCardHeight(i) + (i > 0 ? props.expandedGap : 0)
		}, 0)
	}
	if (!hasBehind.value) return frontCardHeight.value
	const behind = Math.min(props.items.length - 1, props.maxVisibleBehind)
	const pad = isHovered.value ? 6 : 0
	return frontCardHeight.value + currentPeek() * behind + pad
})

const stackShellHeight = computed(() => {
	return containerHeight.value + (hasActionBar.value ? actionBarHeight.value : 0)
})
const containerOverflow = computed(() => {
	if (isExpanded.value) return 'visible'
	if (!containerHeightSettled.value) return 'hidden'
	if (!hasBehind.value && hasMeasuredCard(0)) return 'visible'
	return 'hidden'
})

const springTransition = computed(() =>
	prefersReducedMotion.value || !initialMeasurementSettled.value
		? { duration: 0 }
		: { type: 'spring' as const, stiffness: 260, damping: 32 },
)
const heightTransition = computed(() =>
	singleItemEntrance.value ? { duration: 0.12, ease: 'easeOut' as const } : springTransition.value,
)

const exitTransition = computed(() =>
	prefersReducedMotion.value ? { duration: 0 } : { duration: 0.18 },
)

const shellExitTransition = computed(() =>
	prefersReducedMotion.value ? { duration: 0 } : { duration: 0.16 },
)

function collapsedCardPosition(index: number) {
	const hidden = index > props.maxVisibleBehind
	return {
		y: index * currentPeek(),
		scale: Math.max(0.8, 1 - index * currentScaleStep()),
		opacity: hidden ? 0 : 1,
	}
}

function expandedCardPosition(index: number) {
	let y = 0
	for (let i = 0; i < index; i++) {
		y += measuredCardHeight(i) + props.expandedGap
	}
	return { y, scale: 1, opacity: 1 }
}

function cardPosition(index: number) {
	const position = isExpanded.value ? expandedCardPosition(index) : collapsedCardPosition(index)
	const item = props.items[index]
	if (index === 0 && singleItemEntrance.value) {
		return {
			...position,
			opacity: 0,
		}
	}
	if (!item || !enteringItemIds.value.has(item.id)) return position

	return {
		...position,
		y: position.y + 8,
		opacity: 0,
		scale: Math.min(1, position.scale + 0.02),
	}
}

function contentOpacity(index: number) {
	return isExpanded.value && hasMeasuredCard(index) ? 1 : 0
}

// Newly inserted cards need an explicit two-frame enter target because Motion's
// initial state is disabled to avoid animating from zero-height on first mount.
function markEntering(ids: string[]) {
	if (!initialMeasurementSettled.value || prefersReducedMotion.value || ids.length === 0) return

	const next = new Set(enteringItemIds.value)
	for (const id of ids) next.add(id)
	enteringItemIds.value = next

	if (enteringHandle != null) cancelAnimationFrame(enteringHandle)
	enteringHandle = requestAnimationFrame(() => {
		enteringHandle = requestAnimationFrame(() => {
			enteringHandle = null
			enteringItemIds.value = new Set()
		})
	})
}

function onContainerAnimationComplete() {
	phase.value = isExpanded.value ? 'expanded' : 'collapsed'
	containerHeightSettled.value = true
	if (containerHeight.value > 0) {
		singleItemEntrance.value = false
	}
}

const containerMotionProps = computed(() => ({
	onAnimationComplete: onContainerAnimationComplete,
}))

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
	pendingHeights.set(id, node.offsetHeight)
	scheduleHeightFlush()
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

function setActionBarRef(el: unknown) {
	const node = resolveNode(el)
	actionBarObserver?.disconnect()
	actionBarObserver = null
	if (!node) {
		actionBarHeight.value = 0
		return
	}

	actionBarHeight.value = node.offsetHeight
	const ro = new ResizeObserver(() => {
		actionBarHeight.value = node.offsetHeight
	})
	ro.observe(node)
	actionBarObserver = ro
}

function setExpanded(v: boolean) {
	internalExpanded.value = v
	emit('update:expanded', v)
	if (v) emit('expand')
	else emit('collapse')
}

function openStack() {
	if (props.items.length <= 1 || isExpanded.value) return
	phase.value = 'expanding'
	setExpanded(true)
}

function closeStack() {
	if (!isExpanded.value) return
	phase.value = 'collapsing'
	setExpanded(false)
}

function toggleExpanded() {
	if (props.items.length <= 1) return
	if (isExpanded.value) closeStack()
	else openStack()
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
	openStack()
}

function onCardClick(e: MouseEvent) {
	if (!isExpanded.value) return
	const target = e.target as HTMLElement | null
	if (isInteractiveTarget(target, e.currentTarget)) return
	e.stopPropagation()
	closeStack()
}

watch(
	() => props.items.length,
	(n, previousLength) => {
		if (previousLength === 0 && n === 1 && !prefersReducedMotion.value) {
			singleItemEntrance.value = true
		} else if (n !== 1) {
			singleItemEntrance.value = false
		}

		if (n <= 1 && (props.expanded ?? internalExpanded.value)) {
			phase.value = 'collapsed'
			setExpanded(false)
		}
	},
)

watch(isExpanded, (expanded, previousExpanded) => {
	if (previousExpanded === undefined) {
		phase.value = expanded ? 'expanded' : 'collapsed'
		return
	}
	if (expanded && phase.value !== 'expanding') phase.value = 'expanding'
	else if (!expanded && phase.value !== 'collapsing') phase.value = 'collapsing'
})

watch(containerHeight, (height, previousHeight) => {
	if (height !== previousHeight) {
		const openingSingleItem =
			previousHeight === 0 && height > 0 && props.items.length === 1 && !prefersReducedMotion.value

		if (openingSingleItem) {
			singleItemEntrance.value = true
		} else if (height === 0 || props.items.length !== 1) {
			singleItemEntrance.value = false
		}

		containerHeightSettled.value =
			prefersReducedMotion.value || (!initialMeasurementSettled.value && !openingSingleItem)
	}
})

watch(
	() => props.items.map((i) => i.id),
	(ids, previousIds = []) => {
		const idSet = new Set(ids)
		const previousIdSet = new Set(previousIds)
		markEntering(ids.filter((id) => !previousIdSet.has(id)))

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
	actionBarObserver?.disconnect()
	observers.clear()
	cardEls.clear()
	pendingHeights.clear()
	if (flushHandle != null) cancelAnimationFrame(flushHandle)
	if (initialMeasurementHandle != null) cancelAnimationFrame(initialMeasurementHandle)
	if (enteringHandle != null) cancelAnimationFrame(enteringHandle)
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
})
</script>

<template>
	<AnimatePresence :initial="false">
		<Motion
			v-if="items.length > 0"
			v-bind="attrs"
			as="div"
			class="relative"
			:initial="false"
			:animate="{ height: stackShellHeight, opacity: 1, y: 0 }"
			:exit="{
				height: 0,
				opacity: 0,
				overflow: 'hidden',
				y: -4,
				transition: shellExitTransition,
			}"
			:transition="heightTransition"
		>
			<Transition
				enter-active-class="overflow-hidden transition-all duration-150 ease-out"
				enter-from-class="-translate-y-1 opacity-0 max-h-0"
				enter-to-class="translate-y-0 opacity-100 max-h-14"
				leave-active-class="overflow-hidden transition-all duration-100 ease-in"
				leave-from-class="translate-y-0 opacity-100 max-h-14"
				leave-to-class="-translate-y-1 opacity-0 max-h-0"
			>
				<div v-if="hasActionBar" :ref="(el: unknown) => setActionBarRef(el)">
					<div class="flex items-center justify-between pb-2">
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
				</div>
			</Transition>

			<!-- Expanded-target overflow must become visible immediately so cards added
			during the tail of the expand spring do not inherit collapse clipping. -->
			<Motion
				:id="stackId"
				as="div"
				class="relative"
				:initial="false"
				:animate="{ height: containerHeight }"
				:transition="heightTransition"
				:style="{ overflow: containerOverflow }"
				v-bind="containerMotionProps"
				@mouseenter="isHovered = true"
				@mouseleave="isHovered = false"
				@click="onContainerClick"
			>
				<AnimatePresence :initial="false">
					<Motion
						v-for="(item, index) in items"
						:key="item.id"
						as="div"
						class="absolute inset-x-0 top-0 rounded-2xl bg-bg will-change-transform"
						:initial="false"
						:animate="cardPosition(index)"
						:exit="{ opacity: 0, scale: 0.9, transition: exitTransition }"
						:transition="springTransition"
						:style="{
							zIndex: items.length - index,
							transformOrigin: 'top center',
						}"
						:aria-hidden="isSettledCollapsed && index !== 0 ? 'true' : undefined"
						@click="onCardClick"
					>
						<template v-if="index === 0">
							<div :ref="(el: unknown) => setCardRef(item.id, el)">
								<slot
									name="item"
									:item="item"
									:index="index"
									:is-front="true"
									:expanded="isExpanded"
									:dismissible="itemDismissible"
								/>
							</div>
						</template>
						<template v-else>
							<div class="relative">
								<Motion
									as="div"
									:class="[
										'absolute inset-0 rounded-2xl border border-solid',
										placeholderClasses[item.type],
									]"
									:initial="false"
									:animate="{ opacity: isExpanded ? 0 : 1 }"
									:transition="springTransition"
									aria-hidden="true"
								/>
								<Motion
									as="div"
									:initial="false"
									:animate="{ height: targetCardHeight(index) }"
									:transition="springTransition"
									:style="{ overflow: isExpanded ? 'visible' : 'hidden' }"
								>
									<Motion
										as="div"
										:initial="false"
										:animate="{ opacity: contentOpacity(index) }"
										:transition="springTransition"
									>
										<div
											:ref="(el: unknown) => setCardRef(item.id, el)"
											:inert="!isExpanded ? true : undefined"
										>
											<slot
												name="item"
												:item="item"
												:index="index"
												:is-front="false"
												:expanded="isExpanded"
												:dismissible="itemDismissible"
											/>
										</div>
									</Motion>
								</Motion>
							</div>
						</template>
					</Motion>
				</AnimatePresence>
			</Motion>
		</Motion>
	</AnimatePresence>
</template>
