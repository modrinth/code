<script lang="ts"></script>

<script setup lang="ts">
import {
	ArrowUpRightIcon,
	ChevronLeftIcon,
	ChevronRightIcon,
	PanelLeftCloseIcon,
	PanelLeftOpenIcon,
	RightArrowIcon,
	XIcon,
} from '@modrinth/assets'
import { useElementSize, useEventListener } from '@vueuse/core'
import {
	type Component,
	type ComponentPublicInstance,
	computed,
	nextTick,
	onUnmounted,
	ref,
	watch,
} from 'vue'

import { defineMessages, type MessageDescriptor, useVIntl } from '../../composables/i18n'
import { useScrollIndicator } from '../../composables/scroll-indicator'
import { commonMessages } from '../../utils/common-messages'
import { truncatedTooltip } from '../../utils/truncate'
import LoadingIndicator from '../base/LoadingIndicator.vue'
import NewModal from './NewModal.vue'
export interface Tab {
	name: MessageDescriptor
	category?: MessageDescriptor
	icon: Component
	content?: Component
	href?: string
	badge?: MessageDescriptor
	shown?: boolean
}

type Layout = 'desktop' | 'rail' | 'mobile'

const MOBILE_MAX_WIDTH = 640
const RAIL_MAX_WIDTH = 900

const slots = defineSlots<{
	title?(): unknown
	/** Context shown before the header, e.g. the server or instance name. */
	breadcrumb?(): unknown
	'sidebar-header'?(): unknown
	footer?(): unknown
	content?(props: { tab: Tab | undefined; index: number }): unknown
	'floating-action-bar'?(): unknown
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	expandSidebar: {
		id: 'modal.tabbed.expand-sidebar',
		defaultMessage: 'Expand sidebar',
	},
	collapseSidebar: {
		id: 'modal.tabbed.collapse-sidebar',
		defaultMessage: 'Collapse sidebar',
	},
	collapse: {
		id: 'modal.tabbed.collapse',
		defaultMessage: 'Collapse',
	},
})

const props = withDefaults(
	defineProps<{
		tabs: Tab[]
		header?: string
		maxWidth?: string
		width?: string
		closable?: boolean
		onHide?: () => void
		onAfterHide?: () => void
		onShow?: () => void
		beforeHide?: () => boolean
		beforeTabChange?: (fromIndex: number, toIndex: number) => boolean
		hideTabSelection?: boolean
		floatingActionBarShown?: boolean
		disableClose?: boolean
	}>(),
	{
		header: undefined,
		maxWidth: '928px',
		width: '928px',
		closable: true,
		onHide: undefined,
		onAfterHide: undefined,
		onShow: undefined,
		beforeHide: undefined,
		beforeTabChange: undefined,
		hideTabSelection: false,
		floatingActionBarShown: false,
		disableClose: false,
	},
)

const visibleTabs = computed(() => props.tabs.filter((tab) => tab.shown !== false))

const selectedTab = ref(0)
const tabLabelRefs = ref<Record<number, HTMLElement | null>>({})
const mobileTabRefs = ref<Record<number, HTMLElement | null>>({})

function setTabLabelRef(index: number, element: Element | ComponentPublicInstance | null) {
	tabLabelRefs.value[index] = element instanceof HTMLElement ? element : null
}

function setMobileTabRef(index: number, element: Element | ComponentPublicInstance | null) {
	mobileTabRefs.value[index] = element instanceof HTMLElement ? element : null
}

function tabLabelTooltip(index: number, label: string) {
	return truncatedTooltip(tabLabelRefs.value[index], label)
}

const layoutRoot = ref<HTMLElement | null>(null)
const ownerWindow = computed(() => layoutRoot.value?.ownerDocument.defaultView ?? null)
const viewportWidth = ref<number>()

function updateViewportWidth() {
	viewportWidth.value = ownerWindow.value?.innerWidth
}

