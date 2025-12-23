import type { Labrinth } from '@modrinth/api-client'

export interface ModerationActions {
	tryGoNext: () => void
	tryGoBack: () => void
	tryGenerateMessage: () => void
	trySkipProject: () => void

	tryToggleCollapse: () => void
	tryResetProgress: () => void
	tryExitModeration: () => void

	tryApprove: () => void
	tryReject: () => void
	tryWithhold: () => void
	tryEditMessage: () => void

	tryToggleAction: (actionIndex: number) => void
	trySelectDropdownOption: (actionIndex: number, optionIndex: number) => void
	tryToggleChip: (actionIndex: number, chipIndex: number) => void

	tryFocusNextAction: () => void
	tryFocusPreviousAction: () => void
	tryActivateFocusedAction: () => void
}

export interface ModerationState {
	currentStage: number
	totalStages: number
	currentStageId: string | undefined
	currentStageTitle: string

	isCollapsed: boolean
	isDone: boolean
	hasGeneratedMessage: boolean
	isLoadingMessage: boolean
	isModpackPermissionsStage: boolean

	futureProjectCount: number
	visibleActionsCount: number

	focusedActionIndex: number | null
	focusedActionType: 'button' | 'toggle' | 'dropdown' | 'multi-select' | null
}

export interface ModerationContext {
	project: Labrinth.Projects.v2.Project
	state: ModerationState
	actions: ModerationActions
}

export interface KeybindDefinition {
	key: string
	ctrl?: boolean
	shift?: boolean
	alt?: boolean
	meta?: boolean
	preventDefault?: boolean
}

export interface KeybindListener {
	id: string
	keybind: KeybindDefinition | KeybindDefinition[] | string | string[]
	description: string
	enabled?: (ctx: ModerationContext) => boolean
	action: (ctx: ModerationContext) => void
}

export function parseKeybind(keybindString: string): KeybindDefinition {
	const parts = keybindString.split('+').map((p) => p.trim().toLowerCase())

	return {
		key: parts.find((p) => !['ctrl', 'shift', 'alt', 'meta', 'cmd'].includes(p)) || '',
		ctrl: parts.includes('ctrl') || parts.includes('cmd'),
		shift: parts.includes('shift'),
		alt: parts.includes('alt'),
		meta: parts.includes('meta') || parts.includes('cmd'),
		preventDefault: true,
	}
}

export function normalizeKeybind(keybind: KeybindDefinition | string): KeybindDefinition {
	return typeof keybind === 'string' ? parseKeybind(keybind) : keybind
}

export function matchesKeybind(event: KeyboardEvent, keybind: KeybindDefinition | string): boolean {
	const def = normalizeKeybind(keybind)
	return (
		event.key.toLowerCase() === def.key.toLowerCase() &&
		event.ctrlKey === (def.ctrl ?? false) &&
		event.shiftKey === (def.shift ?? false) &&
		event.altKey === (def.alt ?? false) &&
		event.metaKey === (def.meta ?? false)
	)
}

export function handleKeybind(
	event: KeyboardEvent,
	ctx: ModerationContext,
	keybinds: KeybindListener[],
): boolean {
	if (
		event.target instanceof HTMLInputElement ||
		event.target instanceof HTMLTextAreaElement ||
		(event.target as HTMLElement)?.closest('.cm-editor') ||
		(event.target as HTMLElement)?.classList?.contains('cm-content') ||
		(event.target as HTMLElement)?.classList?.contains('cm-line')
	) {
		return false
	}

	for (const keybind of keybinds) {
		if (keybind.enabled && !keybind.enabled(ctx)) {
			continue
		}

		const keybindDefs = Array.isArray(keybind.keybind)
			? keybind.keybind.map(normalizeKeybind)
			: [normalizeKeybind(keybind.keybind)]

		const matches = keybindDefs.some((def) => matchesKeybind(event, def))

		if (matches) {
			keybind.action(ctx)

			const shouldPrevent = keybindDefs.some((def) => def.preventDefault !== false)
			if (shouldPrevent) {
				event.preventDefault()
			}

			return true
		}
	}

	return false
}
