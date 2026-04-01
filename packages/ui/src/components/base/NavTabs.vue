<template>
	<nav
		v-if="filteredLinks.length > 1"
		class="relative flex w-fit gap-2 overflow-x-auto text-sm mx-auto font-bold mt-6"
	>
		<template v-if="mode === 'navigation'">
			<RouterLink
				v-for="(link, index) in filteredLinks"
				v-show="link.shown ?? true"
				:key="link.href"
				:replace="replace"
				:to="query ? (link.href ? `?${query}=${link.href}` : '?') : link.href"
				class="flex tab-button flex-row items-center px-8 py-2 rounded-t-[10px] bg-surface-1"
				:class="{
					'tab-button-selected': isActiveAndNotSubpage(index),
				}"
				@mouseenter="link.onHover?.()"
				@focus="link.onHover?.()"
			>
				<span class="text-nowrap">
					{{ link.label }}
				</span>
			</RouterLink>
		</template>

		<template v-else>
			<div
				v-for="(link, index) in filteredLinks"
				v-show="link.shown ?? true"
				:key="link.href"
				class="flex tab-button flex-row items-center px-8 py-2 rounded-t-[10px] bg-surface-1"
				:class="{
					'tab-button-selected': isActiveAndNotSubpage(index),
				}"
				@click="emit('tabClick', index, link)"
			>
				<span class="text-nowrap">
					{{ link.label }}
				</span>
			</div>
		</template>
	</nav>
</template>

<script setup lang="ts">
import type { Component } from 'vue'
import { computed, onMounted, ref, watch } from 'vue'
import { RouterLink, useRoute } from 'vue-router'

const route = useRoute()

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

const currentActiveIndex = ref(-1)
const subpageSelected = ref(false)

const filteredLinks = computed(() => props.links.filter((link) => link.shown ?? true))

const isActiveAndNotSubpage = computed(
	() => (index: number) => currentActiveIndex.value === index && !subpageSelected.value,
)

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

function updateActiveTab() {
	const { index, isSubpage } = computeActiveIndex()
	currentActiveIndex.value = index
	subpageSelected.value = isSubpage
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
	() => {
		updateActiveTab()
	},
	{ deep: true },
)
</script>
<style scoped>
.tab-button {
	background: linear-gradient(to bottom, white, var(--color-green-300), var(--color-green-600));
	color: var(--color-green-900);
	border: 1px solid var(--color-green-500);
	border-bottom: 0;
}
.tab-button-selected {
	background: linear-gradient(to bottom, white, #cacaca, #b5b5b5);
	color: black;
	border: 1px solid #afafaf;
	border-bottom: 0;
}
</style>
}
