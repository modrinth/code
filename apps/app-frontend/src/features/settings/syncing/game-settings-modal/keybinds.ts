import { defineMessages, type MessageDescriptor, type VIntlFormatters } from '@modrinth/ui'

type FormatMessage = VIntlFormatters['formatMessage']

type StopKeybindRecording = () => void
let stopActiveKeybindRecording: StopKeybindRecording | null = null

export function activateKeybindRecording(stop: StopKeybindRecording) {
	if (stopActiveKeybindRecording === stop) return
	const stopPrevious = stopActiveKeybindRecording
	stopActiveKeybindRecording = stop
	stopPrevious?.()
}

export function deactivateKeybindRecording(stop: StopKeybindRecording) {
	if (stopActiveKeybindRecording === stop) {
		stopActiveKeybindRecording = null
	}
}

const keyMessages = defineMessages({
	unbound: {
		id: 'app.settings.game-options.keybind.key.unbound',
		defaultMessage: 'Unbound',
	},
	escape: {
		id: 'app.settings.game-options.keybind.key.escape',
		defaultMessage: 'Escape',
	},
	backspace: {
		id: 'app.settings.game-options.keybind.key.backspace',
		defaultMessage: 'Backspace',
	},
	tab: {
		id: 'app.settings.game-options.keybind.key.tab',
		defaultMessage: 'Tab',
	},
	enter: {
		id: 'app.settings.game-options.keybind.key.enter',
		defaultMessage: 'Enter',
	},
	leftControl: {
		id: 'app.settings.game-options.keybind.key.left-control',
		defaultMessage: 'Left Ctrl',
	},
	rightControl: {
		id: 'app.settings.game-options.keybind.key.right-control',
		defaultMessage: 'Right Ctrl',
	},
	leftShift: {
		id: 'app.settings.game-options.keybind.key.left-shift',
		defaultMessage: 'Left Shift',
	},
	rightShift: {
		id: 'app.settings.game-options.keybind.key.right-shift',
		defaultMessage: 'Right Shift',
	},
	leftAlt: {
		id: 'app.settings.game-options.keybind.key.left-alt',
		defaultMessage: 'Left Alt',
	},
	rightAlt: {
		id: 'app.settings.game-options.keybind.key.right-alt',
		defaultMessage: 'Right Alt',
	},
	space: {
		id: 'app.settings.game-options.keybind.key.space',
		defaultMessage: 'Space',
	},
	capsLock: {
		id: 'app.settings.game-options.keybind.key.caps-lock',
		defaultMessage: 'Caps Lock',
	},
	numLock: {
		id: 'app.settings.game-options.keybind.key.num-lock',
		defaultMessage: 'Num Lock',
	},
	scrollLock: {
		id: 'app.settings.game-options.keybind.key.scroll-lock',
		defaultMessage: 'Scroll Lock',
	},
	printScreen: {
		id: 'app.settings.game-options.keybind.key.print-screen',
		defaultMessage: 'Print Screen',
	},
	pause: {
		id: 'app.settings.game-options.keybind.key.pause',
		defaultMessage: 'Pause',
	},
	home: {
		id: 'app.settings.game-options.keybind.key.home',
		defaultMessage: 'Home',
	},
	end: {
		id: 'app.settings.game-options.keybind.key.end',
		defaultMessage: 'End',
	},
	pageUp: {
		id: 'app.settings.game-options.keybind.key.page-up',
		defaultMessage: 'Page Up',
	},
	pageDown: {
		id: 'app.settings.game-options.keybind.key.page-down',
		defaultMessage: 'Page Down',
	},
	insert: {
		id: 'app.settings.game-options.keybind.key.insert',
		defaultMessage: 'Insert',
	},
	delete: {
		id: 'app.settings.game-options.keybind.key.delete',
		defaultMessage: 'Delete',
	},
	arrowUp: {
		id: 'app.settings.game-options.keybind.key.arrow-up',
		defaultMessage: 'Up Arrow',
	},
	arrowDown: {
		id: 'app.settings.game-options.keybind.key.arrow-down',
		defaultMessage: 'Down Arrow',
	},
	arrowLeft: {
		id: 'app.settings.game-options.keybind.key.arrow-left',
		defaultMessage: 'Left Arrow',
	},
	arrowRight: {
		id: 'app.settings.game-options.keybind.key.arrow-right',
		defaultMessage: 'Right Arrow',
	},
	leftSuper: {
		id: 'app.settings.game-options.keybind.key.left-super',
		defaultMessage: 'Left Super',
	},
	rightSuper: {
		id: 'app.settings.game-options.keybind.key.right-super',
		defaultMessage: 'Right Super',
	},
	leftCommand: {
		id: 'app.settings.game-options.keybind.key.left-command',
		defaultMessage: 'Left Command',
	},
	rightCommand: {
		id: 'app.settings.game-options.keybind.key.right-command',
		defaultMessage: 'Right Command',
	},
	menu: {
		id: 'app.settings.game-options.keybind.key.menu',
		defaultMessage: 'Menu',
	},
	keypadKey: {
		id: 'app.settings.game-options.keybind.key.keypad-key',
		defaultMessage: 'Numpad {key}',
	},
	leftMouse: {
		id: 'app.settings.game-options.keybind.mouse.left',
		defaultMessage: 'Left Mouse',
	},
	rightMouse: {
		id: 'app.settings.game-options.keybind.mouse.right',
		defaultMessage: 'Right Mouse',
	},
	middleMouse: {
		id: 'app.settings.game-options.keybind.mouse.middle',
		defaultMessage: 'Middle Mouse',
	},
	mouseButton: {
		id: 'app.settings.game-options.keybind.mouse.button',
		defaultMessage: 'Mouse {button}',
	},
	scancode: {
		id: 'app.settings.game-options.keybind.scancode',
		defaultMessage: 'Scancode {code}',
	},
	unknownKey: {
		id: 'app.settings.game-options.keybind.key.unknown',
		defaultMessage: 'Unknown key',
	},
	controlModifier: {
		id: 'app.settings.game-options.keybind.modifier.control',
		defaultMessage: 'Ctrl',
	},
	shiftModifier: {
		id: 'app.settings.game-options.keybind.modifier.shift',
		defaultMessage: 'Shift',
	},
	altModifier: {
		id: 'app.settings.game-options.keybind.modifier.alt',
		defaultMessage: 'Alt',
	},
	commandModifier: {
		id: 'app.settings.game-options.keybind.modifier.command',
		defaultMessage: 'Command',
	},
	superModifier: {
		id: 'app.settings.game-options.keybind.modifier.super',
		defaultMessage: 'Super',
	},
})

