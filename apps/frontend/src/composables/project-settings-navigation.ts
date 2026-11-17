import { createContext } from '@modrinth/ui'
import { useMutationObserver } from '@vueuse/core'
import { nextTick, onScopeDispose, type Ref, watch } from 'vue'
import { isNavigationFailure, NavigationFailureType } from 'vue-router'

const [injectNavigation, provideNavigation] = createContext<{
	reveal: (path: string, field?: string) => Promise<void>
}>('ProjectSettingsNavigation')

export const injectProjectSettingsNavigation = injectNavigation

export function provideProjectSettingsNavigation(root: Ref<HTMLElement | null>) {
	const router = useRouter()
	let pendingField: string | undefined
	let pendingPath: string | undefined
	let sequence = 0
	let animation: Animation | undefined

	const removeNavigationGuard = router.beforeResolve((to) => {
		if (to.fullPath === pendingPath) to.meta.scrollToTop = false
	})

	function highlight() {
		if (!pendingField || !root.value) return
		const element = [...root.value.querySelectorAll<HTMLElement>('[data-project-field]')].find(
			(element) =>
				element.dataset.projectField === pendingField && element.getClientRects().length > 0,
		)
		const view = element?.ownerDocument.defaultView
		if (!element || !view) return
		pendingField = undefined
		animation?.cancel()
		const reducedMotion = view.matchMedia('(prefers-reduced-motion: reduce)').matches
		const fitsViewport = element.getBoundingClientRect().height < view.innerHeight
		element.scrollIntoView({
			behavior: reducedMotion ? 'instant' : 'smooth',
			block: fitsViewport ? 'center' : 'start',
		})
		const backgroundColor = 'var(--color-orange-highlight)'
		animation = element.animate(
			[
				{ backgroundColor },
				{ backgroundColor, offset: 0.7 },
				{ backgroundColor: view.getComputedStyle(element).backgroundColor },
			],
			{ duration: reducedMotion ? 1800 : 2400, easing: 'ease-out' },
		)
	}

	watch(
		() => router.currentRoute.value.fullPath,
		() => {
			pendingField = undefined
			animation?.cancel()
		},
		{ flush: 'sync' },
	)
	useMutationObserver(root, highlight, { childList: true, subtree: true })
	onScopeDispose(() => {
		sequence++
		removeNavigationGuard()
		animation?.cancel()
	})
	provideNavigation({
		async reveal(path, field) {
			const request = ++sequence
			pendingField = undefined
			pendingPath = field ? router.resolve(path).fullPath : undefined
			const failure = await router.push(path).finally(() => {
				if (request === sequence) pendingPath = undefined
			})
			if (
				request !== sequence ||
				(failure && !isNavigationFailure(failure, NavigationFailureType.duplicated))
			)
				return
			pendingField = field
			await nextTick()
			highlight()
		},
	})
}
