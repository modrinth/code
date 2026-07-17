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
}

export type ModerationProjectContext = {
	project: Labrinth.Projects.v2.Project
	scope: 'project'
}

export type ModerationChecklistContext = {
	project: Labrinth.Projects.v2.Project
	scope: 'checklist'
	state: ModerationState,
	actions: ModerationActions,
}

export type ModerationContext = ModerationProjectContext | ModerationChecklistContext

export interface KeybindDefinition {
	key: string
	ctrl?: boolean
	shift?: boolean
	alt?: boolean
	meta?: boolean
	preventDefault?: boolean
}

export type BaseKeybindListener<T> = {
	keybind: KeybindDefinition | KeybindDefinition[] | string | string[]
	description: string
	scope: 'project' | 'checklist'
	enabled?: (ctx: T) => boolean
	action: (ctx: T) => void
}

export type KeybindProjectListener = BaseKeybindListener<ModerationProjectContext> & { scope: 'project' }
export type KeybindChecklistListener = BaseKeybindListener<ModerationChecklistContext> & { scope: 'checklist' }
export type KeybindListener = KeybindProjectListener | KeybindChecklistListener

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

export function toKeybindDefinition(event: KeyboardEvent): KeybindDefinition {
	return {
		key: event.key.toLowerCase(),
		ctrl: event.ctrlKey,
		shift: event.shiftKey,
		alt: event.altKey,
		meta: event.metaKey,
		preventDefault: true,
	}
}
