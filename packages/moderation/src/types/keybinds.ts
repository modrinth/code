import type { Labrinth } from '@modrinth/api-client'

import type { MiscReviewPanelKey } from '../data/issues'

export interface TechReviewActions {
	goToTop: () => void
	goToBottom: () => void
}

export type ModerationProjectContext = {
	project: Labrinth.Projects.v2.Project
	scope: 'project'
	notifyCopied: (value: string, title: string) => void
}

export type ModerationTechReviewContext = {
	scope: 'tech-review'
	actions: TechReviewActions
}

export type ModerationGlobalContext = {
	scope: 'global'
	officialUrl: string
	alternativeUrl: string
	notifyCopied: (value: string, title: string) => void
}

export type ModerationProjectReviewContext = {
	scope: 'project-review'
	openTab: (
		tab:
			| 'description'
			| 'gallery'
			| 'disclosures'
			| 'versions'
			| 'permissions'
			| 'tech-review'
			| 'slicer',
	) => void
}

export type ModerationContext =
	| ModerationPanelToggleContext
	| ModerationProjectContext
	| ModerationTechReviewContext
	| ModerationGlobalContext
	| ModerationProjectReviewContext
	| ModerationConversationContext
	| ModerationReviewActionsContext
	| ModerationReviewComposerContext

export type ModerationPanelToggleContext = {
	scope: 'review-panel-toggle'
	available: (index: number) => boolean
	toggle: (index: number) => void
}

export type ModerationConversationContext = {
	scope: 'review-conversation'
	openEditor: (mode: 'reply' | 'note') => void
}

export type ReviewShortcutAction =
	| 'cycle-conversation'
	| 'cycle-misc'
	| 'toggle-icon'
	| 'new-issue'
	| 'toggle-left'
	| 'toggle-right'
	| 'reveal-right'
	| 'back'
	| 'next'
	| 'exit'
	| 'reset'
	| 'approve'
	| 'withhold'
	| 'reject'
	| 'edit'
	| 'collapse'
	| 're-review'
	| MiscReviewPanelKey

export type ModerationReviewActionsContext = {
	scope: 'review-actions'
	available: (action: ReviewShortcutAction) => boolean
	run: (action: ReviewShortcutAction) => void
}

export type ModerationReviewComposerContext = {
	scope: 'review-composer'
	canSend: () => boolean
	send: (mode: 'reply' | 'note') => void
}

export interface KeybindDefinition {
	key: string
	mod?: boolean
	ctrl?: boolean
	shift?: boolean
	alt?: boolean
	meta?: boolean
	preventDefault?: boolean
}

export type BaseKeybindListener<T> = {
	keybind: KeybindDefinition | KeybindDefinition[] | string | string[]
	description: string
	scope: ModerationContext['scope']
	enabled?: (ctx: T) => boolean
	action: (ctx: T) => void
}

export type KeybindProjectListener = BaseKeybindListener<ModerationProjectContext> & {
	scope: 'project'
}
export type KeybindTechReviewListener = BaseKeybindListener<ModerationTechReviewContext> & {
	scope: 'tech-review'
}
export type KeybindGlobalListener = BaseKeybindListener<ModerationGlobalContext> & {
	scope: 'global'
}
export type KeybindProjectReviewListener = BaseKeybindListener<ModerationProjectReviewContext> & {
	scope: 'project-review'
}
export type KeybindListener =
	| (BaseKeybindListener<ModerationPanelToggleContext> & {
			scope: 'review-panel-toggle'
	  })
	| KeybindProjectListener
	| KeybindTechReviewListener
	| KeybindGlobalListener
	| KeybindProjectReviewListener
	| (BaseKeybindListener<ModerationConversationContext> & {
			scope: 'review-conversation'
	  })
	| (BaseKeybindListener<ModerationReviewActionsContext> & {
			scope: 'review-actions'
	  })
	| (BaseKeybindListener<ModerationReviewComposerContext> & {
			scope: 'review-composer'
	  })

