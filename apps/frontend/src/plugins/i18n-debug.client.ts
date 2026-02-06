import { I18N_DEBUG_KEY, type I18nDebugContext } from '@modrinth/ui'

export default defineNuxtPlugin({
	name: 'i18n-debug',
	enforce: 'post',
	setup(nuxtApp) {
		const flags = useFeatureFlags()
		if (!flags.value.i18nDebug) return

		const enabled = ref(true)
		const keyReveal = ref(false)
		const registry = reactive(new Map()) as Map<string, { key: string; value: string; defaultMessage?: string; timestamp: number }>
		const panelOpen = ref(false)

		const context: I18nDebugContext = { enabled, keyReveal, registry, panelOpen }
		nuxtApp.vueApp.provide(I18N_DEBUG_KEY, context)

		import('@modrinth/ui/src/assets/i18n-debug.css')

		nuxtApp.hook('app:mounted', () => {
			document.body.classList.add('i18n-debug')
			startMutationObserver(registry, keyReveal)
			setupKeyTooltip()
			registerKeyboardShortcuts(panelOpen, keyReveal)
		})

		watch(
			() => flags.value.i18nDebug,
			(active) => {
				enabled.value = active
				if (!active) {
					keyReveal.value = false
					panelOpen.value = false
					document.body.classList.remove('i18n-debug')
					clearAllAnnotations()
					hideKeyTooltip()
					registry.clear()
				} else {
					document.body.classList.add('i18n-debug')
					annotateFullDocument(registry)
				}
			},
		)
	},
})

function startMutationObserver(
	registry: Map<string, { key: string; value: string; defaultMessage?: string; timestamp: number }>,
	keyReveal: Ref<boolean>,
) {
	let pending = false

	const observer = new MutationObserver((mutations) => {
		if (pending || keyReveal.value) return
		pending = true
		requestAnimationFrame(() => {
			pending = false
			if (!keyReveal.value) {
				processMutations(mutations, registry)
			}
		})
	})

	observer.observe(document.body, {
		childList: true,
		subtree: true,
		characterData: true,
	})

	// Re-annotate whenever the registry grows (keys register after render)
	let annotateTimer: ReturnType<typeof setTimeout> | undefined
	let lastSize = 0
	watch(
		() => registry.size,
		(size) => {
			if (size <= lastSize || keyReveal.value) return
			lastSize = size
			clearTimeout(annotateTimer)
			annotateTimer = setTimeout(() => annotateFullDocument(registry), 200)
		},
		{ immediate: true },
	)
}

function processMutations(
	mutations: MutationRecord[],
	registry: Map<string, { key: string; value: string; defaultMessage?: string; timestamp: number }>,
) {
	const reverseLookup = new Map<string, string>()
	for (const [, entry] of registry) {
		if (entry.value) {
			reverseLookup.set(entry.value, entry.key)
		}
	}

	if (reverseLookup.size === 0) return

	for (const mutation of mutations) {
		if (mutation.type === 'childList') {
			for (const node of mutation.addedNodes) {
				if (node.nodeType === Node.ELEMENT_NODE) {
					annotateTextNodes(node as Element, reverseLookup)
				} else if (node.nodeType === Node.TEXT_NODE) {
					annotateTextNode(node as Text, reverseLookup)
				}
			}
			for (const node of mutation.removedNodes) {
				if (node.nodeType === Node.ELEMENT_NODE) {
					clearStaleAttributes(node as Element)
				}
			}
		} else if (mutation.type === 'characterData') {
			if (mutation.target.nodeType === Node.TEXT_NODE) {
				annotateTextNode(mutation.target as Text, reverseLookup)
			}
		}
	}
}

function annotateTextNodes(element: Element, reverseLookup: Map<string, string>) {
	if (element.closest('.i18n-debug-panel')) return

	const walker = document.createTreeWalker(element, NodeFilter.SHOW_TEXT)
	let node: Text | null
	while ((node = walker.nextNode() as Text | null)) {
		annotateTextNode(node, reverseLookup)
	}
}

function annotateTextNode(node: Text, reverseLookup: Map<string, string>) {
	const parent = node.parentElement
	if (!parent || parent.closest('.i18n-debug-panel')) return

	const text = node.textContent?.trim()
	if (!text) return

	const key = reverseLookup.get(text)
	if (key) {
		parent.setAttribute('data-i18n-key', key)
	}
}

function clearStaleAttributes(element: Element) {
	if (element.hasAttribute?.('data-i18n-key')) {
		element.removeAttribute('data-i18n-key')
	}
	const children = element.querySelectorAll?.('[data-i18n-key]')
	if (children) {
		for (const child of children) {
			child.removeAttribute('data-i18n-key')
		}
	}
}

function clearAllAnnotations() {
	for (const el of document.querySelectorAll('[data-i18n-key]')) {
		el.removeAttribute('data-i18n-key')
	}
}

function hideKeyTooltip() {
	const tooltip = document.querySelector('.i18n-key-tooltip') as HTMLElement | null
	if (tooltip) tooltip.style.display = 'none'
}

function annotateFullDocument(
	registry: Map<string, { key: string; value: string; defaultMessage?: string; timestamp: number }>,
) {
	const reverseLookup = new Map<string, string>()
	for (const [, entry] of registry) {
		if (entry.value) {
			reverseLookup.set(entry.value, entry.key)
		}
	}
	if (reverseLookup.size === 0) return
	annotateTextNodes(document.body, reverseLookup)
}

function setupKeyTooltip() {
	const tooltip = document.createElement('div')
	tooltip.className = 'i18n-key-tooltip'
	tooltip.style.display = 'none'
	document.body.appendChild(tooltip)

	let activeTarget: Element | null = null

	function positionTooltip() {
		if (!activeTarget) return
		const rect = activeTarget.getBoundingClientRect()
		const tooltipRect = tooltip.getBoundingClientRect()
		let top = rect.top - tooltipRect.height - 6
		if (top < 4) top = rect.bottom + 6
		let left = rect.left
		if (left + tooltipRect.width > window.innerWidth - 4) {
			left = window.innerWidth - tooltipRect.width - 4
		}
		tooltip.style.top = `${top}px`
		tooltip.style.left = `${left}px`
	}

	document.body.addEventListener('mouseover', (e) => {
		const target = (e.target as Element).closest?.('[data-i18n-key]')
		if (!target) return
		const key = target.getAttribute('data-i18n-key')
		if (!key) return
		activeTarget = target
		tooltip.textContent = key
		tooltip.style.display = ''
		positionTooltip()
	})

	document.body.addEventListener('mouseout', (e) => {
		const target = (e.target as Element).closest?.('[data-i18n-key]')
		if (!target) return
		const related = (e as MouseEvent).relatedTarget as Element | null
		if (related?.closest?.('[data-i18n-key]') === target) return
		activeTarget = null
		tooltip.style.display = 'none'
	})

	document.addEventListener('scroll', () => positionTooltip(), { capture: true, passive: true })
}

function registerKeyboardShortcuts(panelOpen: Ref<boolean>, keyReveal: Ref<boolean>) {
	document.addEventListener('keydown', (e: KeyboardEvent) => {
		// Use Cmd on macOS, Ctrl on other platforms
		const mod = e.metaKey || e.ctrlKey
		if (!mod || !e.shiftKey) return

		if (e.code === 'Period') {
			e.preventDefault()
			panelOpen.value = !panelOpen.value
		} else if (e.code === 'KeyK') {
			e.preventDefault()
			keyReveal.value = !keyReveal.value
		}
	})
}
