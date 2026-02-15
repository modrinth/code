<template>
	<nav
		ref="scrollContainer"
		class="experimental-styles-within relative flex w-fit overflow-x-auto rounded-full bg-bg-raised p-1 text-sm font-bold"
		:class="{ 'card-shadow': mode === 'navigation' }"
	>
		<template v-if="mode === 'navigation'">
			<NuxtLink
				v-for="(link, index) in filteredLinks"
				v-show="link.shown ?? true"
				:key="link.href"
				ref="tabLinkElements"
				:to="query ? (link.href ? `?${query}=${link.href}` : '?') : link.href"
				class="button-animation z-[1] flex flex-row items-center gap-2 px-4 py-2 focus:rounded-full"
				:class="getSSRFallbackClasses(index)"
				@mouseenter="link.onHover?.()"
				@focus="link.onHover?.()"
			>
				<component :is="link.icon" v-if="link.icon" class="size-5" :class="getIconClasses(index)" />
				<span class="text-nowrap" :class="getLabelClasses(index)">
					{{ link.label }}
				</span>
			</NuxtLink>
		</template>

		<template v-else>
			<div
				v-for="(link, index) in filteredLinks"
				v-show="link.shown ?? true"
				:key="link.href"
				ref="tabLinkElements"
				class="button-animation z-[1] flex flex-row items-center gap-2 px-4 py-2 hover:cursor-pointer focus:rounded-full"
				:class="getSSRFallbackClasses(index)"
				@click="emit('tabClick', index, link)"
			>
				<component :is="link.icon" v-if="link.icon" class="size-5" :class="getIconClasses(index)" />
				<span class="text-nowrap" :class="getLabelClasses(index)">
					{{ link.label }}
				</span>
			</div>
		</template>

		<!-- Animated slider background -->
		<div
			class="pointer-events-none absolute h-[calc(100%-0.5rem)] overflow-hidden rounded-full p-1"
			:class="[
				subpageSelected ? 'bg-button-bg' : 'bg-button-bgSelected',
				{ 'navtabs-transition': transitionsEnabled },
			]"
			:style="sliderStyle"
			aria-hidden="true"
		/>
	</nav>
</template>

<script setup lang="ts">
import type { Component } from 'vue'
import { computed, onMounted, ref, watch } from 'vue'

const route = useNativeRoute()

interface Tab {
	label: string
	href: string
	shown?: boolean
	icon?: Component
	subpages?: string[]
	onHover?: () => void
}

const props = withDefaults(
	defineProps<{
		links: Tab[]
		query?: string
		mode?: 'navigation' | 'local'
		activeIndex?: number
	}>(),
	{
		mode: 'navigation',
		query: undefined,
		activeIndex: undefined,
	},
)

const emit = defineEmits<{
	tabClick: [index: number, tab: Tab]
}>()

// DOM refs
const scrollContainer = ref<HTMLElement | null>(null)
const tabLinkElements = ref<HTMLElement[]>()

// Slider pos state
const sliderLeft = ref(4)
const sliderTop = ref(4)
const sliderRight = ref(4)
const sliderBottom = ref(4)

// active tab state
const currentActiveIndex = ref(-1)
const subpageSelected = ref(false)

// SSR state
const sliderReady = ref(false) // Slider is positioned and should be visible
const transitionsEnabled = ref(false) // CSS transitions should apply (after first paint)

// Stagger delays for the trailing edges of the slider animation
const sliderDelays = ref({ left: '0ms', top: '0ms', right: '0ms', bottom: '0ms' })

const filteredLinks = computed(() => props.links.filter((link) => link.shown ?? true))

const sliderStyle = computed(() => ({
	left: `${sliderLeft.value}px`,
	top: `${sliderTop.value}px`,
	right: `${sliderRight.value}px`,
	bottom: `${sliderBottom.value}px`,
	opacity: sliderReady.value && currentActiveIndex.value !== -1 ? 1 : 0,
}))

const leftDelay = computed(() => sliderDelays.value.left)
const rightDelay = computed(() => sliderDelays.value.right)
const topDelay = computed(() => sliderDelays.value.top)
const bottomDelay = computed(() => sliderDelays.value.bottom)

const isActiveAndNotSubpage = computed(
	() => (index: number) => currentActiveIndex.value === index && !subpageSelected.value,
)

function getSSRFallbackClasses(index: number) {
	if (sliderReady.value) return {}
	if (currentActiveIndex.value !== index) return {}

	return {
		'rounded-full': true,
		'bg-button-bgSelected': !subpageSelected.value,
		'bg-button-bg': subpageSelected.value,
	}
}

function getIconClasses(index: number) {
	return {
		'text-button-textSelected': isActiveAndNotSubpage.value(index),
		'text-secondary': !isActiveAndNotSubpage.value(index),
	}
}

function getLabelClasses(index: number) {
	return {
		'text-button-textSelected': isActiveAndNotSubpage.value(index),
		'text-contrast': !isActiveAndNotSubpage.value(index),
	}
}