export function parseKeybind(keybindString: string): KeybindDefinition {
	const parts = keybindString.split('+').map((p) => p.trim().toLowerCase())

	return {
		key: parts.find((p) => !['ctrl', 'shift', 'alt', 'meta', 'cmd'].includes(p)) || '',
		mod: parts.includes('ctrl'),
		ctrl: false,
		shift: parts.includes('shift'),
		alt: parts.includes('alt'),
		meta: parts.includes('meta') || parts.includes('cmd'),
		preventDefault: true,
	}
}

export function normalizeKeybind(keybind: KeybindDefinition | string): KeybindDefinition {
	return typeof keybind === 'string' ? parseKeybind(keybind) : keybind
}

type KeybindEvent = Pick<
	KeyboardEvent,
	'key' | 'code' | 'ctrlKey' | 'metaKey' | 'shiftKey' | 'altKey'
>

export function matchesKeybind(event: KeybindEvent, keybind: KeybindDefinition | string): boolean {
	const def = normalizeKeybind(keybind)
	const portableMod = def.mod || (def.ctrl && (def.meta || def.meta === undefined))
	const modifiersMatch = portableMod
		? event.ctrlKey || event.metaKey
		: event.ctrlKey === !!def.ctrl && event.metaKey === !!def.meta
	return (
		(event.key.toLowerCase() === def.key.toLowerCase() ||
			((event.shiftKey || event.altKey) &&
				/^[1-9]$/.test(def.key) &&
				event.code === `Digit${def.key}`) ||
			(event.altKey && /^[a-z]$/i.test(def.key) && event.code === `Key${def.key.toUpperCase()}`)) &&
		!!modifiersMatch &&
		event.shiftKey === (def.shift ?? false) &&
		event.altKey === (def.alt ?? false)
	)
}

export function keybindsOverlap(left: KeybindDefinition, right: KeybindDefinition): boolean {
	for (const ctrlKey of [false, true]) {
		for (const metaKey of [false, true]) {
			const event = {
				key: left.key,
				code: /^[a-z]$/i.test(left.key) ? `Key${left.key.toUpperCase()}` : '',
				ctrlKey,
				metaKey,
				shiftKey: left.shift ?? false,
				altKey: left.alt ?? false,
			}
			if (matchesKeybind(event, left) && matchesKeybind(event, right)) return true
		}
	}
	return false
}

export function toKeybindDefinition(event: KeyboardEvent): KeybindDefinition {
	return {
		key:
			(event.shiftKey || event.altKey) && /^Digit[1-9]$/.test(event.code)
				? event.code.slice(-1)
				: event.altKey && /^Key[A-Z]$/.test(event.code)
					? event.code.slice(3).toLowerCase()
					: event.key.toLowerCase(),
		ctrl: event.ctrlKey,
		shift: event.shiftKey,
		alt: event.altKey,
		meta: event.metaKey,
		preventDefault: true,
	}
}

export function formatKeybind(definition: KeybindDefinition, isMac: boolean): string {
	const keys: string[] = []
	if (definition.mod || (definition.ctrl && (definition.meta || definition.meta === undefined))) {
		keys.push(isMac ? '⌘' : 'Ctrl')
	} else {
		if (definition.ctrl) keys.push(isMac ? '⌃' : 'Ctrl')
		if (definition.meta) keys.push(isMac ? '⌘' : 'Meta')
	}
	if (definition.shift) keys.push(isMac ? '⇧' : 'Shift')
	if (definition.alt) keys.push(isMac ? '⌥' : 'Alt')
	const key = definition.key.toUpperCase()
	const labels: Record<string, string> = {
		ARROWLEFT: '←',
		ARROWRIGHT: '→',
		ARROWUP: '↑',
		ARROWDOWN: '↓',
		ENTER: '↵',
		ESCAPE: 'Esc',
		' ': 'Space',
	}
	keys.push(labels[key] ?? key)
	return keys.join(' ')
}
