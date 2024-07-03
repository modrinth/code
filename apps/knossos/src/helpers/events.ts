/**
 * Checks if any of the modifier keys is down for the event.
 * @param e Event that is triggered with the state of modified keys.
 * @returns Whether any of the modifier keys is pressed.
 */
export function isModifierKeyDown(
  e: Pick<KeyboardEvent, 'ctrlKey' | 'altKey' | 'metaKey' | 'shiftKey'>
) {
  return e.ctrlKey || e.altKey || e.metaKey || e.shiftKey
}
