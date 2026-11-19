import { onMounted, onScopeDispose, ref } from 'vue'
import { type RouteLocationNormalized, type RouteLocationResolved, useRouter } from 'vue-router'

export function useRouteExitTransition(
	shouldAnimate: (to: RouteLocationNormalized, from: RouteLocationNormalized) => boolean,
	duration: number,
) {
	const router = useRouter()
	const leaving = ref(false)
	const cleanups: (() => void)[] = []

	onMounted(() => {
		cleanups.push(
			router.beforeEach(async (to, from) => {
				if (leaving.value || to.path === from.path || !shouldAnimate(to, from)) return
				if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return
				leaving.value = true
				await new Promise((resolve) => setTimeout(resolve, duration))
			}),
			router.afterEach((_to, _from, failure) => {
				if (failure) leaving.value = false
			}),
		)
	})

	onScopeDispose(() => cleanups.forEach((cleanup) => cleanup()))

	return leaving
}

export function useRouteEnterTransition(enabled: boolean) {
	const entering = ref(enabled)

	onMounted(() => {
		if (!entering.value) return
		requestAnimationFrame(() => requestAnimationFrame(() => (entering.value = false)))
	})

	return entering
}

export function usePreviousRoute(): RouteLocationResolved | null {
	const router = useRouter()
	const back = router.options.history.state?.back
	return typeof back === 'string' ? router.resolve(back) : null
}
