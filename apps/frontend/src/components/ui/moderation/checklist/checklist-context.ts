import type { ActiveAction, NodeState, StageNode } from '@modrinth/moderation/src/types/node'
import type { Writer } from '@modrinth/moderation/src/types/node/mutate'
import { createContext } from '@modrinth/ui'
import type { ProjectStatus } from '@modrinth/utils'
import type { Component, ComputedRef, InjectionKey, Ref } from 'vue'

export interface LiveNode {
	isActive: boolean
	isVisible: boolean
	isFixActionable: boolean
	messageCount: number
	fixCount: number
	hasRequiredMissing: boolean
	activeActions: ActiveAction[]
}

export const STATE_KEY: InjectionKey<Ref<Record<string, Record<string, NodeState>>>> =
	Symbol('checklistState')

/* -------------------------------------------------------------------------- */
/*  Stage → on-screen element mapping (Phase 2)                               */
/* -------------------------------------------------------------------------- */

/**
 * Where a checklist stage's buttons are surfaced in the review view. Every key except
 * `global` is a real `ReviewTabId` (Tags / License / Links / Disclosures / Permissions were
 * promoted from Settings sub-sections to top-level tabs in Phase 2); `global` stages have no
 * on-screen element and live in the right-hand checklist module.
 */
export type ChecklistElementKey =
	| 'description'
	| 'gallery'
	| 'versions'
	| 'tags'
	| 'license'
	| 'links'
	| 'disclosures'
	| 'permissions'
	| 'global'

/** Keyed by stage id (see `packages/moderation/src/data/stages/*`). */
export const STAGE_ELEMENT: Record<string, ChecklistElementKey> = {
	'title-slug': 'description',
	summary: 'description',
	description: 'description',
	gallery: 'gallery',
	versions: 'versions',
	metadata: 'versions',
	reupload: 'versions',
	'undefined-project': 'versions',
	tags: 'tags',
	license: 'license',
	links: 'links',
	disclosures: 'disclosures',
	permissions: 'permissions',
	rules: 'global',
	're-review': 'global',
	'status-alerts': 'global',
	'post-approval': 'global',
}

export function elementForStage(stageId: string | undefined): ChecklistElementKey | undefined {
	return stageId ? STAGE_ELEMENT[stageId] : undefined
}

/* -------------------------------------------------------------------------- */
/*  Checklist engine context (Phase 2)                                        */
/* -------------------------------------------------------------------------- */

export interface TouchedChecklistNode {
	statePath: string[]
	stageId: string
	label: string
}

export interface ChecklistLockStatus {
	locked: boolean
	lockedBy?: { id: string; username: string; avatar_url?: string }
	lockedAt?: Date
	expiresAt?: Date
	expired?: boolean
	isOwnLock: boolean
}

/**
 * Everything the redistributed Phase 2 surfaces (element frames, right module, bottom
 * walkthrough) and the legacy floating widget need from the shared checklist engine.
 * The engine itself lives in `ModerationChecklistProvider.vue`.
 */
export interface ModerationChecklistEngine {
	/** True once the moderator is actively reviewing (lock + stages initialised). */
	active: Ref<boolean>

	// Stage graph + state
	nodeStates: Ref<Record<string, Record<string, NodeState>>>
	resolvedStages: Ref<StageNode[]>
	checklistLive: ComputedRef<Map<object, LiveNode>>
	touchedStages: Ref<Set<string>>
	/** Every issue node ever activated, keyed by `statePath.join('/')`. */
	touchedNodes: Ref<Record<string, TouchedChecklistNode>>
	/** Currently-active issue node paths (`statePath.join('/')`). */
	activeNodePaths: ComputedRef<Set<string>>
	writerForStage: (stageId: string) => Writer
	/** Toggle any node on/off by its full state path (`[stageId, ...groups, leafId]`). */
	setNodeActive: (statePath: string[], active: boolean) => void
	/** App-component map (`loader-picker`, `game-version-picker`) for `NodeRenderer`. */
	appComponents: Record<string, Component>

	// Stage navigation
	currentStage: Ref<number>
	currentStageObj: ComputedRef<StageNode>
	stageOptions: ComputedRef<ChecklistStageOption[]>
	setStage: (target: number | string) => void
	nextStage: () => void
	previousStage: () => void
	/** Open the review tab / settings sub-section that hosts the given stage. */
	focusStage: (stage: StageNode | undefined) => void

	// Element helpers
	stagesForElement: (key: ChecklistElementKey) => StageNode[]
	activeActionsForStage: (stage: StageNode) => ActiveAction[]

	// Message generation + editing
	message: Ref<string | null>
	generatedMessage: ComputedRef<boolean>
	loadingMessage: Ref<boolean>
	useSimpleEditor: Ref<boolean>
	generateMessage: () => Promise<void> | void
	onUploadHandler: (file: File) => Promise<string>

	// Decision
	moderationDecision: Ref<ProjectStatus | null>
	loadingModerationDecision: ComputedRef<boolean>
	approveSendStatus: ComputedRef<ProjectStatus>
	sendMessage: (status: ProjectStatus) => Promise<void> | void
	/** Post a plain reply to the moderation thread (no status change). */
	postThreadReply: (text: string, isPrivate?: boolean) => Promise<boolean>
	done: Ref<boolean>
	hasNextProject: Ref<boolean>

	// Lifecycle / utility
	resetProgress: () => void
	handleExit: () => void | Promise<void>
	skipCurrentProject: () => void | Promise<void>
	endChecklist: (status?: string) => void | Promise<void>
	reviewAnyway: () => void
	alreadyReviewed: Ref<boolean>
	reviewedAnyway: Ref<boolean>
	checklistHasState: ComputedRef<boolean>
	currentStageHasState: ComputedRef<boolean>
	isOnFirstStage: ComputedRef<boolean>

	// Lock
	lockStatus: Ref<ChecklistLockStatus | null>
	lockTimeRemaining: Ref<string | null>
	isLockedByOther: ComputedRef<boolean>
	requestTakeOver: () => void
}

export interface ChecklistStageOption {
	id: string
	label: string
	action: () => void
	text: string
	icon?: unknown
	messages?: number
	fixes?: number
	requiredMissing?: boolean
	visited?: boolean
	tone?: 'green'
}

export const [injectModerationChecklist, provideModerationChecklist] =
	createContext<ModerationChecklistEngine>('ModerationChecklistProvider', 'moderationChecklist')

/* -------------------------------------------------------------------------- */
/*  Review-layout data bridge                                                 */
/* -------------------------------------------------------------------------- */

/**
 * `ModerationReviewLayout` is teleported out of `ModerationChecklist` (so it can inject the
 * checklist engine), which means it can't receive `[type]/[project].vue`'s local computeds as
 * props. `[type]/[project].vue` provides them here instead.
 */
export interface ReviewLayoutData {
	project: Ref<Record<string, unknown>>
	projectV3: Ref<Record<string, unknown>>
	organization: Ref<unknown>
	members: Ref<unknown[]>
	creatorsLoading: Ref<boolean>
	isServerProject: Ref<boolean>
	serverDataLoaded: Ref<boolean>
	serverRequiredContent: Ref<unknown>
	serverRecommendedVersion: Ref<unknown>
	serverSupportedVersions: Ref<unknown[]>
	serverModpackLoaders: Ref<unknown[]>
}

export const [injectReviewLayoutData, provideReviewLayoutData] = createContext<ReviewLayoutData>(
	'type-project',
	'reviewLayoutData',
)
