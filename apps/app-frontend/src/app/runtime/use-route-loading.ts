import { onScopeDispose, type Ref, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { trackEvent } from '@/helpers/analytics'
import { debugStartup } from '@/helpers/startup-debug'

import { useAppLoadingState } from './use-app-loading-state'

type RouteLoadingOptions = {
	stateInitialized: Readonly<Ref<boolean>>
	stateFailed: Readonly<Ref<boolean>>
}

export function useRouteLoading({ stateInitialized, stateFailed }: RouteLoadingOptions) {
	const router = useRouter()
	const route = useRoute()
	const canNavigateBack = ref(false)
	const canNavigateForward = ref(false)
	let initialLoadToken: symbol | null = null
	let routerToken: symbol | null = null
	let suspenseToken: symbol | null = null
	let suspensePending = false
	const releaseTimers = new Set<ReturnType<typeof setTimeout>>()

	const loading = useAppLoadingState(() => ({
		stateInitialized: stateInitialized.value,
		stateFailed: stateFailed.value,
		initialStatePending: !!initialLoadToken,
		navigationPending: !!routerToken,
		routeSuspensePending: !!suspenseToken,
		route: route.path,
	}))
	loading.setEnabled(false)
	initialLoadToken = loading.begin('Initial app state')

	function updateHistoryNavigationState() {
		const historyState = window.history.state
		canNavigateBack.value = historyState?.back != null
		canNavigateForward.value = historyState?.forward != null
	}

	function releaseInitialAndNavigationTokens() {
		if (initialLoadToken) {
			loading.end(initialLoadToken)
			initialLoadToken = null
		}
		if (routerToken) {
			loading.end(routerToken)
			routerToken = null
		}
	}

	updateHistoryNavigationState()

	const removeBeforeEach = router.beforeEach((to, from) => {
		debugStartup('Route navigation started', { to: to.path, from: from.path })
		suspensePending = false
		if (routerToken) loading.end(routerToken)
		routerToken = loading.begin(`Route navigation: ${to.path}`)
	})
	const removeAfterEach = router.afterEach((to, from, failure) => {
		debugStartup('Route navigation settled', { to: to.path, failed: !!failure })
		updateHistoryNavigationState()
		trackEvent('PageView', {
			path: to.path,
			fromPath: from.path,
			failed: !!failure,
		})
		const timer = setTimeout(() => {
			releaseTimers.delete(timer)
			debugStartup('Route loading release check', {
				route: to.path,
				suspensePending,
				stateInitialized: stateInitialized.value,
			})
			if (!suspensePending && stateInitialized.value) {
				releaseInitialAndNavigationTokens()
			}
		}, 100)
		releaseTimers.add(timer)
	})

	function onSuspensePending() {
		debugStartup('Route Suspense pending', { route: route.path })
		suspensePending = true
		if (suspenseToken) loading.end(suspenseToken)
		suspenseToken = loading.begin(`Route Suspense: ${route.path}`)
	}

	function onSuspenseResolve() {
		debugStartup('Route Suspense resolved', { route: route.path })
		if (suspenseToken) {
			loading.end(suspenseToken)
			suspenseToken = null
		}
		if (routerToken) {
			loading.end(routerToken)
			routerToken = null
		}
	}

	watch(stateInitialized, (ready) => {
		debugStartup('State readiness changed', { ready })
		if (ready) releaseInitialAndNavigationTokens()
	})

	onScopeDispose(() => {
		removeBeforeEach()
		removeAfterEach()
		for (const timer of releaseTimers) clearTimeout(timer)
		releaseTimers.clear()
		releaseInitialAndNavigationTokens()
		if (suspenseToken) {
			loading.end(suspenseToken)
			suspenseToken = null
		}
	})

	return { loading, canNavigateBack, canNavigateForward, onSuspensePending, onSuspenseResolve }
}