watch(ownerWindow, updateViewportWidth, { immediate: true })
useEventListener(ownerWindow, 'resize', updateViewportWidth)

const canUseRail = computed(() => !slots['sidebar-header'] && !slots.footer)

const layout = computed<Layout>(() => {
	const width = viewportWidth.value
	if (width === undefined || width > RAIL_MAX_WIDTH) return 'desktop'
	if (width <= MOBILE_MAX_WIDTH) return 'mobile'
	return canUseRail.value ? 'rail' : 'desktop'
})
const isMobile = computed(() => layout.value === 'mobile')
const railExpanded = ref(false)

const mobileView = ref<'list' | 'detail'>('list')
const showMobileDetail = computed(() => isMobile.value && mobileView.value === 'detail')
const showContent = computed(
	() => !isMobile.value || mobileView.value === 'detail' || props.hideTabSelection,
)
const showSidebar = computed(() => !showMobileDetail.value)
const mobileBackButton = ref<HTMLElement | null>(null)

const scrollContainer = ref<HTMLElement | null>(null)
const { showTopFade, showBottomFade, checkScrollState, forceCheck } =
	useScrollIndicator(scrollContainer)

const floatingActionBarContainer = ref<HTMLElement | null>(null)
const { height: floatingActionBarHeight } = useElementSize(floatingActionBarContainer)
const contentBottomPadding = computed(() =>
	props.floatingActionBarShown ? `calc(${floatingActionBarHeight.value}px + 2.25rem)` : '1.5rem',
)

watch(contentBottomPadding, () => forceCheck(), { flush: 'post' })

const sidebarScrollContainer = ref<HTMLElement | null>(null)
const {
	showTopFade: showSidebarTopFade,
	showBottomFade: showSidebarBottomFade,
	checkScrollState: checkSidebarScrollState,
} = useScrollIndicator(sidebarScrollContainer)

const modal = ref<InstanceType<typeof NewModal> | null>(null)

function setTab(index: number): boolean {
	if (index === selectedTab.value && !props.hideTabSelection) return true
	if (props.beforeTabChange?.(selectedTab.value, index) === false) return false
	selectedTab.value = index
	nextTick(() => forceCheck())
	return true
}

function openTab(index: number) {
	if (!setTab(index) || !isMobile.value) return
	mobileView.value = 'detail'
	nextTick(() => {
		scrollContainer.value?.scrollTo({ top: 0 })
		forceCheck()
		mobileBackButton.value?.focus({ preventScroll: true })
	})
}

function showTabList() {
	mobileView.value = 'list'
	nextTick(() => mobileTabRefs.value[selectedTab.value]?.focus({ preventScroll: true }))
}

function show(event?: MouseEvent) {
	mobileView.value = 'list'
	modal.value?.show(event)
	nextTick(() => {
		if (props.hideTabSelection) return
		tabLabelRefs.value[selectedTab.value]?.parentElement?.scrollIntoView({
			behavior: 'instant',
			block: 'nearest',
			inline: 'nearest',
		})
		checkSidebarScrollState()
	})
}

function hide(): boolean {
	return modal.value?.hide() ?? false
}

const HISTORY_STATE_KEY = 'tabbedModalSheet'
let historyWindow: Window | null = null

function pushHistoryEntry(win: Window) {
	if (historyWindow) return
	historyWindow = win
	win.history.pushState({ ...win.history.state, [HISTORY_STATE_KEY]: true }, '')
}

function releaseHistoryEntry() {
	const win = historyWindow
	if (!win) return
	historyWindow = null
	if (win.history.state?.[HISTORY_STATE_KEY]) win.history.back()
}

function onHistoryPop() {
	const win = historyWindow
	if (!win) return
	historyWindow = null
	if (showMobileDetail.value) {
		showTabList()
		pushHistoryEntry(win)
	} else if (!hide()) {
		pushHistoryEntry(win)
	}
}

