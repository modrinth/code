<template>
	<div
		ref="panelRoot"
		class="flex min-h-[calc(var(--files-viewer-height)_+_var(--files-navbar-height,3rem)_+_18px)] w-full scroll-mt-[var(--files-sticky-top,0px)] flex-col gap-3 p-1 pl-2 [overflow-y:clip]"
		:class="{ 'snap-start': isFileActive }"
		:style="{
			'--files-navbar-height': `${navbarHeight}px`,
			'--files-trailing-space': `${trailingSpace}px`,
			'--files-table-header-top': `calc(var(--files-sticky-top, 0px) + ${navbarHeight}px + ${tabStripHeight}px)`,
			'--files-viewer-height':
				'max(24rem, calc(var(--files-viewport-height, 100dvh) - var(--files-sticky-top, 0px) - var(--files-navbar-height, 3rem) - 18px - var(--files-trailing-space, 0px)))',
		}"
	>
		<div
			ref="navbarWrapper"
			class="sticky top-[var(--files-sticky-top,0px)] z-[29] bg-surface-1 py-1 pr-[calc(0.75rem+1px)] before:pointer-events-none before:top-[calc(-1.5rem_+_1px)] before:absolute before:inset-x-0 before:h-[var(--files-sticky-top,0px)] before:bg-[var(--color-bg,_var(--surface-1))] before:content-['']"
		>
			<FileNavbar
				:sidebar-open="sidebarOpen"
				:active-location="ui.activeLocation.value"
				:is-editor-find-open="ui.fileEditorApi.value?.isFindOpen.value ?? false"
				:search-query="view.searchQuery.value"
				:show-refresh-button="ui.showRefreshButton.value"
				:show-install-from-url="ui.showInstallFromUrl"
				:base-id="ui.baseId"
				:disabled="ui.isBusy.value"
				:disabled-tooltip="ui.busyTooltip.value"
				:small-mode="props.smallMode"
				:can-go-back="ui.fileTabs.canGoBack.value"
				:can-go-forward="ui.fileTabs.canGoForward.value"
				:sidebar-toggleable="ui.advancedView.value && toggleLocation != null"
				:sidebar-toggle-always="toggleLocation === 'navigation'"
				:compact="ui.compactNavigation.value"
				@back="ui.fileTabs.back"
				@forward="ui.fileTabs.forward"
				@navigate="ui.navigateToSegment"
				@navigate-home="() => ui.navigateToSegment(0)"
				@prefetch-home="ui.handleHomePrefetch"
				@update:search-query="(value) => (view.searchQuery.value = value)"
				@create="ui.showCreateModal"
				@upload="ui.initiateFileUpload"
				@upload-zip="() => {}"
				@unzip-from-url="ui.showUnzipFromUrlModal"
				@refresh="ui.refresh"
				@share="() => ui.shareEditorToMclogs()"
				@find="() => ui.toggleFind()"
				@toggle-sidebar="() => ui.setSidebarOpen(!sidebarOpen)"
				@contextmenu="openBreadcrumbMenu"
			/>
		</div>
		<div
			class="@container relative flex flex-col overflow-clip rounded-[20px] border border-solid border-surface-4 shadow-sm"
		>
			<div
				class="shrink-0 overflow-hidden"
				:class="
					isFileActive
						? 'h-[var(--files-viewer-height)]'
						: ui.advancedView.value
							? 'sticky top-[calc(var(--files-sticky-top,0px)_+_var(--files-navbar-height,0px))] z-20 h-9'
							: 'h-0'
				"
			>
				<FileTabs />
			</div>
			<FileEmptyWorkspace v-if="!hasTabs" />
			<FileListing
				v-show="hasTabs && !isFileActive"
				ref="listing"
				:view="view"
				:header-sticky-top="tableHeaderStickyTop"
				@navigate="ui.navigateTo"
			/>
		</div>
	</div>
</template>

<script setup lang="ts">
import { useElementSize, useEventListener } from '@vueuse/core'
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import { findScrollableAncestor } from '#ui/composables/virtual-scroll.ts'
import { injectLoadingState } from '#ui/providers/loading-state'

import { useEntryMenu } from '../composables/entry-menu'
import { useSidebarToggleLocation } from '../composables/sidebar-toggle'
import { injectFileBrowserUI } from '../providers/file-browser-ui'
import { parentInfoFrom } from '../utils'
import FileEmptyWorkspace from './FileEmptyWorkspace.vue'
import FileListing from './FileListing.vue'
import FileNavbar from './FileNavbar.vue'
import FileTabs from './tabs/FileTabs.vue'

/**
 * The files tab laid out within a scrolling page: the navbar, the tab strip and, below it, the
 * active tab's listing or editor. Hosts using the workbench show each tab's own view instead.
 */
const ui = injectFileBrowserUI()
const view = computed(() => ui.activeView.value)

const props = withDefaults(
	defineProps<{
		smallMode?: boolean
	}>(),
	{
		smallMode: false,
	},
)

const sidebarOpen = computed(() => ui.sidebarOpen.value)
const toggleLocation = useSidebarToggleLocation()
const isFileActive = computed(() => ui.fileTabs.activeLocation.value.type === 'file')
/** In the advanced view every tab can be closed, which leaves an empty workspace. */
const hasTabs = computed(() => ui.fileTabs.tabs.value.length > 0)

const { openMenu } = useEntryMenu()

/** Right-click on a breadcrumb, `depth` segments deep (the last one is the active location). */
function openBreadcrumbMenu(event: MouseEvent, depth: number) {
	const location = ui.activeLocation.value
	const segments = location.path.split('/').filter(Boolean)
	openMenu(event, depth >= segments.length ? location : parentInfoFrom(location, depth))
}

