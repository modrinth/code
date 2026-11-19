import type { Ref } from 'vue'
import type { RouteLocationNormalized, RouteLocationResolved } from 'vue-router'

import { createContext } from './create-context'

export type RouteExitPredicate = (to: RouteLocationNormalized) => boolean

export interface RouteScopeContext {
	previousRoute: RouteLocationResolved | null
	leavingTo: Readonly<Ref<RouteLocationNormalized | null>>
	registerExitTransition: (shouldAnimate: RouteExitPredicate) => () => void
}

export const [injectRouteScope, provideRouteScope] = createContext<RouteScopeContext>(
	'RouteScope',
	'routeScopeContext',
)
