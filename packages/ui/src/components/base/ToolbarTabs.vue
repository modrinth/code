<script setup lang="ts" generic="T extends string">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { type RouteLocationRaw, RouterLink } from 'vue-router'

export interface ToolbarTab<T extends string = string> {
	value: T
	label: string
	to?: RouteLocationRaw
}

const props = defineProps<{
	tabs: ToolbarTab<T>[]
	label: string
	replace?: boolean
}>()

const active = defineModel<T>()

const SLIDE_DURATION_MS = 200
const STAGGER_DELAY_MS = 100
const SLIDE_DURATION = `${SLIDE_DURATION_MS}ms`
const STAGGER_DELAY = `${STAGGER_DELAY_MS}ms`

const tablist = ref<HTMLElement | null>(null)
const sliderReady = ref(false)
const transitionsEnabled = ref(false)
const slider = ref({ left: 0, top: 0, right: 0, bottom: 0 })
const sliderDelays = ref({ left: '0ms', top: '0ms', right: '0ms', bottom: '0ms' })
const sliding = ref(false)

let slideEndTimeout: ReturnType<typeof setTimeout> | undefined

const activeIndex = computed(() => props.tabs.findIndex((tab) => tab.value === active.value))

const sliderStyle = computed(() => ({
	left: `${slider.value.left}px`,
	top: `${slider.value.top}px`,
	right: `${slider.value.right}px`,
	bottom: `${slider.value.bottom}px`,
}))

function getSliderPosition() {
	const container = tablist.value
	const tab = container?.querySelectorAll<HTMLElement>('[role="tab"]')[activeIndex.value]
	if (!container || !tab) return null

	return {
		left: tab.offsetLeft,
		top: tab.offsetTop,
		right: container.clientWidth - tab.offsetLeft - tab.offsetWidth,
		bottom: container.clientHeight - tab.offsetTop - tab.offsetHeight,
	}
}

function positionSlider(animate = true) {
	const position = getSliderPosition()
	if (!position) return

	clearTimeout(slideEndTimeout)

	if (!sliderReady.value || !animate) {
		sliding.value = false
		transitionsEnabled.value = false
		slider.value = position
		sliderReady.value = true
		void nextTick(() => requestAnimationFrame(() => (transitionsEnabled.value = true)))
		return
	}

	const movingLeft = position.left < slider.value.left
	const movingUp = position.top < slider.value.top
	sliderDelays.value = {
		left: movingLeft ? '0ms' : STAGGER_DELAY,
		right: movingLeft ? STAGGER_DELAY : '0ms',
		top: movingUp ? '0ms' : STAGGER_DELAY,
		bottom: movingUp ? STAGGER_DELAY : '0ms',
	}
	slider.value = position
	sliding.value = true
	slideEndTimeout = setTimeout(() => (sliding.value = false), SLIDE_DURATION_MS + STAGGER_DELAY_MS)
}

let resizeObserver: ResizeObserver | undefined

onMounted(() => {
	positionSlider(false)
	resizeObserver = new ResizeObserver(() => positionSlider(false))
	if (tablist.value) resizeObserver.observe(tablist.value)
})

onUnmounted(() => {
	resizeObserver?.disconnect()
	clearTimeout(slideEndTimeout)
})

watch(activeIndex, async () => {
	await nextTick()
	positionSlider()
})

watch(
	() => props.tabs.map((tab) => `${tab.value}:${tab.label}`).join('|'),
	async () => {
		await nextTick()
		positionSlider(false)
	},
)
</script>

<template>
	<div
		ref="tablist"
		role="tablist"
		:aria-label="label"
		class="relative flex min-w-0 items-center gap-1 overflow-x-auto rounded-[14px] border border-solid border-surface-5 p-[3px] shadow-[0_24px_48px_rgba(0,0,0,0.03),0_10px_18px_rgba(0,0,0,0.03),0_5px_8px_rgba(0,0,0,0.04),0_2px_4px_rgba(0,0,0,0.04)]"
		:class="{ 'toolbar-tab-color-delayed': transitionsEnabled }"
	>
		<component
			:is="tab.to ? RouterLink : 'button'"
			v-for="tab in tabs"
			:key="tab.value"
			role="tab"
			v-bind="tab.to ? { to: tab.to, replace } : { type: 'button' }"
			:aria-selected="active === tab.value"
			class="toolbar-tab-color no-click-animation relative z-[1] flex h-[34px] shrink-0 items-center whitespace-nowrap rounded-xl border border-solid border-transparent px-3 text-base font-semibold leading-5 focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-brand-shadow"
			:class="[
				active === tab.value ? 'text-brand' : ['text-primary', { 'toolbar-tab-hover': !sliding }],
				{ 'toolbar-tab-active': active === tab.value && !sliderReady },
			]"
			@click="active = tab.value"
		>
			{{ tab.label }}
		</component>
		<div
			v-if="sliderReady && activeIndex !== -1"
			class="toolbar-tab-active pointer-events-none absolute rounded-xl border border-solid"
			:class="{ 'toolbar-tab-slider-transition': transitionsEnabled }"
			:style="sliderStyle"
			aria-hidden="true"
		/>
	</div>
</template>

<style scoped>
.toolbar-tab-active {
	background-color: color-mix(in srgb, var(--color-brand) 30%, var(--surface-3));
	border-color: color-mix(in srgb, var(--color-brand) 60%, transparent);
}

.toolbar-tab-slider-transition {
	transition:
		left v-bind(SLIDE_DURATION) cubic-bezier(0.4, 0, 0.2, 1) v-bind('sliderDelays.left'),
		right v-bind(SLIDE_DURATION) cubic-bezier(0.4, 0, 0.2, 1) v-bind('sliderDelays.right'),
		top v-bind(SLIDE_DURATION) cubic-bezier(0.4, 0, 0.2, 1) v-bind('sliderDelays.top'),
		bottom v-bind(SLIDE_DURATION) cubic-bezier(0.4, 0, 0.2, 1) v-bind('sliderDelays.bottom');
}

.toolbar-tab-color {
	transition: color 150ms cubic-bezier(0.4, 0, 0.2, 1);
}

.toolbar-tab-hover {
	transition:
		color 150ms cubic-bezier(0.4, 0, 0.2, 1),
		background-color 100ms cubic-bezier(0.4, 0, 0.2, 1);
}

.toolbar-tab-hover:hover {
	background-color: color-mix(in srgb, var(--surface-4) 50%, transparent);
}

.toolbar-tab-color-delayed .toolbar-tab-color {
	transition-delay: 150ms, 0ms;
}
</style>
