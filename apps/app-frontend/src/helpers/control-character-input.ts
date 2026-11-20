function isControlCharacter(character: string) {
	const code = character.charCodeAt(0)
	return code <= 0x1f || code === 0x7f
}

/**
 * Stops text inputs from receiving text that is only control characters.
 *
 * On macOS, the webview can insert the legacy arrow-key codes (U+001C to U+001F) into an input
 * when an arrow key cannot move the caret any further.
 * See https://github.com/tauri-apps/tauri/issues/10194.
 */
export function blockControlCharacterInput() {
	document.addEventListener(
		'beforeinput',
		(event) => {
			if (
				event.inputType === 'insertText' &&
				event.data &&
				[...event.data].every(isControlCharacter)
			) {
				event.preventDefault()
			}
		},
		{ capture: true },
	)
}