const keyboardCodeTokens: Record<string, string> = {
	Escape: 'escape',
	Minus: 'minus',
	Equal: 'equal',
	Backspace: 'backspace',
	Tab: 'tab',
	BracketLeft: 'left.bracket',
	BracketRight: 'right.bracket',
	Enter: 'enter',
	ControlLeft: 'left.control',
	Semicolon: 'semicolon',
	Quote: 'apostrophe',
	Backquote: 'grave.accent',
	ShiftLeft: 'left.shift',
	Backslash: 'backslash',
	Comma: 'comma',
	Period: 'period',
	Slash: 'slash',
	ShiftRight: 'right.shift',
	AltLeft: 'left.alt',
	Space: 'space',
	CapsLock: 'caps.lock',
	NumLock: 'num.lock',
	ScrollLock: 'scroll.lock',
	NumpadSubtract: 'keypad.subtract',
	NumpadAdd: 'keypad.add',
	NumpadDecimal: 'keypad.decimal',
	NumpadComma: 'keypad.decimal',
	NumpadEqual: 'keypad.equal',
	NumpadEnter: 'keypad.enter',
	ControlRight: 'right.control',
	NumpadDivide: 'keypad.divide',
	NumpadMultiply: 'keypad.multiply',
	PrintScreen: 'print.screen',
	AltRight: 'right.alt',
	Pause: 'pause',
	Home: 'home',
	ArrowUp: 'up',
	PageUp: 'page.up',
	ArrowLeft: 'left',
	ArrowRight: 'right',
	End: 'end',
	ArrowDown: 'down',
	PageDown: 'page.down',
	Insert: 'insert',
	Delete: 'delete',
	MetaLeft: 'left.win',
	MetaRight: 'right.win',
	ContextMenu: 'menu',
}

const keyboardLabels: Record<string, MessageDescriptor | string> = {
	escape: keyMessages.escape,
	minus: '-',
	equal: '=',
	backspace: keyMessages.backspace,
	tab: keyMessages.tab,
	'left.bracket': '[',
	'right.bracket': ']',
	enter: keyMessages.enter,
	'left.control': keyMessages.leftControl,
	semicolon: ';',
	apostrophe: "'",
	'grave.accent': '`',
	'left.shift': keyMessages.leftShift,
	backslash: '\\',
	comma: ',',
	period: '.',
	slash: '/',
	'right.shift': keyMessages.rightShift,
	'left.alt': keyMessages.leftAlt,
	space: keyMessages.space,
	'caps.lock': keyMessages.capsLock,
	'num.lock': keyMessages.numLock,
	'scroll.lock': keyMessages.scrollLock,
	'keypad.subtract': '-',
	'keypad.add': '+',
	'keypad.decimal': '.',
	'keypad.equal': '=',
	'keypad.enter': keyMessages.enter,
	'right.control': keyMessages.rightControl,
	'keypad.divide': '/',
	'keypad.multiply': '*',
	'print.screen': keyMessages.printScreen,
	'right.alt': keyMessages.rightAlt,
	pause: keyMessages.pause,
	home: keyMessages.home,
	up: keyMessages.arrowUp,
	'page.up': keyMessages.pageUp,
	left: keyMessages.arrowLeft,
	right: keyMessages.arrowRight,
	end: keyMessages.end,
	down: keyMessages.arrowDown,
	'page.down': keyMessages.pageDown,
	insert: keyMessages.insert,
	delete: keyMessages.delete,
	menu: keyMessages.menu,
}