function computeActiveIndex(): { index: number; isSubpage: boolean } {
	if (props.mode === 'local' && props.activeIndex !== undefined) {
		return {
			index: Math.min(props.activeIndex, filteredLinks.value.length - 1),
			isSubpage: false,
		}
	}

	for (let i = filteredLinks.value.length - 1; i >= 0; i--) {
		const link = filteredLinks.value[i]
		const decodedPath = decodeURIComponent(route.path)

		// Query-based matching
		if (props.query) {
			const queryValue = route.query[props.query]
			if (queryValue === link.href || (!queryValue && !link.href)) {
				return { index: i, isSubpage: false }
			}
			continue
		}

		// Exact path match
		if (decodedPath === link.href) {
			return { index: i, isSubpage: false }
		}

		// Subpage match
		const isSubpageMatch =
			decodedPath.includes(link.href) ||
			link.subpages?.some((subpage) => decodedPath.includes(subpage))

		if (isSubpageMatch) {
			return { index: i, isSubpage: true }
		}
	}

	return { index: -1, isSubpage: false }
}

function getTabElement(index: number): HTMLElement | null {
    if (index === -1) return null

    const container = scrollContainer.value as HTMLElement | undefined
    if (!container) return null

    const tabs = container.querySelectorAll('.button-animation')
    const element = tabs[index] as HTMLElement | undefined

    if (!element) return null

    // In navigation mode, elements are NuxtLinks, but since we used querySelectorAll,
    // we already have the raw HTMLElement ($el), so no further conversion is needed.
    // In local mode, elements are already plain divs.
    return element
}

function positionSlider() {
	const el = getTabElement(currentActiveIndex.value)
	if (!el?.offsetParent) return

	const parent = el.offsetParent as HTMLElement
	const newPosition = {
		left: el.offsetLeft,
		top: el.offsetTop,
		right: parent.offsetWidth - el.offsetLeft - el.offsetWidth,
		bottom: parent.offsetHeight - el.offsetTop - el.offsetHeight,
	}

	const isInitialPosition = sliderLeft.value === 4 && sliderRight.value === 4

	if (isInitialPosition) {
		// Initial positioning: set position instantly, no animation
		sliderLeft.value = newPosition.left
		sliderRight.value = newPosition.right
		sliderTop.value = newPosition.top
		sliderBottom.value = newPosition.bottom

		sliderReady.value = true

		// enable transitions after slider is painted, so future changes animate
		requestAnimationFrame(() => {
			transitionsEnabled.value = true
		})
	} else {
		animateSliderTo(newPosition)
	}
}

function animateSliderTo(newPosition: {
	left: number
	top: number
	right: number
	bottom: number
}) {
	const STAGGER_DELAY = '200ms'

	// Set stagger delays: leading edge moves immediately, trailing edge is delayed
	sliderDelays.value = {
		left: newPosition.left < sliderLeft.value ? '0ms' : STAGGER_DELAY,
		right: newPosition.left < sliderLeft.value ? STAGGER_DELAY : '0ms',
		top: newPosition.top < sliderTop.value ? '0ms' : STAGGER_DELAY,
		bottom: newPosition.top < sliderTop.value ? STAGGER_DELAY : '0ms',
	}

	sliderLeft.value = newPosition.left
	sliderRight.value = newPosition.right
	sliderTop.value = newPosition.top
	sliderBottom.value = newPosition.bottom
}

async function updateActiveTab() {
	await nextTick()
	const { index, isSubpage } = computeActiveIndex()
	currentActiveIndex.value = index
	subpageSelected.value = isSubpage

	if (index !== -1) {
		positionSlider()
	} else {
		sliderLeft.value = 0
		sliderRight.value = 0
	}
}

const initialActive = computeActiveIndex()
currentActiveIndex.value = initialActive.index
subpageSelected.value = initialActive.isSubpage

onMounted(updateActiveTab)

watch(
	() => [route.path, route.query],
	() => {
		if (props.mode === 'navigation') {
			updateActiveTab()
		}
	},
)

watch(
	() => props.activeIndex,
	() => {
		if (props.mode === 'local') {
			updateActiveTab()
		}
	},
)

watch(() => props.links, updateActiveTab, { deep: true })
</script>

<style scoped>
.navtabs-transition {
	transition:
		left 150ms cubic-bezier(0.4, 0, 0.2, 1) v-bind(leftDelay),
		right 150ms cubic-bezier(0.4, 0, 0.2, 1) v-bind(rightDelay),
		top 150ms cubic-bezier(0.4, 0, 0.2, 1) v-bind(topDelay),
		bottom 150ms cubic-bezier(0.4, 0, 0.2, 1) v-bind(bottomDelay),
		opacity 250ms cubic-bezier(0.5, 0, 0.2, 1) 50ms;
}

.card-shadow {
	box-shadow: var(--shadow-card);
}
</style>