watch([ownerWindow, isMobile], ([win, mobile]) => {
	if (win && mobile) pushHistoryEntry(win)
	else if (!mobile) releaseHistoryEntry()
})
useEventListener(ownerWindow, 'popstate', onHistoryPop)
onUnmounted(releaseHistoryEntry)

function handleHide() {
	releaseHistoryEntry()
	props.onHide?.()
}

function startsCategory(index: number) {
	const category = visibleTabs.value[index]?.category
	return !!category && category.id !== visibleTabs.value[index - 1]?.category?.id
}

function startsGroup(index: number) {
	const tab = visibleTabs.value[index]
	const previous = visibleTabs.value[index - 1]
	if (!tab || !previous) return false
	return tab.category?.id !== previous.category?.id || !!tab.href !== !!previous.href
}

const mobileTabGroups = computed(() => {
	const groups: { index: number; tab: Tab }[][] = []
	visibleTabs.value.forEach((tab, index) => {
		if (index === 0 || startsGroup(index)) groups.push([])
		groups[groups.length - 1].push({ index, tab })
	})
	return groups
})

function isTabSelected(tab: Tab, index: number) {
	return !tab.href && !props.hideTabSelection && selectedTab.value === index
}

function tabAccessibleName(tab: Tab) {
	const name = formatMessage(tab.name)
	return tab.badge ? `${name} (${formatMessage(tab.badge)})` : name
}

const selectedTabName = computed(() => {
	const tab = visibleTabs.value[selectedTab.value]
	return tab ? formatMessage(tab.name) : ''
})

const gridClass = computed(() => {
	switch (layout.value) {
		case 'mobile':
			return 'flex flex-1 flex-col'
		case 'rail':
			return `grid h-[min(65vh,640px)] transition-[grid-template-columns] duration-200 ease-out motion-reduce:transition-none ${railExpanded.value ? 'grid-cols-[250px_minmax(0,1fr)]' : 'grid-cols-[76px_minmax(0,1fr)]'}`
		default:
			return 'grid h-[min(65vh,640px)] grid-cols-[250px_minmax(0,1fr)]'
	}
})

const sidebarClass = computed(() => {
	switch (layout.value) {
		case 'mobile':
			return props.hideTabSelection ? 'shrink-0 px-4 pt-4' : 'flex-1 gap-4 p-4'
		case 'rail':
			return 'py-3'
		default:
			return 'gap-2 p-4'
	}
})

const floatingActionBarClass = computed(() => {
	switch (layout.value) {
		case 'mobile':
			return 'bottom-4 left-3 right-3'
		case 'rail':
			return `bottom-3 right-6 transition-[left] duration-200 ease-out motion-reduce:transition-none ${railExpanded.value ? 'left-[calc(250px+1.5rem)]' : 'left-[calc(76px+1.5rem)]'}`
		default:
			return 'bottom-3 left-[calc(250px+1.5rem)] right-6'
	}
})

