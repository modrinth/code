import type { Ref } from 'vue'

import type { createReviewShortcuts } from '../shortcuts'

export function useActionKeybinds(
	element: Readonly<Ref<HTMLElement | null>>,
	shortcuts: ReturnType<typeof createReviewShortcuts>,
) {
	shortcuts.listen('keydown', (event) => {
		if (
			event.defaultPrevented ||
			event.repeat ||
			event.isComposing ||
			event.ctrlKey ||
			event.metaKey ||
			(event.shiftKey && event.altKey)
		) {
			return
		}
		const digit = /^[1-9]$/.test(event.key)
			? event.key
			: (event.shiftKey || event.altKey) && /^Digit[1-9]$/.test(event.code)
				? event.code.slice(-1)
				: undefined
		if (!digit) return
		const modifier = event.altKey ? 'Alt+' : event.shiftKey ? 'Shift+' : ''
		const keybind = `${modifier}${digit}`

		const target = event.target
		if (
			target instanceof HTMLElement &&
			(target.isContentEditable || target.closest('input, textarea, select, [role="textbox"]'))
		) {
			return
		}

		const panel = element.value
		if (!panel || panel.closest('[inert], [hidden]')) return
		if ((event.target as Node | null)?.ownerDocument !== panel.ownerDocument) return
		const dialog = target instanceof Element ? target.closest('[role="dialog"]') : null
		if (dialog && dialog !== panel && !dialog.contains(panel)) return
		if (
			[...panel.ownerDocument.querySelectorAll('[role="dialog"][aria-modal="true"]')].some(
				(modal) => !modal.contains(panel),
			)
		) {
			return
		}
		const button = panel.querySelector<HTMLButtonElement>(
			`button[data-review-keybind="${keybind}"]`,
		)
		if (!button || button.disabled || button.closest('[data-review-panel]') !== panel) return

		event.preventDefault()
		const focused = panel.ownerDocument.activeElement
		if (focused instanceof HTMLElement) focused.blur()
		button.click()
		button.blur()
	})
}
