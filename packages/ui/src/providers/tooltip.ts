import type { Placement } from '@floating-ui/vue'
import type { App, Directive } from 'vue'
import { ref } from 'vue'

export type TooltipPlacement = Placement
export type TooltipProps = string | { text: string } | null | undefined
export type TooltipDirective = Directive<HTMLElement, TooltipProps>
export type TooltipContent = () => unknown

interface TooltipSource {
	placement: TooltipPlacement
	getText: () => string | null
	render?: TooltipContent
	theme?: string
	pinned?: boolean
	panelClass?: string
}

interface TooltipState {
	reference: HTMLElement | null
	text: string | null
	placement: TooltipPlacement
	theme: string
	content: TooltipContent | null
	panelClass?: string
}

declare module 'vue' {
	export interface GlobalDirectives {
		vTooltip: TooltipDirective
	}
}

const SHOW_DELAY = 300
const HIDE_DELAY = 400

export const activeTooltip = ref<TooltipState>({
	reference: null,
	text: null,
	placement: 'top',
	theme: 'tooltip',
	content: null,
	panelClass: undefined,
})

const sources = new WeakMap<HTMLElement, TooltipSource>()
let showTimer: ReturnType<typeof setTimeout> | undefined
let hideTimer: ReturnType<typeof setTimeout> | undefined
let pending: HTMLElement | undefined
let hovered: HTMLElement | undefined
let focused: HTMLElement | undefined
let lastOpen = 0

function stopShow() {
	clearTimeout(showTimer)
	showTimer = undefined
	pending = undefined
}

function stopHide() {
	clearTimeout(hideTimer)
	hideTimer = undefined
}

function clear() {
	activeTooltip.value = {
		reference: null,
		text: null,
		placement: 'top',
		theme: 'tooltip',
		content: null,
		panelClass: undefined,
	}
}

function hasTooltipContent(el: HTMLElement) {
	const source = sources.get(el)
	return !!(source?.getText() || source?.render)
}

function open(el: HTMLElement) {
	const source = sources.get(el)
	pending = undefined
	const text = source?.getText() ?? null
	const content = source?.render ?? null
	if (!text && !content) {
		if (activeTooltip.value.reference === el) {
			clear()
		}
		return
	}
	activeTooltip.value = {
		reference: el,
		text,
		placement: source?.placement ?? 'top',
		theme: source?.theme ?? 'tooltip',
		content,
		panelClass: source?.panelClass,
	}
	lastOpen = Date.now()
}

function show(immediate: boolean) {
	const el = focused ?? hovered
	if (!el || !hasTooltipContent(el)) {
		return
	}
	stopShow()
	stopHide()
	if (immediate || activeTooltip.value.reference || Date.now() - lastOpen <= SHOW_DELAY) {
		open(el)
		return
	}
	pending = el
	showTimer = setTimeout(() => open(el), SHOW_DELAY)
}

function hide(el: HTMLElement, immediate: boolean) {
	if (sources.get(el)?.pinned) {
		return
	}
	if (focused ?? hovered) {
		show(true)
		return
	}
	if (pending === el) {
		stopShow()
	}
	if (activeTooltip.value.reference !== el) {
		return
	}
	if (immediate) {
		stopHide()
		clear()
		return
	}
	stopHide()
	hideTimer = setTimeout(() => {
		if ((focused ?? hovered) || sources.get(el)?.pinned) {
			return
		}
		if (activeTooltip.value.reference !== el) {
			return
		}
		clear()
		stopHide()
	}, HIDE_DELAY)
}

export function bindTooltipSource(el: HTMLElement, source: TooltipSource) {
	const wasPinned = sources.get(el)?.pinned
	sources.set(el, source)
	if (source.pinned || activeTooltip.value.reference === el) {
		open(el)
	}
	if (wasPinned && !source.pinned) {
		hide(el, true)
	}
}

export function unbindTooltipSource(el: HTMLElement) {
	sources.delete(el)
	if (pending === el) {
		stopShow()
	}
	if (hovered === el) {
		hovered = undefined
	}
	if (focused === el) {
		focused = undefined
	}
	if (activeTooltip.value.reference === el) {
		stopHide()
		clear()
	}
}

function isFocusVisible(el: HTMLElement) {
	const active = document.activeElement
	if (!(active instanceof Element) || !el.contains(active)) {
		return false
	}
	return active.matches(':focus-visible')
}

