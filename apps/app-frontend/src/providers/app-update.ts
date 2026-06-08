import { computed, ref } from 'vue'

export const APP_UPDATE_POPUP_DELAY_MS = 24 * 60 * 60 * 1000

const UPDATE_PROMPT_STORAGE_KEY = 'modrinth-app-update-prompt-state'

export interface AppUpdate {
	rid: number
	version: string
	currentVersion?: string
}

interface UpdatePromptState {
	version: string
	stage: AppUpdatePromptStage
	actionableSince: number
	lastUserActionAt?: number
	popupShownAt?: number
}

export type AppUpdatePromptStage = 'available' | 'downloaded'

interface AppUpdateActions {
	download?: () => Promise<void> | void
	install?: () => Promise<void> | void
	changelog?: () => Promise<void> | void
}

const progress = ref(0)
const metered = ref(true)
const finishedDownloading = ref(false)
const downloading = ref(false)
const restarting = ref(false)
const availableUpdate = ref<AppUpdate | null>(null)
const updateSize = ref<number | null>(null)
const updatesEnabled = ref(true)

let actions: AppUpdateActions = {}

function getCurrentAppUpdatePromptStage(): AppUpdatePromptStage {
	return finishedDownloading.value ? 'downloaded' : 'available'
}

export const appUpdateState = {
	progress,
	metered,
	finishedDownloading,
	downloading,
	restarting,
	availableUpdate,
	updateSize,
	updatesEnabled,
	downloadProgress: computed(() => progress.value),
	downloadPercent: computed(() => Math.trunc(progress.value * 100)),
	isVisible: computed(() => !!availableUpdate.value && !restarting.value && updatesEnabled.value),
}

function readPromptState(): UpdatePromptState | null {
	try {
		const raw = localStorage.getItem(UPDATE_PROMPT_STORAGE_KEY)
		if (!raw) {
			return null
		}

		const parsed = JSON.parse(raw) as Partial<UpdatePromptState>
		if (!parsed.version || typeof parsed.actionableSince !== 'number') {
			return null
		}

		return {
			...parsed,
			stage: parsed.stage ?? 'available',
		} as UpdatePromptState
	} catch {
		return null
	}
}

function writePromptState(state: UpdatePromptState): void {
	try {
		localStorage.setItem(UPDATE_PROMPT_STORAGE_KEY, JSON.stringify(state))
	} catch (error) {
		console.warn('Failed to persist update prompt state:', error)
	}
}

export function markAppUpdateActionable(
	version: string,
	stage: AppUpdatePromptStage = 'available',
	now = Date.now(),
): void {
	const existing = readPromptState()
	if (existing?.version === version && existing.stage === stage) {
		return
	}

	writePromptState({
		version,
		stage,
		actionableSince: now,
	})
}

export function recordAppUpdateUserAction(
	version = availableUpdate.value?.version,
	stage: AppUpdatePromptStage = getCurrentAppUpdatePromptStage(),
): void {
	if (!version) {
		return
	}

	const now = Date.now()
	const existing = readPromptState()
	const isSamePrompt = existing?.version === version && existing.stage === stage
	writePromptState({
		version,
		stage,
		actionableSince: isSamePrompt ? existing.actionableSince : now,
		lastUserActionAt: now,
		popupShownAt: isSamePrompt ? existing.popupShownAt : undefined,
	})
}

export function markAppUpdatePopupShown(
	version: string,
	stage: AppUpdatePromptStage = 'available',
	now = Date.now(),
): void {
	const existing = readPromptState()
	const isSamePrompt = existing?.version === version && existing.stage === stage
	writePromptState({
		version,
		stage,
		actionableSince: isSamePrompt ? existing.actionableSince : now,
		lastUserActionAt: isSamePrompt ? existing.lastUserActionAt : undefined,
		popupShownAt: now,
	})
}

export function getNextAppUpdatePopupTime(
	version: string,
	stage: AppUpdatePromptStage = 'available',
): number | null {
	const existing = readPromptState()
	if (existing?.version !== version || existing.stage !== stage || existing.popupShownAt) {
		return null
	}

	return (
		Math.max(existing.actionableSince, existing.lastUserActionAt ?? 0) + APP_UPDATE_POPUP_DELAY_MS
	)
}

export function setAppUpdateActions(nextActions: AppUpdateActions): void {
	actions = nextActions
}

export async function downloadAvailableAppUpdate(): Promise<void> {
	recordAppUpdateUserAction(undefined, 'available')
	await actions.download?.()
}

export async function installAvailableAppUpdate(): Promise<void> {
	recordAppUpdateUserAction(undefined, 'downloaded')
	await actions.install?.()
}

export async function openAppUpdateChangelog(): Promise<void> {
	recordAppUpdateUserAction()
	await actions.changelog?.()
}
