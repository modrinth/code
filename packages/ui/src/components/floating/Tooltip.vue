<script setup lang="ts">
import { arrow, autoUpdate, flip, offset, shift, useFloating } from '@floating-ui/vue'
import {
	computed,
	defineComponent,
	onBeforeUnmount,
	type PropType,
	ref,
	useTemplateRef,
	type VNode,
	watch,
} from 'vue'

import {
	bindTooltipSource,
	preventHide,
	type TooltipBaseProps,
	type TooltipContent,
	tooltipEnter,
	tooltipFocusIn,
	tooltipFocusOut,
	tooltipLeave,
	unbindTooltipSource,
} from '../../providers/tooltip'

defineOptions({ inheritAttrs: false })

const TooltipSlot = defineComponent({
	props: {
		content: { type: Function as PropType<TooltipContent>, required: true },
	},
	setup(props) {
		return () => props.content()
	},
})

const SIDES = {
	top: { origin: 'center bottom', arrow: 'bottom', rotate: 45 },
	right: { origin: 'left center', arrow: 'left', rotate: 135 },
	bottom: { origin: 'center top', arrow: 'top', rotate: 225 },
	left: { origin: 'right center', arrow: 'right', rotate: 315 },
} as const

const props = withDefaults(
	defineProps<TooltipBaseProps & {
		disabled?: boolean
	}>(),
	{ disabled: false, theme: 'tooltip', placement: 'top', allowTransfer: true },
)

const slots = defineSlots<{ popper?: () => VNode }>()
const trigger = useTemplateRef<HTMLElement>('trigger')
const floating = useTemplateRef<HTMLElement>('floating')
const arrowEl = useTemplateRef<HTMLElement>('arrowEl')
const hosted = computed(() => props.reference !== undefined)
const isOpen = computed(
	() => props.reference != null && (!!props.text || props.content != null) && !props.disabled,
)
const referenceEl = computed(() => props.reference ?? null)
const moving = ref(false)
const jumpKey = ref(0)
let addedTabIndex = false

const { floatingStyles, middlewareData, placement, update } = useFloating(referenceEl, floating, {
	placement: () => props.placement,
	strategy: 'fixed',
	transform: false,
	whileElementsMounted: autoUpdate,
	open: isOpen,
	middleware: [offset(8), flip(), shift({ padding: 8 }), arrow({ element: arrowEl })],
})

watch(
	[referenceEl, () => props.placement, isOpen],
	() => {
		if (isOpen.value) {
			update()
		}
	},
	{ flush: 'post' },
)

const isDisabled = computed(() => props.disabled)

watch(isDisabled, (value) => {
	if (trigger.value) {
		if (value) {
			if (trigger.value.matches(':hover')) {
				onLeave()
			}
		} else {
			if (trigger.value.matches(':hover')) {
				onEnter()
			}
		}
	}
})

watch(
	referenceEl,
	(el, prev, onCleanup) => {
		if (!el || !prev || !isOpen.value) {
			return
		}
		const from = (floating.value ?? prev).getBoundingClientRect()
		const to = el.getBoundingClientRect()
		const dx = from.left + from.width / 2 - (to.left + to.width / 2)
		const dy = from.top + from.height / 2 - (to.top + to.height / 2)
		if (Math.hypot(dx, dy) > 300) {
			moving.value = false
			jumpKey.value++
			return
		}
		moving.value = true
		const timer = setTimeout(() => {
			moving.value = false
		}, 150)
		onCleanup(() => clearTimeout(timer))
	},
	{ flush: 'sync' },
)

watch(
	[
		trigger,
		() => props.disabled,
		() => props.pinned,
		() => props.placement,
		() => props.theme,
		() => props.text,
		() => props.panelClass,
		() => slots.popper,
	],
	() => {
		syncSource()
		syncFocusable()
	},
	{ flush: 'post', immediate: true },
)

onBeforeUnmount(() => {
	if (trigger.value) {
		unbindTooltipSource(trigger.value)
	}
})

