<template>
	<nav
		ref="scrollContainer"
		class="experimental-styles-within relative flex w-fit overflow-x-auto rounded-full bg-bg-raised p-1 text-sm font-bold"
		:class="[mode === 'navigation' ? 'card-shadow' : undefined]"
	>
		<template v-if="mode === 'navigation'">
			<NuxtLink
				v-for="(link, index) in filteredLinks"
				v-show="link.shown === undefined ? true : link.shown"
				:key="link.href"
				ref="tabLinkElements"
				:to="query ? (link.href ? `?${query}=${link.href}` : '?') : link.href"
				class="button-animation z-[1] flex flex-row items-center gap-2 px-4 py-2 focus:rounded-full"
			>
				<component
					:is="link.icon"
					v-if="link.icon"
					class="size-5"
					:class="{
						'text-brand': currentActiveIndex === index && !subpageSelected,
						'text-secondary': currentActiveIndex !== index || subpageSelected,
					}"
				/>
				<span class="text-nowrap text-contrast">{{ link.label }}</span>
			</NuxtLink>
		</template>
		<template v-else>
			<div
				v-for="(link, index) in filteredLinks"
				v-show="link.shown === undefined ? true : link.shown"
				:key="link.href"
				ref="tabLinkElements"
				class="button-animation z-[1] flex flex-row items-center gap-2 px-4 py-2 hover:cursor-pointer focus:rounded-full"
				@click="emit('tabClick', index, link)"
			>
				<component
					:is="link.icon"
					v-if="link.icon"
					class="size-5"
					:class="{
						'text-brand': currentActiveIndex === index && !subpageSelected,
						'text-secondary': currentActiveIndex !== index || subpageSelected,
					}"
				/>
				<span
					class="text-nowrap"
					:class="{
						'text-brand': currentActiveIndex === index && !subpageSelected,
						'text-contrast': currentActiveIndex !== index || subpageSelected,
					}"
					>{{ link.label }}</span
				>
			</div>
		</template>
		<div
			:class="`navtabs-transition pointer-events-none absolute h-[calc(100%-0.5rem)] overflow-hidden rounded-full p-1 ${
				subpageSelected ? 'bg-button-bg' : 'bg-button-bgSelected'
			}`"
			:style="{
				left: sliderLeftPx,
				top: sliderTopPx,
				right: sliderRightPx,
				bottom: sliderBottomPx,
				opacity:
					sliderLeft === 4 && sliderLeft === sliderRight ? 0 : currentActiveIndex === -1 ? 0 : 1,
			}"
			aria-hidden="true"
		></div>
	</nav>
</template>

<script setup lang="ts">
import type { Component } from 'vue'
import { computed, nextTick, onMounted, ref, watch } from 'vue'

const route = useNativeRoute()

interface Tab {
	label: string
	href: string
	shown?: boolean
	icon?: Component
	subpages?: string[]
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

const scrollContainer = ref<HTMLElement | null>(null)

const sliderLeft = ref(4)
const sliderTop = ref(4)
const sliderRight = ref(4)
const sliderBottom = ref(4)
const currentActiveIndex = ref(-1)
const subpageSelected = ref(false)

const filteredLinks = computed(() =>
	props.links.filter((x) => (x.shown === undefined ? true : x.shown)),
)
const sliderLeftPx = computed(() => `${sliderLeft.value}px`)
const sliderTopPx = computed(() => `${sliderTop.value}px`)
const sliderRightPx = computed(() => `${sliderRight.value}px`)
const sliderBottomPx = computed(() => `${sliderBottom.value}px`)

const tabLinkElements = ref()

function pickLink() {
	let index = -1
	subpageSelected.value = false

	if (props.mode === 'local' && props.activeIndex !== undefined) {
		index = Math.min(props.activeIndex, filteredLinks.value.length - 1)
	} else {
		for (let i = filteredLinks.value.length - 1; i >= 0; i--) {
			const link = filteredLinks.value[i]
			if (props.query) {
				if (route.query[props.query] === link.href || (!route.query[props.query] && !link.href)) {
					index = i
					break
				}
			} else if (decodeURIComponent(route.path) === link.href) {
				index = i
				break
			} else if (
				decodeURIComponent(route.path).includes(link.href) ||
				(link.subpages &&
					link.subpages.some((subpage) => decodeURIComponent(route.path).includes(subpage)))
			) {
				index = i
				subpageSelected.value = true
				break
			}
		}
	}

	currentActiveIndex.value = index

	if (currentActiveIndex.value !== -1) {
		nextTick(() => startAnimation())
	} else {
		sliderLeft.value = 0
		sliderRight.value = 0
	}
}

function startAnimation() {
	// In navigation mode, elements are NuxtLinks with $el property
	// In local mode, elements are plain divs
	const el =
		props.mode === 'navigation'
			? tabLinkElements.value[currentActiveIndex.value]?.$el
			: tabLinkElements.value[currentActiveIndex.value]

	if (!el || !el.offsetParent) return

	const newValues = {
		left: el.offsetLeft,
		top: el.offsetTop,
		right: el.offsetParent.offsetWidth - el.offsetLeft - el.offsetWidth,
		bottom: el.offsetParent.offsetHeight - el.offsetTop - el.offsetHeight,
	}

	if (sliderLeft.value === 4 && sliderRight.value === 4) {
		sliderLeft.value = newValues.left
		sliderRight.value = newValues.right
		sliderTop.value = newValues.top
		sliderBottom.value = newValues.bottom
	} else {
		const delay = 200

		if (newValues.left < sliderLeft.value) {
			sliderLeft.value = newValues.left
			setTimeout(() => {
				sliderRight.value = newValues.right
			}, delay)
		} else {
			sliderRight.value = newValues.right
			setTimeout(() => {
				sliderLeft.value = newValues.left
			}, delay)
		}

		if (newValues.top < sliderTop.value) {
			sliderTop.value = newValues.top
			setTimeout(() => {
				sliderBottom.value = newValues.bottom
			}, delay)
		} else {
			sliderBottom.value = newValues.bottom
			setTimeout(() => {
				sliderTop.value = newValues.top
			}, delay)
		}
	}
}

onMounted(() => {
	pickLink()
})

watch(
	() => [route.path, route.query],
	() => {
		if (props.mode === 'navigation') {
			pickLink()
		}
	},
)

watch(
	() => props.activeIndex,
	() => {
		if (props.mode === 'local') {
			pickLink()
		}
	},
)

watch(
	() => props.links,
	() => {
		// Re-trigger animation when links change
		pickLink()
	},
	{ deep: true },
)
</script>

<style scoped>
.navtabs-transition {
	transition:
		all 150ms cubic-bezier(0.4, 0, 0.2, 1),
		opacity 250ms cubic-bezier(0.5, 0, 0.2, 1) 50ms;
}

.card-shadow {
	box-shadow: var(--shadow-card);
}
</style>