export function tooltipEnter(el: HTMLElement) {
	if (!hasTooltipContent(el)) {
		return
	}
	hovered = el
	show(false)
}

export function tooltipLeave(el: HTMLElement) {
	if (hovered === el) {
		hovered = undefined
	}
	if (focused === el && !isFocusVisible(el)) {
		focused = undefined
	}
	hide(el, false)
}

export function tooltipFocusIn(el: HTMLElement) {
	if (!isFocusVisible(el) || !hasTooltipContent(el)) {
		return
	}
	focused = el
	show(true)
}

export function tooltipFocusOut(el: HTMLElement) {
	if (focused === el) {
		focused = undefined
	}
	hide(el, true)
}

export function dismissTooltip() {
	stopShow()
	stopHide()
	hovered = undefined
	focused = undefined
	lastOpen = 0
	clear()
}

export function installTooltipDirective(app: App) {
	const listeners = new WeakMap<HTMLElement, AbortController>()
	const addedTabIndex = new WeakSet<HTMLElement>()
	const addedAriaLabel = new WeakSet<HTMLElement>()

	app.directive('tooltip', {
		mounted(el, binding) {
			sync(el, binding.value, binding.modifiers)
			const ac = new AbortController()
			listeners.set(el, ac)
			el.addEventListener('mouseenter', () => tooltipEnter(el), { signal: ac.signal })
			el.addEventListener('mouseleave', () => tooltipLeave(el), { signal: ac.signal })
			el.addEventListener('focusin', () => tooltipFocusIn(el), { signal: ac.signal })
			el.addEventListener(
				'focusout',
				(event) => {
					if (event.relatedTarget instanceof Node && el.contains(event.relatedTarget)) {
						return
					}
					tooltipFocusOut(el)
				},
				{ signal: ac.signal },
			)
		},
		updated(el, binding) {
			sync(el, binding.value, binding.modifiers)
		},
		beforeUnmount(el) {
			unbindTooltipSource(el)
			listeners.get(el)?.abort()
			releaseFocusable(el, addedTabIndex)
			releaseAriaLabel(el, addedAriaLabel)
		},
	} satisfies TooltipDirective)

	function sync(el: HTMLElement, value: TooltipProps, modifiers: Record<string, boolean>) {
		const text = tooltipText(value)
		if (!text) {
			releaseFocusable(el, addedTabIndex)
			releaseAriaLabel(el, addedAriaLabel)
			unbindTooltipSource(el)
			return
		}
		ensureFocusable(el, addedTabIndex)
		ensureAriaLabel(el, text, addedAriaLabel)
		bindTooltipSource(el, {
			placement: tooltipPlacement(modifiers),
			getText: () => tooltipText(value),
		})
	}
}

function tooltipText(value: TooltipProps): string | null {
	if (value == null) {
		return null
	}
	return typeof value === 'string' ? value : (value.text ?? null)
}

function ensureAriaLabel(el: HTMLElement, text: string, added: WeakSet<HTMLElement>) {
	if (added.has(el)) {
		el.setAttribute('aria-label', text)
		return
	}
	if (
		el.hasAttribute('aria-label') ||
		el.hasAttribute('aria-labelledby') ||
		el.querySelector('[aria-label], [aria-labelledby]')
	) {
		return
	}
	el.setAttribute('aria-label', text)
	added.add(el)
}

function releaseAriaLabel(el: HTMLElement, added: WeakSet<HTMLElement>) {
	if (!added.has(el)) {
		return
	}
	el.removeAttribute('aria-label')
	added.delete(el)
}

function ensureFocusable(el: HTMLElement, added: WeakSet<HTMLElement>) {
	if (
		el.hasAttribute('tabindex') ||
		el.tabIndex >= 0 ||
		el.querySelector(
			'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
		)
	) {
		return
	}
	el.tabIndex = 0
	added.add(el)
}

function releaseFocusable(el: HTMLElement, added: WeakSet<HTMLElement>) {
	if (!added.has(el)) {
		return
	}
	el.removeAttribute('tabindex')
	added.delete(el)
}

function tooltipPlacement(modifiers: Partial<Record<string, boolean>>): TooltipPlacement {
	if (modifiers.right) {
		return 'right'
	}
	if (modifiers.bottom) {
		return 'bottom'
	}
	if (modifiers.left) {
		return 'left'
	}
	return 'top'
}
