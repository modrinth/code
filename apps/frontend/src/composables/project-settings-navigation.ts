import { createContext } from '@modrinth/ui'
import { useMutationObserver } from '@vueuse/core'
import { nextTick, onScopeDispose, type Ref, useId, watch } from 'vue'
import { isNavigationFailure, NavigationFailureType } from 'vue-router'

const [injectNavigation, provideNavigation] = createContext<{
	reveal: (path: string, field?: string) => Promise<void>
	moderationDrawerId: string
	openModerationChecklist: () => void
}>('ProjectSettingsNavigation')

export const injectProjectSettingsNavigation = injectNavigation

export function provideProjectSettingsNavigation(
	root: Ref<HTMLElement | null>,
	openModerationChecklist: () => void,
) {
	const router = useRouter()
	const moderationDrawerId = useId()
	let pendingField: string | undefined
	let pendingPath: string | undefined
	let sequence = 0
	let clearHighlight: (() => void) | undefined

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
		clearHighlight?.()
		const reducedMotion = view.matchMedia('(prefers-reduced-motion: reduce)').matches
		const fitsViewport = element.getBoundingClientRect().height < view.innerHeight
		element.scrollIntoView({
			behavior: reducedMotion ? 'instant' : 'smooth',
			block: fitsViewport ? 'center' : 'start',
		})
		if (element.hasAttribute('data-project-field-scroll-only')) return

		const outline = element.ownerDocument.createElement('span')
		outline.className = 'project-field-highlight-outline'
		outline.setAttribute('aria-hidden', 'true')
		const needsPosition = view.getComputedStyle(element).position === 'static'
		if (needsPosition) element.classList.add('project-field-highlight-anchor')
		element.append(outline)

		const holdDuration = 1000
		const fadeDuration = reducedMotion ? 0 : 200
		const duration = holdDuration + fadeDuration
		const animation = outline.animate(
			reducedMotion
				? [{ opacity: 1 }, { opacity: 1 }]
				: [{ opacity: 1 }, { opacity: 1, offset: holdDuration / duration }, { opacity: 0 }],
			{ duration },
		)
		function removeOutline() {
			outline.remove()
			if (needsPosition) element.classList.remove('project-field-highlight-anchor')
			clearHighlight = undefined
		}
		clearHighlight = () => {
			animation.onfinish = null
			animation.cancel()
			removeOutline()
		}
		animation.onfinish = removeOutline
	}

	watch(
		() => router.currentRoute.value.fullPath,
		() => {
			pendingField = undefined
			clearHighlight?.()
		},
		{ flush: 'sync' },
	)
	useMutationObserver(root, highlight, { childList: true, subtree: true })
	onScopeDispose(() => {
		sequence++
		removeNavigationGuard()
		clearHighlight?.()
	})
	provideNavigation({
		moderationDrawerId,
		openModerationChecklist,
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
