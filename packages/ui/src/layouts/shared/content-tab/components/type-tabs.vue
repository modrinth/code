<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonProjectTypeCategoryMessages } from '#ui/utils/common-messages'

import type { ContentType } from '../types'

const props = defineProps<{
	types: ContentType[]
}>()

const active = defineModel<ContentType>({ required: true })

const { formatMessage } = useVIntl()

const messages = defineMessages({
	label: {
		id: 'content.page-layout.type-tabs.label',
		defaultMessage: 'Content types',
	},
})

const STAGGER_DELAY = '100ms'

const tablist = ref<HTMLElement | null>(null)
const sliderReady = ref(false)
const transitionsEnabled = ref(false)
const slider = ref({ left: 0, top: 0, right: 0, bottom: 0 })
const sliderDelays = ref({ left: '0ms', top: '0ms', right: '0ms', bottom: '0ms' })

const activeIndex = computed(() => props.types.indexOf(active.value))

const sliderStyle = computed(() => ({
	left: `${slider.value.left}px`,
	top: `${slider.value.top}px`,
	right: `${slider.value.right}px`,
	bottom: `${slider.value.bottom}px`,
}))

const leftDelay = computed(() => sliderDelays.value.left)
const rightDelay = computed(() => sliderDelays.value.right)
const topDelay = computed(() => sliderDelays.value.top)
const bottomDelay = computed(() => sliderDelays.value.bottom)

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

function enableTransitionsNextFrame() {
	requestAnimationFrame(() => {
		transitionsEnabled.value = true
	})
}

function positionSlider(animate = true) {
	const position = getSliderPosition()
	if (!position) return

	if (!sliderReady.value || !animate) {
		transitionsEnabled.value = false
		slider.value = position
		sliderReady.value = true
		void nextTick(enableTransitionsNextFrame)
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
}

let resizeObserver: ResizeObserver | undefined

onMounted(() => {
	positionSlider(false)
	resizeObserver = new ResizeObserver(() => positionSlider(false))
	if (tablist.value) resizeObserver.observe(tablist.value)
})

onUnmounted(() => {
	resizeObserver?.disconnect()
})

watch(activeIndex, async () => {
	await nextTick()
	positionSlider()
})

watch(
	() => props.types,
	async () => {
		await nextTick()
		positionSlider(false)
	},
	{ deep: true },
)
</script>

<template>
	<div
		ref="tablist"
		role="tablist"
		:aria-label="formatMessage(messages.label)"
		class="relative flex min-w-0 items-center gap-1 overflow-x-auto rounded-[14px] border border-solid border-surface-5 p-[3px] shadow-[0_24px_48px_rgba(0,0,0,0.03),0_10px_18px_rgba(0,0,0,0.03),0_5px_8px_rgba(0,0,0,0.04),0_2px_4px_rgba(0,0,0,0.04)]"
		:class="{ 'type-tab-color-delayed': transitionsEnabled }"
	>
		<button
			v-for="type in types"
			:key="type"
			role="tab"
			type="button"
			:aria-selected="active === type"
			class="type-tab-color flex h-[34px] shrink-0 items-center whitespace-nowrap rounded-xl border border-solid border-transparent px-3 text-base font-semibold leading-5 focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-brand-shadow"
			:class="[
				active === type ? 'text-brand' : 'type-tab-hover text-primary',
				{ 'type-tab-active': active === type && !sliderReady },
			]"
			@click="active = type"
		>
			<span class="relative z-[1]">
				{{ formatMessage(commonProjectTypeCategoryMessages[type]) }}
			</span>
		</button>
		<div
			v-if="sliderReady && activeIndex !== -1"
			class="type-tab-active pointer-events-none absolute rounded-xl border border-solid"
			:class="{ 'type-tab-slider-transition': transitionsEnabled }"
			:style="sliderStyle"
			aria-hidden="true"
		/>
	</div>
</template>

<style scoped>
.type-tab-active {
	background-color: color-mix(in srgb, var(--color-brand) 30%, var(--surface-3));
	border-color: color-mix(in srgb, var(--color-brand) 60%, transparent);
}

.type-tab-hover:hover {
	background-color: color-mix(in srgb, var(--surface-4) 50%, transparent);
}

.type-tab-slider-transition {
	transition:
		left 200ms cubic-bezier(0.4, 0, 0.2, 1) v-bind(leftDelay),
		right 200ms cubic-bezier(0.4, 0, 0.2, 1) v-bind(rightDelay),
		top 200ms cubic-bezier(0.4, 0, 0.2, 1) v-bind(topDelay),
		bottom 200ms cubic-bezier(0.4, 0, 0.2, 1) v-bind(bottomDelay);
}

.type-tab-color {
	transition:
		color 150ms cubic-bezier(0.4, 0, 0.2, 1),
		background-color 100ms cubic-bezier(0.4, 0, 0.2, 1);
}

.type-tab-color-delayed .type-tab-color {
	transition-delay: 150ms, 0ms;
}
</style>
