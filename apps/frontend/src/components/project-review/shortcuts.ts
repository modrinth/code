import type { ReviewShortcutAction } from '@modrinth/moderation'
import { createContext } from '@modrinth/ui'
import { onScopeDispose, type Ref, shallowRef, watch } from 'vue'

type Shortcut = { run: () => void; available: () => boolean }
type Interaction = {
	element: () => HTMLElement | null | undefined
	edit?: () => void
	collapse?: () => void
	editable?: () => boolean
}

export const [injectReviewShortcuts, provideReviewShortcuts] =
	createContext<ReturnType<typeof createReviewShortcuts>>('ReviewShortcuts')

export function createReviewShortcuts() {
	const windows = shallowRef<Window[]>(typeof window === 'undefined' ? [] : [window])
	const keyboardFocusedElement = shallowRef<HTMLElement>()
	let keyboardNavigation = false
	function clearKeyboardFocus() {
		keyboardNavigation = false
		keyboardFocusedElement.value = undefined
	}
	listenReviewWindows(windows, 'pointerdown', clearKeyboardFocus, { capture: true })
	listenReviewWindows(windows, 'blur', clearKeyboardFocus)
	listenReviewWindows(
		windows,
		'focusin',
		(event) => {
			keyboardFocusedElement.value =
				keyboardNavigation && event.target instanceof HTMLElement ? event.target : undefined
		},
		{ capture: true },
	)
	listenReviewWindows(
		windows,
		'focusout',
		() => {
			keyboardFocusedElement.value = undefined
		},
		{ capture: true },
	)
	listenReviewWindows(
		windows,
		'keydown',
		(event) => {
			if (event.defaultPrevented || event.isComposing) return
			if (event.key === 'Tab') {
				keyboardNavigation = true
				return
			}
			const focused = (event.target as Node | null)?.ownerDocument?.activeElement
			if (!(focused instanceof HTMLElement)) return
			if (event.key === 'Escape') {
				clearKeyboardFocus()
				focused.blur()
				return
			}
			if (
				!keyboardNavigation &&
				!['Enter', ' '].includes(event.key) &&
				!focused.isContentEditable &&
				!focused.closest('input, textarea, select, [role="textbox"], [role="dialog"], .cm-editor')
			)
				focused.blur()
		},
		{ capture: true },
	)
	const actions = new Map<ReviewShortcutAction, Shortcut>()
	const interactions = new Set<Interaction>()
	function interaction(action: 'edit' | 'collapse') {
		const available = [...interactions].filter((entry) => {
			const element = entry.element()
			return (
				element?.isConnected &&
				element.getClientRects().length &&
				!element.closest('[inert], [hidden]') &&
				entry[action] &&
				(action !== 'edit' || entry.editable?.() !== false)
			)
		})
		const hovered = available.filter((entry) => entry.element()?.matches(':hover'))
		const focused = available.filter((entry) =>
			entry.element()?.contains(keyboardFocusedElement.value ?? null),
		)
		const candidates = hovered.length ? hovered : focused
		return candidates.find(
			(entry) =>
				!candidates.some(
					(other) =>
						other !== entry &&
						other.element() !== entry.element() &&
						entry.element()?.contains(other.element() ?? null),
				),
		)
	}
	return {
		listen<K extends keyof WindowEventMap>(
			event: K,
			listener: (event: WindowEventMap[K]) => void,
			options?: AddEventListenerOptions,
		) {
			return listenReviewWindows(windows, event, listener, options)
		},
		windows,
		registerWindow(target: Window) {
			windows.value = [...windows.value, target]
			return () => {
				windows.value = windows.value.filter((entry) => entry !== target)
			}
		},
		keyboardFocusedElement,
		register(action: ReviewShortcutAction, run: () => void, available: () => boolean = () => true) {
			const shortcut = { run, available }
			actions.set(action, shortcut)
			return () => {
				if (actions.get(action) === shortcut) actions.delete(action)
			}
		},
		registerInteraction(entry: Interaction) {
			interactions.add(entry)
			return () => interactions.delete(entry)
		},
		available(action: ReviewShortcutAction) {
			return action === 'edit' || action === 'collapse'
				? !!interaction(action)
				: (actions.get(action)?.available() ?? false)
		},
		run(action: ReviewShortcutAction) {
			if (action === 'edit' || action === 'collapse') interaction(action)?.[action]?.()
			else {
				const shortcut = actions.get(action)
				if (shortcut?.available()) shortcut.run()
			}
		},
	}
}

export function useReviewShortcut(
	action: ReviewShortcutAction,
	run: () => void,
	available?: () => boolean,
) {
	onScopeDispose(injectReviewShortcuts().register(action, run, available))
}

export function useReviewInteraction(entry: Interaction) {
	onScopeDispose(injectReviewShortcuts().registerInteraction(entry))
}

export function reviewShortcutBlocked(event: KeyboardEvent) {
	if (event.defaultPrevented || event.repeat || event.isComposing) return true
	if (
		[
			...((event.target as Node | null)?.ownerDocument?.querySelectorAll(
				'[role="dialog"][aria-modal="true"]',
			) ?? []),
		].some((modal) => modal.getClientRects().length > 0)
	)
		return true
	const target = event.target
	return (
		target instanceof HTMLElement &&
		(target.isContentEditable ||
			!!target.closest('input, textarea, select, [role="textbox"], [role="dialog"], .cm-editor'))
	)
}

export function useReviewEventListener<K extends keyof WindowEventMap>(
	event: K,
	listener: (event: WindowEventMap[K]) => void,
	options?: AddEventListenerOptions,
) {
	return injectReviewShortcuts().listen(event, listener, options)
}

function listenReviewWindows<K extends keyof WindowEventMap>(
	windows: Ref<Window[]>,
	event: K,
	listener: (event: WindowEventMap[K]) => void,
	options?: AddEventListenerOptions,
) {
	const stop = watch(
		windows,
		(targets, _, onCleanup) => {
			for (const target of targets) target.addEventListener(event, listener, options)
			onCleanup(() => {
				for (const target of targets) target.removeEventListener(event, listener, options)
			})
		},
		{ immediate: true, flush: 'sync' },
	)
	onScopeDispose(stop)
	return stop
}