defineExpose({ show, hide, selectedTab, setTab })
</script>
<template>
	<NewModal
		ref="modal"
		:header="header"
		:max-width="maxWidth"
		:width="width"
		:closable="closable"
		:on-hide="handleHide"
		:on-after-hide="onAfterHide"
		:on-show="onShow"
		:before-hide="beforeHide"
		:disable-close="disableClose"
		:hide-header="isMobile"
		:class="['!bg-surface-3', { '!rounded-[20px]': !isMobile }]"
		mobile-sheet
		no-padding
	>
		<template v-if="$slots.title || $slots.breadcrumb" #title>
			<slot name="title">
				<span class="flex min-w-0 items-center gap-2 text-lg font-semibold text-primary">
					<span class="flex min-w-0 items-center gap-2">
						<slot name="breadcrumb" />
					</span>
					<ChevronRightIcon class="shrink-0" aria-hidden="true" />
					<span class="font-extrabold text-contrast">{{ header }}</span>
				</span>
			</slot>
		</template>
		<div
			v-if="isMobile"
			data-sheet-drag-handle
			class="flex touch-none items-center gap-3 border-0 border-b border-solid border-surface-5 pb-4 pr-4 pt-2.5"
			:class="showMobileDetail ? 'pl-3' : 'pl-5'"
		>
			<button
				v-if="showMobileDetail"
				ref="mobileBackButton"
				type="button"
				:aria-label="formatMessage(commonMessages.backButton)"
				class="flex size-11 shrink-0 cursor-pointer items-center justify-center rounded-full border-none bg-transparent text-contrast transition-all hover:bg-surface-4 active:scale-95"
				@click="showTabList"
			>
				<ChevronLeftIcon class="size-6" aria-hidden="true" />
			</button>
			<div class="flex min-h-[52px] min-w-0 flex-1 flex-col justify-center gap-0.5">
				<span
					v-if="$slots.breadcrumb || (showMobileDetail && header)"
					class="flex min-w-0 items-center gap-1.5 truncate text-sm font-semibold text-secondary"
				>
					<slot v-if="$slots.breadcrumb" name="breadcrumb" />
					<template v-else>{{ header }}</template>
				</span>
				<h2 class="m-0 truncate text-2xl font-semibold leading-[30px] text-contrast">
					<template v-if="showMobileDetail">{{ selectedTabName }}</template>
					<template v-else-if="header">{{ header }}</template>
					<slot v-else name="title" />
				</h2>
			</div>
			<button
				v-if="closable"
				type="button"
				:aria-label="formatMessage(commonMessages.closeButton)"
				:disabled="disableClose"
				class="flex size-11 shrink-0 cursor-pointer items-center justify-center rounded-full border-none bg-surface-4 text-primary transition-all hover:text-contrast active:scale-95 disabled:cursor-not-allowed disabled:opacity-50"
				@click="hide"
			>
				<XIcon class="size-5" aria-hidden="true" />
			</button>
		</div>
		<div ref="layoutRoot" class="relative min-h-0 overflow-hidden" :class="gridClass">
			<div
				v-show="showSidebar"
				class="relative flex min-h-0 min-w-0 flex-col bg-surface-3"
				:class="sidebarClass"
			>
				<div
					v-if="!isMobile"
					aria-hidden="true"
					class="pointer-events-none absolute inset-y-0 right-0 border-0 border-r border-solid border-surface-5"
				/>
				<slot name="sidebar-header" />
				<div v-if="layout === 'rail'" class="flex shrink-0 flex-col gap-1 px-3 pb-1">
					<button
						v-tooltip.right="railExpanded ? undefined : formatMessage(messages.expandSidebar)"
						type="button"
						:aria-label="
							formatMessage(railExpanded ? messages.collapseSidebar : messages.expandSidebar)
						"
						:aria-expanded="railExpanded"
						class="relative flex h-11 w-full shrink-0 cursor-pointer items-center gap-2 overflow-hidden rounded-[14px] border-none bg-transparent px-4 text-left text-base font-semibold leading-5 text-primary no-underline transition-all hover:bg-surface-4 hover:text-contrast active:scale-[0.97]"
						@click="railExpanded = !railExpanded"
					>
						<PanelLeftCloseIcon v-if="railExpanded" class="size-5 shrink-0" aria-hidden="true" />
						<PanelLeftOpenIcon v-else class="size-5 shrink-0" aria-hidden="true" />
						<span
							aria-hidden="true"
							class="min-w-0 flex-1 truncate transition-opacity duration-200 ease-out motion-reduce:transition-none"
							:class="railExpanded ? 'opacity-100' : 'opacity-0'"
						>
							{{ formatMessage(messages.collapse) }}
						</span>
					</button>
					<div
						aria-hidden="true"
						class="mx-auto my-1 h-px shrink-0 bg-surface-5 transition-[width] duration-200 ease-out motion-reduce:transition-none"
						:class="railExpanded ? 'w-full' : 'w-8'"
					/>
				</div>

				<div v-if="!isMobile || !hideTabSelection" class="relative min-h-0 flex-1">
					<Transition
						enter-active-class="transition-all duration-200 ease-out"
						enter-from-class="opacity-0 max-h-0"
						enter-to-class="opacity-100 max-h-4"
						leave-active-class="transition-all duration-200 ease-in"
						leave-from-class="opacity-100 max-h-4"
						leave-to-class="opacity-0 max-h-0"
					>
						<div
							v-if="showSidebarTopFade"
							class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-4 bg-gradient-to-b from-surface-3 to-transparent"
						/>
					</Transition>

					<nav
						v-if="layout === 'rail'"
						ref="sidebarScrollContainer"
						class="tabbed-modal-scrollbar flex h-full flex-col gap-1 overflow-y-auto overflow-x-hidden overscroll-contain px-3"
						@scroll="checkSidebarScrollState"
					>
						<template v-for="(tab, index) in visibleTabs" :key="index">
							<div
								v-if="startsGroup(index)"
								aria-hidden="true"
								class="mx-auto my-1 h-px shrink-0 bg-surface-5 transition-[width] duration-200 ease-out motion-reduce:transition-none"
								:class="railExpanded ? 'w-full' : 'w-8'"
							/>
							<div
								v-if="startsCategory(index) && tab.category"
								:aria-hidden="!railExpanded"
								class="grid shrink-0 transition-[grid-template-rows,opacity] duration-200 ease-out motion-reduce:transition-none"
								:class="railExpanded ? 'grid-rows-[1fr] opacity-100' : 'grid-rows-[0fr] opacity-0'"
							>
								<div class="overflow-hidden">
									<div
										class="truncate px-4 pb-1.5 pt-1 text-sm font-extrabold uppercase leading-5 text-secondary"
									>
										{{ formatMessage(tab.category) }}
									</div>
								</div>
							</div>
							<component
								:is="tab.href ? 'a' : 'button'"
								v-tooltip.right="railExpanded ? undefined : formatMessage(tab.name)"
								:href="tab.href ?? undefined"
								:target="tab.href ? '_blank' : undefined"
								:rel="tab.href ? 'noopener noreferrer' : undefined"
								:type="tab.href ? undefined : 'button'"
								:aria-label="tabAccessibleName(tab)"
								:aria-current="isTabSelected(tab, index) ? 'page' : undefined"
								:class="`relative flex h-11 w-full shrink-0 cursor-pointer items-center gap-2 overflow-hidden rounded-[14px] border-none px-4 text-left text-base font-semibold leading-5 no-underline transition-all active:scale-[0.97] ${isTabSelected(tab, index) ? 'bg-[color-mix(in_srgb,var(--color-brand)_30%,transparent)] text-brand' : 'bg-transparent text-primary hover:bg-surface-4 hover:text-contrast'}`"
								@click="!tab.href && setTab(index)"
							>
								<component :is="tab.icon" class="size-5 shrink-0" aria-hidden="true" />
								<span
									aria-hidden="true"
									class="min-w-0 flex-1 truncate transition-opacity duration-200 ease-out motion-reduce:transition-none"
									:class="railExpanded ? 'opacity-100' : 'opacity-0'"
								>
									{{ formatMessage(tab.name) }}
								</span>
								<RightArrowIcon
									v-if="tab.href"
									aria-hidden="true"
									class="size-4 shrink-0 transition-opacity duration-200 ease-out motion-reduce:transition-none"
									:class="railExpanded ? 'opacity-100' : 'opacity-0'"
								/>
								<span
									v-if="tab.badge"
									aria-hidden="true"
									class="shrink-0 rounded-full bg-brand-highlight px-1.5 py-0.5 text-xs font-bold text-brand-green transition-opacity duration-200 ease-out motion-reduce:transition-none"
									:class="railExpanded ? 'opacity-100' : 'opacity-0'"
								>
									{{ formatMessage(tab.badge) }}
								</span>
								<span
									v-if="tab.badge"
									aria-hidden="true"
									class="absolute left-[30px] top-2 size-2 rounded-full bg-brand transition-opacity duration-200 ease-out motion-reduce:transition-none"
									:class="railExpanded ? 'opacity-0' : 'opacity-100'"
								/>
							</component>
						</template>
					</nav>

					<nav
						v-else-if="isMobile"
						ref="sidebarScrollContainer"
						class="tabbed-modal-scrollbar flex h-full flex-col gap-4 overflow-y-auto overscroll-contain"
						:style="{ paddingBottom: floatingActionBarShown ? contentBottomPadding : undefined }"
						@scroll="checkSidebarScrollState"
					>
						<div
							v-for="(group, groupIndex) in mobileTabGroups"
							:key="groupIndex"
							class="flex shrink-0 flex-col gap-2"
						>
							<div
								v-if="startsCategory(group[0].index) && group[0].tab.category"
								class="truncate px-4 text-sm font-extrabold uppercase leading-5 text-secondary"
							>
								{{ formatMessage(group[0].tab.category) }}
							</div>
							<div
								class="flex flex-col overflow-hidden rounded-2xl border border-solid border-surface-5 bg-surface-2"
							>
								<component
									:is="tab.href ? 'a' : 'button'"
									v-for="({ tab, index }, rowIndex) in group"
									:key="index"
									:ref="(element) => setMobileTabRef(index, element)"
									:href="tab.href ?? undefined"
									:target="tab.href ? '_blank' : undefined"
									:rel="tab.href ? 'noopener noreferrer' : undefined"
									:type="tab.href ? undefined : 'button'"
									:class="`flex min-h-[52px] w-full cursor-pointer items-center gap-3.5 border-0 border-solid border-surface-4 bg-transparent py-0 pl-4 pr-3.5 text-left text-base font-semibold text-contrast no-underline transition-colors hover:bg-surface-3 active:bg-surface-4 ${rowIndex < group.length - 1 ? 'border-b' : ''}`"
									@click="!tab.href && openTab(index)"
								>
									<component
										:is="tab.icon"
										class="size-5 shrink-0 text-primary"
										aria-hidden="true"
									/>
									<span class="min-w-0 flex-1 truncate">{{ formatMessage(tab.name) }}</span>
									<span
										v-if="tab.badge"
										class="shrink-0 rounded-full px-1.5 py-0.5 text-xs font-bold bg-brand-highlight text-brand-green"
									>
										{{ formatMessage(tab.badge) }}
									</span>
									<ArrowUpRightIcon
										v-if="tab.href"
										class="size-[18px] shrink-0 text-secondary"
										aria-hidden="true"
									/>
									<ChevronRightIcon
										v-else
										class="size-[18px] shrink-0 text-secondary"
										aria-hidden="true"
									/>
								</component>
							</div>
						</div>
						<slot name="footer" />
					</nav>

					<div
						v-else
						ref="sidebarScrollContainer"
						class="tabbed-modal-scrollbar -mr-3 flex h-full flex-col gap-1 overflow-y-auto overscroll-contain pr-3"
						@scroll="checkSidebarScrollState"
					>
						<template v-for="(tab, index) in visibleTabs" :key="index">
							<div
								v-if="startsCategory(index) && tab.category"
								class="shrink-0 truncate pb-1.5 text-sm font-extrabold uppercase leading-5 text-secondary"
								:class="{ 'mt-3': index > 0 }"
							>
								{{ formatMessage(tab.category) }}
							</div>
							<component
								:is="tab.href ? 'a' : 'button'"
								:href="tab.href ?? undefined"
								:target="tab.href ? '_blank' : undefined"
								:rel="tab.href ? 'noopener noreferrer' : undefined"
								:class="`flex min-w-0 shrink-0 gap-2 items-center text-left rounded-[14px] px-4 py-2.5 border-none text-base leading-5 font-semibold cursor-pointer active:scale-[0.97] transition-all no-underline ${isTabSelected(tab, index) ? 'bg-[color-mix(in_srgb,var(--color-brand)_30%,transparent)] text-brand' : 'bg-transparent text-primary hover:bg-surface-4 hover:text-contrast'}`"
								@click="!tab.href && setTab(index)"
							>
								<component :is="tab.icon" class="size-5 flex-shrink-0" />
								<span
									:ref="(element) => setTabLabelRef(index, element)"
									v-tooltip="tabLabelTooltip(index, formatMessage(tab.name))"
									class="min-w-0 flex-1 truncate"
								>
									{{ formatMessage(tab.name) }}
								</span>
								<span
									v-if="tab.badge"
									class="shrink-0 rounded-full px-1.5 py-0.5 text-xs font-bold bg-brand-highlight text-brand-green"
								>
									{{ formatMessage(tab.badge) }}
								</span>
								<RightArrowIcon v-if="tab.href" class="ml-auto size-4 shrink-0" />
							</component>
						</template>
					</div>

					<Transition
						enter-active-class="transition-all duration-200 ease-out"
						enter-from-class="opacity-0 max-h-0"
						enter-to-class="opacity-100 max-h-14"
						leave-active-class="transition-all duration-200 ease-in"
						leave-from-class="opacity-100 max-h-14"
						leave-to-class="opacity-0 max-h-0"
					>
						<div
							v-if="showSidebarBottomFade"
							class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-14 bg-gradient-to-t from-surface-3 via-surface-3 via-20% to-transparent"
						/>
					</Transition>
				</div>

				<slot v-if="!isMobile" name="footer" />
			</div>
			<div
				v-show="showContent"
				class="relative min-h-0 min-w-0 bg-surface-2"
				:class="{ 'flex-1': isMobile }"
			>
				<Transition
					enter-active-class="transition-all duration-200 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-4"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-4"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="showTopFade"
						class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-4 bg-gradient-to-b from-surface-2 to-transparent"
					/>
				</Transition>

				<div
					ref="scrollContainer"
					class="tabbed-modal-scrollbar absolute inset-0 overflow-y-auto overscroll-contain"
					@scroll="checkScrollState"
				>
					<div
						class="flow-root min-h-full pt-6"
						:class="isMobile ? 'px-5' : 'px-6'"
						:style="{ paddingBottom: contentBottomPadding }"
					>
						<slot name="content" :tab="visibleTabs[selectedTab]" :index="selectedTab">
							<Suspense>
								<component
									:is="visibleTabs[selectedTab]?.content"
									v-if="visibleTabs[selectedTab]?.content"
								/>
								<template #fallback>
									<LoadingIndicator class="py-2" />
								</template>
							</Suspense>
						</slot>
					</div>
				</div>

				<Transition
					enter-active-class="transition-all duration-200 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-14"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-14"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="showBottomFade"
						class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-14 bg-gradient-to-t from-surface-2 to-transparent"
					/>
				</Transition>
			</div>

			<div
				ref="floatingActionBarContainer"
				class="pointer-events-none absolute z-20"
				:class="floatingActionBarClass"
			>
				<div class="pointer-events-auto">
					<slot name="floating-action-bar" />
				</div>
			</div>
		</div>
	</NewModal>
</template>

<style scoped>
.tabbed-modal-scrollbar {
	scrollbar-color: var(--surface-5) transparent;
	scrollbar-width: thin;
}

.tabbed-modal-scrollbar::-webkit-scrollbar {
	width: 4px;
	height: 4px;
}

.tabbed-modal-scrollbar::-webkit-scrollbar-thumb {
	border-radius: 8px;
	background: var(--surface-5);
}
</style>
