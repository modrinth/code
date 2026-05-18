<template>
	<nav
		v-if="filteredLinks.length > 1"
		ref="scrollContainer"
		class="relative flex w-fit overflow-x-auto rounded-full bg-bg-raised p-1 text-sm font-bold"
		:class="{ 'drop-shadow-xl border border-solid border-surface-4': mode === 'navigation' }"
	>
		<template v-if="mode === 'navigation'">
			<template v-for="(link, index) in filteredLinks" :key="link.href">
				<Tooltip
					v-if="link.prompt"
					theme="dismissable-prompt"
					:triggers="[]"
					:shown="link.prompt.shown"
					:auto-hide="false"
					:placement="link.prompt.placement ?? 'bottom'"
				>
					<RouterLink
						ref="tabLinkElements"
						:replace="replace"
						:to="query ? (link.href ? `?${query}=${link.href}` : '?') : link.href"
						class="button-animation z-[1] flex flex-row items-center gap-2 px-4 py-2 focus:rounded-full"
						:class="getSSRFallbackClasses(index)"
						@mouseenter="link.onHover?.()"
						@focus="link.onHover?.()"
						@click="dismissPrompt(link)"
					>
						<component
							:is="link.icon"
							v-if="link.icon"
							class="size-5"
							:class="getIconClasses(index)"
						/>
						<span class="text-nowrap" :class="getLabelClasses(index)">
							{{ link.label }}
						</span>
					</RouterLink>
					<template #popper>
						<div class="grid grid-cols-[min-content] gap-1">
							<div class="flex min-w-48 items-center justify-between gap-8">
								<h3 class="m-0 whitespace-nowrap text-base font-bold text-contrast">
									{{ link.prompt.title }}
								</h3>
								<ButtonStyled size="small" circular>
									<button v-tooltip="link.prompt.dismissLabel" @click="link.prompt.onDismiss?.()">
										<XIcon aria-hidden="true" />
									</button>
								</ButtonStyled>
							</div>
							<p class="m-0 text-wrap text-sm font-medium leading-tight text-secondary">
								{{ link.prompt.description }}
							</p>
						</div>
					</template>
				</Tooltip>
				<RouterLink
					v-else
					ref="tabLinkElements"
					:replace="replace"
					:to="query ? (link.href ? `?${query}=${link.href}` : '?') : link.href"
					class="button-animation z-[1] flex flex-row items-center gap-2 px-4 py-2 focus:rounded-full"
					:class="getSSRFallbackClasses(index)"
					@mouseenter="link.onHover?.()"
					@focus="link.onHover?.()"
				>
					<component
						:is="link.icon"
						v-if="link.icon"
						class="size-5"
						:class="getIconClasses(index)"
					/>
					<span class="text-nowrap" :class="getLabelClasses(index)">
						{{ link.label }}
					</span>
				</RouterLink>
			</template>
		</template>

		<template v-else>
			<template v-for="(link, index) in filteredLinks" :key="link.href">
				<Tooltip
					v-if="link.prompt"
					theme="dismissable-prompt"
					:triggers="[]"
					:shown="link.prompt.shown"
					:auto-hide="false"
					:placement="link.prompt.placement ?? 'bottom'"
				>
					<div
						ref="tabLinkElements"
						class="button-animation z-[1] flex flex-row items-center gap-2 px-4 py-2 hover:cursor-pointer focus:rounded-full"
						:class="getSSRFallbackClasses(index)"
						@click="handleLocalTabClick(index, link)"
					>
						<component
							:is="link.icon"
							v-if="link.icon"
							class="size-5"
							:class="getIconClasses(index)"
						/>
						<span class="text-nowrap" :class="getLabelClasses(index)">
							{{ link.label }}
						</span>
					</div>
					<template #popper>
						<div class="grid grid-cols-[min-content] gap-1">
							<div class="flex min-w-48 items-center justify-between gap-8">
								<h3 class="m-0 whitespace-nowrap text-base font-bold text-contrast">
									{{ link.prompt.title }}
								</h3>
								<ButtonStyled size="small" circular>
									<button v-tooltip="link.prompt.dismissLabel" @click="link.prompt.onDismiss?.()">
										<XIcon aria-hidden="true" />
									</button>
								</ButtonStyled>
							</div>
							<p class="m-0 text-wrap text-sm font-medium leading-tight text-secondary">
								{{ link.prompt.description }}
							</p>
						</div>
					</template>
				</Tooltip>
				<div
					v-else
					ref="tabLinkElements"
					class="button-animation z-[1] flex flex-row items-center gap-2 px-4 py-2 hover:cursor-pointer focus:rounded-full"
					:class="getSSRFallbackClasses(index)"
					@click="handleLocalTabClick(index, link)"
				>
					<component
						:is="link.icon"
						v-if="link.icon"
						class="size-5"
						:class="getIconClasses(index)"
					/>
					<span class="text-nowrap" :class="getLabelClasses(index)">
						{{ link.label }}
					</span>
				</div>
			</template>
		</template>

		<!-- Animated slider background -->
		<div
			v-if="sliderReady && currentActiveIndex !== -1"
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
import { XIcon } from '@modrinth/assets'
import { Tooltip } from 'floating-vue'
import type { Component } from 'vue'
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { RouterLink, useRoute } from 'vue-router'

import ButtonStyled from './ButtonStyled.vue'

const route = useRoute()

interface TabPrompt {
	title: string
	description: string
	dismissLabel?: string
	shown?: boolean
	placement?: string
	onDismiss?: () => void
}

interface Tab {
	label: string
	href: string
	shown?: boolean
	icon?: Component
	subpages?: string[]
	onHover?: () => void
	prompt?: TabPrompt
}

const props = withDefaults(
	defineProps<{
		replace?: boolean
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
const sliderReady = ref(false)
const transitionsEnabled = ref(false)

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

function dismissPrompt(link: Tab) {
	if (link.prompt?.shown) {
		link.prompt.onDismiss?.()
	}
}

function handleLocalTabClick(index: number, link: Tab) {
	dismissPrompt(link)
	emit('tabClick', index, link)
}

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
		const decodedHref = decodeURIComponent(link.href.split('?')[0])

		if (props.query) {
			const queryValue = route.query[props.query]
			if (queryValue === link.href || (!queryValue && !link.href)) {
				return { index: i, isSubpage: false }
			}
			continue
		}

		if (decodedPath === decodedHref) {
			return { index: i, isSubpage: false }
		}

		const isSubpageMatch =
			(decodedPath.startsWith(decodedHref) &&
				(decodedPath.length === decodedHref.length || decodedPath[decodedHref.length] === '/')) ||
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

	if (!sliderReady.value || isInitialPosition) {
		sliderLeft.value = newPosition.left
		sliderRight.value = newPosition.right
		sliderTop.value = newPosition.top
		sliderBottom.value = newPosition.bottom

		sliderReady.value = true

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

watch(
	() => props.links,
	async () => {
		sliderReady.value = false
		transitionsEnabled.value = false
		await nextTick()
		updateActiveTab()
	},
	{ deep: true },
)
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
</style>
