import { computed, onMounted, onScopeDispose, onUnmounted, ref, shallowRef } from 'vue'
import { type RouteLocationNormalized, type Router, useRouter } from 'vue-router'

import {
	injectRouteScope,
	type LinkedRoute,
	type RouteExitPredicate,
	type RouteScopeContext,
} from '#ui/providers/route-scope'

const EXIT_DURATION_MS = 200

function resolvePreviousRoute(router: Router) {
	const back = router.options.history.state?.back
	return typeof back === 'string' ? router.resolve(back) : null
}

export function useRouteScope(): RouteScopeContext {
	const router = useRouter()
	const leavingTo = shallowRef<RouteLocationNormalized | null>(null)
	const exitPredicates = new Set<RouteExitPredicate>()

	const cleanups: (() => void)[] = []
	onMounted(() => {
		cleanups.push(
			router.beforeEach(async (to, from) => {
				if (leavingTo.value || to.path === from.path) return
				if (![...exitPredicates].some((shouldAnimate) => shouldAnimate(to))) return
				if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return
				leavingTo.value = to
				await new Promise((resolve) =>
					requestAnimationFrame(() => setTimeout(resolve, EXIT_DURATION_MS)),
				)
			}),
			router.afterEach((_to, _from, failure) => {
				if (failure) leavingTo.value = null
			}),
		)
	})
	onScopeDispose(() => cleanups.forEach((cleanup) => cleanup()))

	return {
		previousRoute: resolvePreviousRoute(router),
		leavingTo,
		registerExitTransition(shouldAnimate) {
			exitPredicates.add(shouldAnimate)
			return () => exitPredicates.delete(shouldAnimate)
		},
	}
}

export function usePreviousRoute() {
	return injectRouteScope(null)?.previousRoute ?? resolvePreviousRoute(useRouter())
}

export interface RouteLinkTransition {
	arrived: boolean
	leaving: boolean
	active: boolean
	route: LinkedRoute | null
}

export function useRouteLinkTransition(isLinked: RouteExitPredicate) {
	const scope = injectRouteScope(null) ?? useRouteScope()
	const previousRoute = usePreviousRoute()
	const origin = previousRoute && isLinked(previousRoute) ? previousRoute : null
	const entering = useRouteEnterTransition(!!origin)

	let unregister: (() => void) | undefined
	onMounted(() => (unregister = scope.registerExitTransition(isLinked)))
	onUnmounted(() => unregister?.())

	return computed<RouteLinkTransition>(() => {
		const to = scope.leavingTo.value
		const destination = to && isLinked(to) ? to : null
		return {
			arrived: !!origin,
			leaving: !!destination,
			active: entering.value || !!destination,
			route: destination ?? origin,
		}
	})
}

export function useRouteEnterTransition(enabled: boolean) {
	const entering = ref(enabled)

	onMounted(() => {
		if (!entering.value) return
		requestAnimationFrame(() => requestAnimationFrame(() => (entering.value = false)))
	})

	return entering
}
