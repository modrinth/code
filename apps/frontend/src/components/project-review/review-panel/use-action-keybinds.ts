import { useEventListener } from '@vueuse/core'
import type { Ref } from 'vue'

export function useActionKeybinds(
	element: Ref<HTMLElement | null>,
	isShowing: () => boolean = () => false,
) {
	useEventListener('keydown', (event) => {
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
		if (!panel || panel.querySelector('[data-review-panel]:hover')) return
		const dialog = target instanceof Element ? target.closest('[role="dialog"]') : null
		if (dialog && dialog !== panel && !dialog.contains(panel)) return
		if (
			[...document.querySelectorAll('[role="dialog"][aria-modal="true"]')].some(
				(modal) => !modal.contains(panel),
			)
		) {
			return
		}
		if (!panel.matches(':hover')) {
			if (!isShowing() || document.querySelector('[data-review-panel]:hover')) return
		}

		const button = panel.querySelector<HTMLButtonElement>(
			`button[data-review-keybind="${keybind}"]`,
		)
		if (!button || button.disabled || button.closest('[data-review-panel]') !== panel) return

		event.preventDefault()
		if (document.activeElement instanceof HTMLElement) document.activeElement.blur()
		button.click()
		button.blur()
	})
}
