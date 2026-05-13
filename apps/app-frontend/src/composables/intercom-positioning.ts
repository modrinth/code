import { computed, onUnmounted, ref } from 'vue'
import type { RouteLocationNormalizedLoaded } from 'vue-router'

interface ThemeStore {
	toggleSidebar: boolean
	$subscribe: (callback: () => void) => () => void
}

interface IntercomBubblePosition {
	horizontalPadding: number
	verticalPadding: number
}

const APP_LEFT_NAV_WIDTH = '4rem'
const APP_SIDEBAR_WIDTH = 300
const INTERCOM_BUBBLE_DEFAULT_PADDING = 20
const INTERCOM_BUBBLE_WIDTH = 72
const INTERCOM_BUBBLE_RIGHT_VAR = '--app-support-launcher-right'
const INTERCOM_BUBBLE_BOTTOM_VAR = '--app-support-launcher-bottom'

export function useIntercomPositioning({
	route,
	themeStore,
}: {
	route: RouteLocationNormalizedLoaded
	themeStore: ThemeStore
}) {
	const sidebarToggled = ref(true)
	const unsubscribeSidebarToggle = themeStore.$subscribe(() => {
		sidebarToggled.value = !themeStore.toggleSidebar
	})

	onUnmounted(unsubscribeSidebarToggle)

	const forceSidebar = computed(
		() => route.path.startsWith('/browse') || route.path.startsWith('/project'),
	)
	const sidebarVisible = computed(() => sidebarToggled.value || forceSidebar.value)
	const defaultIntercomBubbleHorizontalPadding = computed(() =>
		sidebarVisible.value
			? APP_SIDEBAR_WIDTH + INTERCOM_BUBBLE_DEFAULT_PADDING
			: INTERCOM_BUBBLE_DEFAULT_PADDING,
	)
	const intercomBubbleRequestedHorizontalPadding = ref<number | null>(null)
	const intercomBubbleHorizontalPadding = computed(
		() =>
			intercomBubbleRequestedHorizontalPadding.value ??
			defaultIntercomBubbleHorizontalPadding.value,
	)
	const intercomBubbleVerticalClearance = ref<number | null>(null)
	const intercomBubblePosition = computed(() => ({
		horizontalPadding: intercomBubbleHorizontalPadding.value,
		verticalPadding: intercomBubbleVerticalClearance.value ?? INTERCOM_BUBBLE_DEFAULT_PADDING,
	}))
	const intercomBubbleHorizontalPaddingRequests = new Map<symbol, number>()
	const intercomBubbleClearanceRequests = new Map<symbol, number>()

	function requestIntercomBubbleHorizontalPadding(id: symbol, padding: number | null) {
		if (padding === null) {
			intercomBubbleHorizontalPaddingRequests.delete(id)
		} else {
			intercomBubbleHorizontalPaddingRequests.set(id, padding)
		}

		intercomBubbleRequestedHorizontalPadding.value =
			intercomBubbleHorizontalPaddingRequests.size > 0
				? Math.max(...intercomBubbleHorizontalPaddingRequests.values())
				: null
	}

	function requestIntercomBubbleVerticalClearance(id: symbol, clearance: number | null) {
		if (clearance === null) {
			intercomBubbleClearanceRequests.delete(id)
		} else {
			intercomBubbleClearanceRequests.set(id, clearance)
		}

		intercomBubbleVerticalClearance.value =
			intercomBubbleClearanceRequests.size > 0
				? Math.max(...intercomBubbleClearanceRequests.values())
				: null
	}

	function updateIntercomBubbleStyles({
		horizontalPadding,
		verticalPadding,
	}: IntercomBubblePosition) {
		if (typeof document === 'undefined') return

		document.documentElement.style.setProperty(INTERCOM_BUBBLE_RIGHT_VAR, `${horizontalPadding}px`)
		document.documentElement.style.setProperty(INTERCOM_BUBBLE_BOTTOM_VAR, `${verticalPadding}px`)
	}

	function clearIntercomBubbleStyles() {
		if (typeof document === 'undefined') return

		document.documentElement.style.removeProperty(INTERCOM_BUBBLE_RIGHT_VAR)
		document.documentElement.style.removeProperty(INTERCOM_BUBBLE_BOTTOM_VAR)
	}

	return {
		sidebarToggled,
		forceSidebar,
		sidebarVisible,
		intercomBubblePosition,
		updateIntercomBubbleStyles,
		clearIntercomBubbleStyles,
		pageContext: {
			floatingActionBarOffsets: {
				left: ref(APP_LEFT_NAV_WIDTH),
				right: computed(() => (sidebarVisible.value ? `${APP_SIDEBAR_WIDTH}px` : '0px')),
			},
			intercomBubble: {
				width: ref(INTERCOM_BUBBLE_WIDTH),
				horizontalPadding: intercomBubbleHorizontalPadding,
				requestHorizontalPadding: requestIntercomBubbleHorizontalPadding,
				requestVerticalClearance: requestIntercomBubbleVerticalClearance,
			},
		},
	}
}