const loadingState = injectLoadingState(null)
let refreshToken: symbol | null = null

function endRefreshLoading() {
	if (refreshToken) loadingState?.end(refreshToken)
	refreshToken = null
}

watch(
	() => ui.isRefreshing.value,
	(refreshing) => {
		if (!refreshing) return endRefreshLoading()
		if (loadingState && !refreshToken) refreshToken = loadingState.begin()
	},
	{ immediate: true },
)
onBeforeUnmount(endRefreshLoading)

const listing = ref<InstanceType<typeof FileListing>>()

const TAB_STRIP_HEIGHT = 40
/** The tab strip is hidden, along with the tabs feature, while the rework is disabled. */
const tabStripHeight = computed(() => (ui.advancedView.value ? TAB_STRIP_HEIGHT : 0))

const panelRoot = ref<HTMLElement | null>(null)
const navbarWrapper = ref<HTMLElement | null>(null)
const { height: navbarHeight } = useElementSize(navbarWrapper, undefined, { box: 'border-box' })

/** Where the table header sticks: below the host's sticky offset, the sticky navbar and the tab strip. */
const tableHeaderStickyTop = computed(() => {
	const hostOffset = navbarWrapper.value
		? parseFloat(getComputedStyle(navbarWrapper.value).top) || 0
		: 0
	return hostOffset + navbarHeight.value + tabStripHeight.value
})

/** Brings the navbar and file viewer to the top of the scroll area, unless they're already there. */
function scrollPanelIntoView() {
	const panel = panelRoot.value
	if (!panel) return

	const container = findScrollableAncestor(panel)
	const containerTop = container instanceof Window ? 0 : container.getBoundingClientRect().top
	const scrollMargin = parseFloat(getComputedStyle(panel).scrollMarginTop) || 0
	if (Math.abs(panel.getBoundingClientRect().top - containerTop - scrollMargin) < 2) return

	panel.scrollIntoView({ block: 'start', behavior: 'smooth' })
}

const activeFileKey = computed(() =>
	isFileActive.value
		? `${ui.fileTabs.activeTabId.value}:${ui.fileTabs.activeLocation.value.path}`
		: null,
)

/**
 * Scroll space below the file viewer within the page's content (stopping at `<main>`, so a site
 * footer doesn't count). The viewer's height (the editor's, and the listing's minimum) is shortened
 * by this much so that scrolling to the end of the page can never push the tab strip under the
 * sticky navbar, and switching between files and folders keeps the page the same height. Parents'
 * `min-height` slack is deliberately ignored, since it shrinks as the viewer grows.
 */
const trailingSpace = ref(0)

function measureTrailingSpace() {
	const panel = panelRoot.value
	if (!panel) return

	const scrollContainer = findScrollableAncestor(panel)
	const boundary =
		panel.closest('main') ?? (scrollContainer instanceof Window ? document.body : scrollContainer)

	let total = 0
	let element: HTMLElement = panel
	while (element !== boundary && element.parentElement) {
		const parent = element.parentElement
		const elementBottom = element.getBoundingClientRect().bottom
		let contentBottom = elementBottom + (parseFloat(getComputedStyle(element).marginBottom) || 0)

		for (let sibling = element.nextElementSibling; sibling; sibling = sibling.nextElementSibling) {
			const style = getComputedStyle(sibling)
			if (style.display === 'none' || style.position === 'absolute' || style.position === 'fixed') {
				continue
			}
			const siblingBottom =
				sibling.getBoundingClientRect().bottom + (parseFloat(style.marginBottom) || 0)
			contentBottom = Math.max(contentBottom, siblingBottom)
		}

		const parentStyle = getComputedStyle(parent)
		total +=
			contentBottom -
			elementBottom +
			(parseFloat(parentStyle.paddingBottom) || 0) +
			(parseFloat(parentStyle.borderBottomWidth) || 0)
		element = parent
	}

	trailingSpace.value = Math.max(0, Math.round(total))
}

watch(activeFileKey, (key) => {
	if (!key) return
	nextTick(() => {
		measureTrailingSpace()
		nextTick(scrollPanelIntoView)
	})
})

watch(navbarHeight, () => nextTick(measureTrailingSpace))

useEventListener('resize', measureTrailingSpace)

/**
 * While a file is open, the scroll container snaps (by proximity) to the file viewer, so it is
 * easy to bring the whole viewer into view without lining it up by hand.
 */
let snapContainer: HTMLElement | null = null
let previousSnapType = ''

function setViewerSnapping(enabled: boolean) {
	if (enabled && !snapContainer && panelRoot.value) {
		const container = findScrollableAncestor(panelRoot.value)
		snapContainer = container instanceof Window ? document.documentElement : container
		previousSnapType = snapContainer.style.scrollSnapType
		snapContainer.style.scrollSnapType = 'y proximity'
	} else if (!enabled && snapContainer) {
		snapContainer.style.scrollSnapType = previousSnapType
		snapContainer = null
	}
}

watch(isFileActive, setViewerSnapping, { flush: 'post' })

/** The listing stays mounted (hidden) while a file is open, so it has to re-measure when shown again. */
watch(isFileActive, (active) => {
	if (!active) nextTick(() => listing.value?.syncScrollState())
})
onMounted(() => {
	setViewerSnapping(isFileActive.value)
	measureTrailingSpace()
	if (isFileActive.value) nextTick(scrollPanelIntoView)
})
onBeforeUnmount(() => setViewerSnapping(false))
</script>
