<script setup lang="ts" generic="T">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'

import { Button } from '#ui/components/base/buttons'

const props = withDefaults(
	defineProps<{
		items: T[]
		ariaLabel: string
		formatLabel?: (item: T) => string
		neutralItems?: T[]
		disabled?: boolean
		fullWidth?: boolean
		equalWidth?: boolean
	}>(),
	{
		formatLabel: (item: T) => String(item),
		neutralItems: () => [],
		disabled: false,
		fullWidth: false,
		equalWidth: false,
	},
)

const selected = defineModel<T>()

const STAGGER_DELAY = '175ms'

const container = ref<HTMLElement | null>(null)
const buttonElements: (HTMLElement | null)[] = []
const indicator = ref<{ left: number; right: number } | null>(null)
const delays = ref({ left: '0ms', right: '0ms' })
const transitionsEnabled = ref(false)

const layoutClass = computed(() => {
	if (props.equalWidth) return 'inline-grid w-fit auto-cols-fr grid-flow-col'
	return props.fullWidth ? 'flex w-full' : 'inline-flex'
})

const selectedIndex = computed(() => props.items.indexOf(selected.value as T))
const indicatorTone = computed(() =>
	props.neutralItems.includes(selected.value as T) ? 'neutral' : 'green',
)
const leftDelay = computed(() => delays.value.left)
const rightDelay = computed(() => delays.value.right)

function positionIndicator(animate: boolean) {
	const element = buttonElements[selectedIndex.value]
	const parent = container.value
	if (!element || !parent) {
		indicator.value = null
		return
	}

	const next = {
		left: element.offsetLeft,
		right: parent.clientWidth - element.offsetLeft - element.offsetWidth,
	}
	const movingLeft = indicator.value !== null && next.left < indicator.value.left
	delays.value = {
		left: movingLeft ? '0ms' : STAGGER_DELAY,
		right: movingLeft ? STAGGER_DELAY : '0ms',
	}

	if (!animate || indicator.value === null) {
		transitionsEnabled.value = false
		indicator.value = next
		requestAnimationFrame(() => {
			transitionsEnabled.value = true
		})
		return
	}

	indicator.value = next
}

watch(selectedIndex, () => nextTick(() => positionIndicator(true)))
watch(
	() => props.items,
	() => nextTick(() => positionIndicator(false)),
)

function setButtonRef(index: number, button: unknown) {
	buttonElements[index] = (button as { element: HTMLElement | null } | null)?.element ?? null
}

let resizeObserver: ResizeObserver | undefined

onMounted(() => {
	positionIndicator(false)
	resizeObserver = new ResizeObserver(() => positionIndicator(false))
	if (container.value) resizeObserver.observe(container.value)
})

onUnmounted(() => resizeObserver?.disconnect())
</script>

<template>
	<div
		ref="container"
		role="radiogroup"
		:aria-label="ariaLabel"
		class="relative h-9 items-center gap-0.5 rounded-xl border border-solid border-surface-5 p-px shadow-[0_1px_3px_rgba(0,0,0,0.15)]"
		:class="[layoutClass, { 'opacity-50': disabled }]"
	>
		<div
			v-if="indicator"
			class="indicator pointer-events-none absolute inset-y-px rounded-[10px]"
			:class="{ 'indicator-transition': transitionsEnabled }"
			:data-tone="indicatorTone"
			:style="{ left: `${indicator.left}px`, right: `${indicator.right}px` }"
			aria-hidden="true"
		/>
		<Button
			v-for="(item, index) in items"
			:key="String(item)"
			:ref="(button) => setButtonRef(index, button)"
			type="quiet"
			size="sm"
			role="radio"
			:aria-checked="index === selectedIndex"
			:interaction="index === selectedIndex ? 'none' : 'surface'"
			class="relative z-[1] !rounded-[10px] !px-3 !font-medium disabled:!opacity-100"
			:class="[
				index === selectedIndex && indicatorTone === 'green' ? '!text-green' : '!text-contrast',
				{ 'flex-1': fullWidth },
			]"
			:disabled="disabled"
			@click="selected = item"
		>
			{{ formatLabel(item) }}
		</Button>
	</div>
</template>

<style scoped>
.indicator[data-tone='green'] {
	background-color: color-mix(in srgb, var(--color-green) 30%, var(--surface-3));
	box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--color-green) 60%, transparent);
}

.indicator[data-tone='neutral'] {
	background-color: var(--surface-4);
	box-shadow: inset 0 0 0 1px var(--surface-5);
}

.indicator-transition {
	transition:
		left 80ms cubic-bezier(0.4, 0, 0.2, 1) v-bind(leftDelay),
		right 80ms cubic-bezier(0.4, 0, 0.2, 1) v-bind(rightDelay),
		background-color 100ms ease,
		box-shadow 100ms ease;
}
</style>