const side = computed(() => (placement.value.split('-')[0] ?? 'top') as keyof typeof SIDES)
const transformOrigin = computed(() => SIDES[side.value].origin)
const arrowStyles = computed(() => {
	const { arrow: edge, rotate } = SIDES[side.value]
	const { x, y } = middlewareData.value.arrow ?? {}
	return {
		left: x != null ? `${x}px` : undefined,
		top: y != null ? `${y}px` : undefined,
		[edge]: '-5px',
		transform: `rotate(${rotate}deg)`,
	}
})

function syncSource() {
	if (hosted.value) {
		return
	}
	const el = trigger.value
	if (!el) {
		return
	}
	if (props.disabled) {
		unbindTooltipSource(el)
		return
	}
	bindTooltipSource(el, {
		...props,
		content: slots.popper ? () => slots.popper?.() : undefined
	})
}

function syncFocusable() {
	if (hosted.value) {
		return
	}
	const el = trigger.value
	if (!el) {
		return
	}
	const needsTabIndex =
		!props.disabled &&
		!el.querySelector(
			'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
		)
	if (needsTabIndex) {
		if (!el.hasAttribute('tabindex')) {
			el.tabIndex = 0
			addedTabIndex = true
		}
		return
	}
	if (!addedTabIndex) {
		return
	}
	el.removeAttribute('tabindex')
	addedTabIndex = false
}

function onEnter() {
	syncSource()
	if (props.disabled || !trigger.value) {
		return
	}
	const delay = props.delay
	tooltipEnter(
		trigger.value,
		delay ? (typeof delay === 'number' ? delay : delay.hover) : undefined,
	)
}

function onTooltipEnter() {
	console.log("WWWWWWWWWWWWWWWWWWWW")
	if ((props.hoverable || props.pinned) && props.reference) {
		console.log("WEEEEEEEE")
		preventHide(props.reference)
	}
}

function onLeave() {
	const triggerEl = trigger.value ?? props.reference
	const delay = props.delay
	if (triggerEl) {
		tooltipLeave(
			triggerEl,
			delay ? (typeof delay === 'number' ? delay : delay.unhover) : undefined,
		)
	}
}

function onFocusIn() {
	syncSource()
	if (props.disabled || !trigger.value) {
		return
	}
	tooltipFocusIn(trigger.value)
}

function onFocusOut(event: FocusEvent) {
	const el = trigger.value
	if (!el) {
		return
	}
	if (event.relatedTarget instanceof Node && el.contains(event.relatedTarget)) {
		return
	}
	tooltipFocusOut(el)
}
</script>
<template>
	<div
		v-if="!hosted"
		ref="trigger"
		v-bind="$attrs"
		@mouseenter="onEnter"
		@mouseleave="onLeave"
		@focusin="onFocusIn"
		@focusout="onFocusOut"
	>
		<slot />
	</div>
	<Teleport v-if="hosted" to="body">
		<Transition name="tooltip">
			<div
				v-if="isOpen"
				:key="jumpKey"
				ref="floating"
				class="v-popper__inner z-[100010] rounded-lg border border-solid border-surface-5 bg-surface-3 px-3 py-1.5 text-sm font-medium text-contrast card-shadow"
				:class="[
					`v-popper--theme-${theme}`,
					moving && 'tooltip-moving',
					panelClass,
					!!(hoverable || pinned) ? '' : 'pointer-events-none',
				]"
				:style="[floatingStyles, { transformOrigin }]"
				@mouseenter="onTooltipEnter"
				@mouseleave="onLeave"
			>
				<TooltipSlot v-if="content" :content="content" />
				<template v-else>{{ text }}</template>
				<div
					ref="arrowEl"
					class="tooltip-arrow absolute size-2 border-b border-r border-0 border-solid border-surface-5 bg-surface-3"
					:style="arrowStyles"
				/>
			</div>
		</Transition>
	</Teleport>
</template>
<style scoped>
.tooltip-moving,
.tooltip-moving .tooltip-arrow {
	transition:
		left 0.15s var(--ease-out-expo),
		top 0.15s var(--ease-out-expo);
}
</style>
