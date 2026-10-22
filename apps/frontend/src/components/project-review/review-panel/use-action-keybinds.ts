import type { Ref } from 'vue'

import { useModerationKeybinds } from '~/composables/moderation'

import type { createReviewShortcuts } from '../shortcuts'

export function useActionKeybinds(
	element: Readonly<Ref<HTMLElement | null>>,
	shortcuts: ReturnType<typeof createReviewShortcuts>,
) {
	const keybinds = useModerationKeybinds()
	shortcuts.listen('keydown', (event) => {
		if (event.defaultPrevented || event.repeat || event.isComposing) {
			return
		}

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
		const toggleButton = (index: number) => {
			const button = panel.querySelector<HTMLButtonElement>(
				`button[data-review-toggle-index="${index}"]`,
			)
			return button &&
				!button.disabled &&
				button.closest('[data-review-panel]') === panel &&
				button.getClientRects().length > 0
				? button
				: undefined
		}
		keybinds.value.handle(event, {
			scope: 'review-panel-toggle',
			available: (index) => !!toggleButton(index),
			toggle: (index) => {
				const button = toggleButton(index)
				if (!button) return
				const focused = panel.ownerDocument.activeElement
				if (focused instanceof HTMLElement) focused.blur()
				button.click()
				button.blur()
			},
		})
	})
}
