/**
 * Checks if any modifier key (Ctrl, Alt, Meta, or Shift) is held down during an event.
 */
export function isModifierKeyDown(
	e: Pick<KeyboardEvent, 'ctrlKey' | 'altKey' | 'metaKey' | 'shiftKey'>,
): boolean {
	return e.ctrlKey || e.altKey || e.metaKey || e.shiftKey
}
