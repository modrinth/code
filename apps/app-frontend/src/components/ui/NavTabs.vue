<template>
	<nav
		v-if="filteredLinks.length > 1"
		ref="scrollContainer"
		class="card-shadow experimental-styles-within relative flex w-fit overflow-clip rounded-full bg-bg-raised p-1 text-sm font-bold"
	>
		<RouterLink
			v-for="(link, index) in filteredLinks"
			v-show="link.shown === undefined ? true : link.shown"
			:key="index"
			ref="tabLinkElements"
			:to="query ? (link.href ? `?${query}=${link.href}` : '?') : link.href"
			:class="`button-animation z-[1] flex flex-row items-center gap-2 px-4 py-2 focus:rounded-full ${activeIndex === index && !subpageSelected ? 'text-button-textSelected' : activeIndex === index && subpageSelected ? 'text-contrast' : 'text-primary'}`"
		>
			<component :is="link.icon" v-if="link.icon" class="size-5" />
			<span class="text-nowrap">{{ link.label }}</span>
		</RouterLink>
		<div
			:class="[
				'pointer-events-none absolute h-[calc(100%-0.5rem)] overflow-hidden rounded-full p-1',
				subpageSelected ? 'bg-button-bg' : 'bg-button-bgSelected',
				{ 'navtabs-transition': transitionsEnabled },
			]"
			:style="{
				left: sliderLeftPx,
				top: sliderTopPx,
				right: sliderRightPx,
				bottom: sliderBottomPx,
				opacity: sliderReady && activeIndex !== -1 ? 1 : 0,
			}"
			aria-hidden="true"
		></div>
	</nav>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import type { RouteLocationRaw } from 'vue-router'
import { RouterLink, useRoute } from 'vue-router'

const route = useRoute()

interface Tab {
	label: string
	href: string | RouteLocationRaw
	shown?: boolean
	icon?: unknown
	subpages?: string[]
}

const props = defineProps<{
	links: Tab[]
	query?: string
}>()

const scrollContainer = ref<HTMLElement | null>(null)
const sliderLeft = ref(4)
const sliderTop = ref(4)
const sliderRight = ref(4)
const sliderBottom = ref(4)
const activeIndex = ref(-1)
const subpageSelected = ref(false)
const sliderReady = ref(false)
const transitionsEnabled = ref(false)
const sliderDelays = ref({ left: '0ms', top: '0ms', right: '0ms', bottom: '0ms' })

const filteredLinks = computed(() =>
	props.links.filter((x) => (x.shown === undefined ? true : x.shown)),
)
const sliderLeftPx = computed(() => `${sliderLeft.value}px`)
const sliderTopPx = computed(() => `${sliderTop.value}px`)
const sliderRightPx = computed(() => `${sliderRight.value}px`)
const sliderBottomPx = computed(() => `${sliderBottom.value}px`)

const leftDelay = computed(() => sliderDelays.value.left)
const rightDelay = computed(() => sliderDelays.value.right)
const topDelay = computed(() => sliderDelays.value.top)
const bottomDelay = computed(() => sliderDelays.value.bottom)

function pickLink() {
	let index = -1
	subpageSelected.value = false
	for (let i = filteredLinks.value.length - 1; i >= 0; i--) {
		const link = filteredLinks.value[i]

		if (route.path === (typeof link.href === 'string' ? link.href : (link.href as any).path)) {
			index = i
			break
		} else if (link.subpages && link.subpages.some((subpage) => route.path.includes(subpage))) {
			index = i
			subpageSelected.value = true
			break
		}
	}
	activeIndex.value = index

	if (activeIndex.value !== -1) {
		startAnimation()
	} else {
		sliderLeft.value = 0
		sliderRight.value = 0
	}
}

function getTabElement(index: number): HTMLElement | null {
	if (index === -1) return null
	const container = scrollContainer.value
	if (!container) return null
	const tabs = container.querySelectorAll('.button-animation')
	return (tabs[index] as HTMLElement) ?? null
}

function startAnimation() {
	const el = getTabElement(activeIndex.value)
	if (!el?.offsetParent) return

	const parent = el.offsetParent as HTMLElement
	const newValues = {
		left: el.offsetLeft,
		top: el.offsetTop,
		right: parent.offsetWidth - el.offsetLeft - el.offsetWidth,
		bottom: parent.offsetHeight - el.offsetTop - el.offsetHeight,
	}

	const isInitialPosition = sliderLeft.value === 4 && sliderRight.value === 4

	if (isInitialPosition) {
		sliderLeft.value = newValues.left
		sliderRight.value = newValues.right
		sliderTop.value = newValues.top
		sliderBottom.value = newValues.bottom
		sliderReady.value = true
		requestAnimationFrame(() => {
			transitionsEnabled.value = true
		})
	} else {
		const STAGGER_DELAY = '200ms'
		sliderDelays.value = {
			left: newValues.left < sliderLeft.value ? '0ms' : STAGGER_DELAY,
			right: newValues.left < sliderLeft.value ? STAGGER_DELAY : '0ms',
			top: newValues.top < sliderTop.value ? '0ms' : STAGGER_DELAY,
			bottom: newValues.top < sliderTop.value ? STAGGER_DELAY : '0ms',
		}
		sliderLeft.value = newValues.left
		sliderRight.value = newValues.right
		sliderTop.value = newValues.top
		sliderBottom.value = newValues.bottom
	}
}

onMounted(() => {
	window.addEventListener('resize', pickLink)
	pickLink()
})

onUnmounted(() => {
	window.removeEventListener('resize', pickLink)
})

watch(
	filteredLinks,
	async () => {
		await nextTick()
		pickLink()
	},
	{ deep: true },
)

watch(route, async () => {
	await nextTick()
	pickLink()
})
</script>
<style scoped>
.navtabs-transition {
	/* Delay on opacity is to hide any jankiness as the page loads */
	transition:
		left 150ms cubic-bezier(0.4, 0, 0.2, 1) v-bind(leftDelay),
		right 150ms cubic-bezier(0.4, 0, 0.2, 1) v-bind(rightDelay),
		top 150ms cubic-bezier(0.4, 0, 0.2, 1) v-bind(topDelay),
		bottom 150ms cubic-bezier(0.4, 0, 0.2, 1) v-bind(bottomDelay),
		opacity 250ms cubic-bezier(0.5, 0, 0.2, 1) 50ms;
}
</style>
