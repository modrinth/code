import type { Placement } from '@floating-ui/vue'
import type { App, Directive } from 'vue'
import { ref } from 'vue'

export type TooltipPlacement = Placement
export type TooltipProps = string | { text: string } | null | undefined
export type TooltipDirective = Directive<HTMLElement, TooltipProps>
export type TooltipContent = () => unknown

export interface TooltipBaseProps {
	reference?: HTMLElement | null
	text?: (() => string | null) | string | null
	placement?: TooltipPlacement
	theme?: string
	content?: TooltipContent | null
	panelClass?: string
	delay?: DelayTimes | null
	hoverable?: boolean
	pinned?: boolean
	allowTransfer?: boolean
}

type RequiredFor<T, K extends keyof T> = Required<Pick<T, K>> & Pick<T, Exclude<keyof T, K>>

type TooltipState = RequiredFor<
	Omit<TooltipBaseProps, 'pinned'>,
	'placement' | 'theme' | 'reference'
>

type TimeoutId = ReturnType<typeof setTimeout>

type ToggledTooltipData = { id: TimeoutId | undefined; el: HTMLElement }

type DelayTimes = { hover?: number; unhover?: number } | number

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

const sources = new WeakMap<HTMLElement, TooltipBaseProps>()
let showTimer: ToggledTooltipData | undefined
let hideTimer: ToggledTooltipData | undefined
let hovered: HTMLElement | undefined
let focused: HTMLElement | undefined
let lastOpen = 0

function stopShow() {
	if (showTimer == null) return
	clearTimeout(showTimer.id)
	showTimer = undefined
}

function stopHide() {
	if (hideTimer == null) return
	clearTimeout(hideTimer.id)
	hideTimer = undefined
}

function hideWaiting(el?: HTMLElement) {
	if (hideTimer == null) return false

	const hidingEl = hideTimer.el
	stopHide()

	const isHidingElement = hidingEl == el
	if (!isHidingElement) hide(hidingEl, true)

	return isHidingElement
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
	return !!((typeof source?.text === 'function' ? source.text() : source?.text) || source?.content)
}

function open(el: HTMLElement) {
	const source = sources.get(el)
	const text = (typeof source?.text === 'function' ? source.text() : source?.text) ?? null
	const content = source?.content ?? null
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
		delay: source?.delay,
		hoverable: source?.hoverable,
	}
	lastOpen = Date.now()
}

function show(immediate: boolean, delay: number = SHOW_DELAY) {
	const el = focused ?? hovered
	if (!el || !hasTooltipContent(el)) {
		return
	}
	if (hideWaiting(el)) return
	if (showTimer) stopShow()
	const source = sources.get(el)
	if (
		immediate ||
		((activeTooltip.value.reference || Date.now() - lastOpen <= delay) && source?.allowTransfer)
	) {
		open(el)
		return
	}
	showTimer = {
		id: setTimeout(() => {
			open(el)
			stopShow()
		}, delay),
		el: el,
	}
}

function hide(el: HTMLElement, immediate: boolean, delay: number = HIDE_DELAY) {
	if (showTimer?.el === el) stopShow()

	const source = sources.get(el)

	if (source?.pinned) {
		return
	}

	if (source?.allowTransfer && (focused ?? hovered)) {
		show(true)
		return
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
	hideTimer = {
		id: setTimeout(() => {
			if ((focused ?? hovered) || sources.get(el)?.pinned) {
				return
			}
			if (activeTooltip.value.reference !== el) {
				return
			}
			clear()
			stopHide()
		}, delay),
		el: el,
	}
}

export function bindTooltipSource(el: HTMLElement, source: TooltipBaseProps) {
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
	if (showTimer?.el === el) {
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

export function tooltipEnter(el: HTMLElement, delay?: number): void {
	if (!hasTooltipContent(el)) {
		return
	}
	console.log('tooltipEnter', delay)
	hovered = el
	show(false, delay)
}

export function preventHide(el: HTMLElement) {
	if (hideTimer?.el == el) {
		stopHide()
	}
}

export function tooltipLeave(el: HTMLElement, delay?: number): void {
	if (hovered === el) {
		hovered = undefined
	}
	if (focused === el && !isFocusVisible(el)) {
		focused = undefined
	}
	hide(el, false, delay)
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

	function sync(el: HTMLElement, value: TooltipProps, modifiers: Partial<Record<string, boolean>>) {
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
			text: () => tooltipText(value),
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