function splitKeybind(value: string): [key: string, modifier?: string] {
	const separator = value.indexOf(':')
	return separator === -1 ? [value] : [value.slice(0, separator), value.slice(separator + 1)]
}

function keyboardIdentifierFromCode(code: string): string | null {
	const letter = /^Key([A-Z])$/.exec(code)
	if (letter) return letter[1].toLowerCase()

	const digit = /^Digit([0-9])$/.exec(code)
	if (digit) return digit[1]

	const functionKey = /^F([1-9]|1[0-9]|2[0-5])$/.exec(code)
	if (functionKey) return `f${functionKey[1]}`

	const keypadDigit = /^Numpad([0-9])$/.exec(code)
	if (keypadDigit) return `keypad.${keypadDigit[1]}`

	return keyboardCodeTokens[code] ?? null
}

export function minecraftKeyTokenFromKeyboardEvent(event: KeyboardEvent): string | null {
	const identifier = keyboardIdentifierFromCode(event.code)
	return identifier ? `key.keyboard.${identifier}` : null
}

export function minecraftMouseTokenFromButton(button: number): string | null {
	if (!Number.isInteger(button) || button < 0 || button > 15) return null
	if (button === 0) return 'key.mouse.left'
	if (button === 1) return 'key.mouse.middle'
	if (button === 2) return 'key.mouse.right'
	return `key.mouse.${button + 1}`
}

function formatKeyboardLabel(
	formatMessage: FormatMessage,
	identifier: string,
	isMac: boolean,
): string {
	if (identifier === 'unknown') return formatMessage(keyMessages.unbound)
	if (/^[a-z]$/.test(identifier)) return identifier.toUpperCase()
	if (/^[0-9]$/.test(identifier)) return identifier
	if (/^f([1-9]|1[0-9]|2[0-5])$/.test(identifier)) return identifier.toUpperCase()

	const keypad = /^keypad\.(.+)$/.exec(identifier)
	if (keypad) {
		const label = keyboardLabels[identifier]
		const key = typeof label === 'string' ? label : label ? formatMessage(label) : keypad[1]
		return formatMessage(keyMessages.keypadKey, { key })
	}

	if (identifier === 'left.win') {
		return formatMessage(isMac ? keyMessages.leftCommand : keyMessages.leftSuper)
	}
	if (identifier === 'right.win') {
		return formatMessage(isMac ? keyMessages.rightCommand : keyMessages.rightSuper)
	}

	const label = keyboardLabels[identifier]
	if (typeof label === 'string') return label
	if (label) return formatMessage(label)
	return identifier
		.split('.')
		.map((part) => `${part.charAt(0).toUpperCase()}${part.slice(1)}`)
		.join(' ')
}

function formatModifier(formatMessage: FormatMessage, modifier: string, isMac: boolean): string {
	switch (modifier.toUpperCase()) {
		case 'CONTROL':
		case 'CTRL':
			return formatMessage(keyMessages.controlModifier)
		case 'SHIFT':
			return formatMessage(keyMessages.shiftModifier)
		case 'ALT':
			return formatMessage(keyMessages.altModifier)
		case 'META':
		case 'SUPER':
			return formatMessage(isMac ? keyMessages.commandModifier : keyMessages.superModifier)
		default:
			return modifier
	}
}

export function formatMinecraftKeybind(
	formatMessage: FormatMessage,
	value: string,
	isMac: boolean,
): string {
	const [key, modifier] = splitKeybind(value)
	let label: string

	if (key.startsWith('key.keyboard.')) {
		label = formatKeyboardLabel(formatMessage, key.slice('key.keyboard.'.length), isMac)
	} else if (key === 'key.mouse.left') {
		label = formatMessage(keyMessages.leftMouse)
	} else if (key === 'key.mouse.right') {
		label = formatMessage(keyMessages.rightMouse)
	} else if (key === 'key.mouse.middle') {
		label = formatMessage(keyMessages.middleMouse)
	} else if (key.startsWith('key.mouse.')) {
		label = formatMessage(keyMessages.mouseButton, {
			button: key.slice('key.mouse.'.length),
		})
	} else if (key.startsWith('scancode.')) {
		label = formatMessage(keyMessages.scancode, { code: key.slice('scancode.'.length) })
	} else {
		label = formatMessage(keyMessages.unknownKey)
	}

	if (!modifier || key === 'key.keyboard.unknown') return label
	const modifiers = modifier
		.split(/[+:]/)
		.filter(Boolean)
		.map((part) => formatModifier(formatMessage, part, isMac))
	return [...modifiers, label].join(' + ')
}

export function minecraftKeybindConflictKey(optionId: string, value: string): string | null {
	const normalized = value.trim().toLowerCase()
	if (!normalized || normalized === 'key.keyboard.unknown') return null

	if (optionId === 'key.debug.modifier') return null
	if (optionId.startsWith('key.debug.') && optionId !== 'key.debug.overlay') {
		return `debug:${normalized}`
	}
	return `direct:${normalized}`
}
