<template>
	<KeybindsModal ref="keybindsModal" />
	<ConfirmModal
		v-if="lockStatus?.locked && !lockStatus?.isOwnLock"
		ref="takeOverModal"
		title="Override moderation lock"
		description="Are you sure you want to override?"
		:has-to-type="false"
		:markdown="false"
		proceed-label="Take over"
		@proceed="confirmTakeOverOverride"
	/>
	<div
		tabindex="0"
		class="moderation-checklist flex max-h-[calc(100vh-2rem)] w-[600px] max-w-full flex-col overflow-hidden rounded-2xl border-[1px] border-solid border-orange bg-bg-raised p-4 transition-all delay-200 duration-200 ease-in-out"
		:class="{ '!w-fit': collapsed, locked: lockStatus?.locked && !lockStatus?.isOwnLock }"
	>
		<div class="flex grow-0 flex-col gap-1">
			<div class="flex items-center gap-2">
				<h1 class="m-0 mr-auto">
					<TeleportOverflowMenu
						v-if="canOpenStageSelectorFromTitle"
						:options="stageOptions"
						btn-class="inline-flex items-center gap-2 bg-transparent p-0 text-2xl font-extrabold text-contrast"
					>
						<component :is="currentStageObj._icon ?? ScaleIcon" class="text-orange" />
						{{ checklistTitleText }}
						<template v-for="opt in stageOptions" #[opt.id] :key="opt.id">
							<component :is="opt.icon" v-if="opt.icon" class="mr-2" />
							<span>
								{{ opt.text }}<span v-if="opt.requiredMissing" class="font-bold text-red">*</span>
							</span>
							<span v-if="opt.messages" class="ml-auto pl-2 font-semibold opacity-75">{{
								opt.messages
							}}</span>
							<span v-if="opt.fixes" class="pl-2 font-semibold text-blue">{{ opt.fixes }}</span>
						</template>
					</TeleportOverflowMenu>
					<button
						v-else
						disabled
						class="inline-flex cursor-default items-center gap-2 bg-transparent p-0 text-2xl font-extrabold text-contrast"
					>
						<component :is="currentStageObj._icon ?? ScaleIcon" class="text-orange" />
						{{ checklistTitleText }}
					</button>
				</h1>
				<ButtonStyled circular>
					<button v-tooltip="`Keyboard shortcuts`" @click="keybindsModal?.show($event)">
						<KeyboardIcon />
					</button>
				</ButtonStyled>
				<ButtonStyled v-if="currentStageObj._guidanceUrl" circular>
					<a v-tooltip="`Stage guidance`" target="_blank" :href="currentStageObj._guidanceUrl">
						<FileTextIcon />
					</a>
				</ButtonStyled>
				<ButtonStyled circular color="red" color-fill="none" hover-color-fill="background">
					<button v-tooltip="`Reset progress`" @click="resetProgress">
						<BrushCleaningIcon />
					</button>
				</ButtonStyled>
				<ButtonStyled circular color="red" color-fill="none" hover-color-fill="background">
					<button v-tooltip="`Exit moderation`" @click="handleExit">
						<XIcon />
					</button>
				</ButtonStyled>
				<ButtonStyled circular>
					<button v-tooltip="collapsed ? `Expand` : `Collapse`" @click="emit('toggleCollapsed')">
						<DropdownIcon class="transition-transform" :class="{ 'rotate-180': collapsed }" />
					</button>
				</ButtonStyled>
			</div>
		</div>
		<p
			v-if="
				currentStageObj._hint &&
				!collapsed &&
				!alreadyReviewed &&
				!done &&
				!generatedMessage &&
				!(lockStatus?.locked && !lockStatus?.isOwnLock)
			"
			class="m-0 text-sm text-secondary"
		>
			{{ currentStageObj._hint }}
		</p>
		<Collapsible
			base-class="grow min-h-0"
			class="flex min-h-0 grow flex-col"
			:collapsed="collapsed"
		>
			<div class="mb-3 mt-2 h-[1px] w-full bg-divider" />

			<div v-if="lockStatus?.locked && !lockStatus?.isOwnLock" class="flex flex-1 flex-col">
				<div class="flex flex-1 flex-col items-center justify-center gap-4 py-8 text-center">
					<LockIcon class="size-8 text-orange" />
					<span class="text-secondary">
						This project
						{{ lockStatus.expired ? 'was being' : 'is currently being' }}
						moderated<template v-if="lockStatus.lockedBy?.username"> by</template>
					</span>
					<span v-if="lockStatus.lockedBy?.username" class="inline-flex items-center gap-1">
						<Avatar :src="lockStatus.lockedBy?.avatar_url" size="2rem" circle />
						<strong class="text-contrast">@{{ lockStatus.lockedBy.username }}</strong>
					</span>
					<span v-if="lockTimeRemaining && !lockStatus.expired" class="text-secondary">
						Lock expires in {{ lockTimeRemaining }}
					</span>
				</div>
				<div class="mt-auto">
					<div
						class="mt-4 flex grow justify-between gap-2 border-0 border-t-[1px] border-solid border-surface-5 pt-4"
					>
						<div class="flex items-center gap-2">
							<ButtonStyled @click="openTakeOverModal">
								<button>
									<LockIcon aria-hidden="true" />
									Take over
								</button>
							</ButtonStyled>
						</div>
						<div class="flex items-center gap-2">
							<ButtonStyled
								v-if="moderationQueue.isQueueMode && moderationQueue.queueLength > 1"
								color="brand"
								@click="skipToNextProject"
							>
								<button>
									<RightArrowIcon aria-hidden="true" />
									Next project ({{ moderationQueue.queueLength }} left)
								</button>
							</ButtonStyled>
						</div>
					</div>
				</div>
			</div>

			<div v-else-if="alreadyReviewed" class="flex flex-1 flex-col">
				<div class="flex flex-1 flex-col items-center justify-center gap-4 py-8 text-center">
					<CheckIcon class="size-8 text-green" />
					<span class="text-secondary"> This project was already moderated. </span>
				</div>
				<div class="mt-auto">
					<div
						class="mt-4 flex grow justify-between gap-2 border-0 border-t-[1px] border-solid border-surface-5 pt-4"
					>
						<div class="flex items-center gap-2">
							<ButtonStyled @click="reviewAnyway">
								<button>
									<ScaleIcon aria-hidden="true" />
									Review anyway
								</button>
							</ButtonStyled>
						</div>
						<div class="flex items-center gap-2">
							<ButtonStyled
								v-if="moderationQueue.isQueueMode && moderationQueue.queueLength > 1"
								color="brand"
								@click="skipToNextProject"
							>
								<button>
									<RightArrowIcon aria-hidden="true" />
									Next project ({{ moderationQueue.queueLength }} left)
								</button>
							</ButtonStyled>
						</div>
					</div>
				</div>
			</div>

			<template v-else>
				<div class="flex min-h-0 flex-1 flex-col">
					<div v-if="done">
						<p>
							You are done moderating this project!
							<template v-if="moderationQueue.hasItems">
								There are
								{{ moderationQueue.queueLength }} left.
							</template>
						</p>
					</div>
					<div v-else-if="generatedMessage" class="flex min-h-0 flex-1 flex-col gap-2">
						<ButtonStyled class="shrink-0 self-start">
							<button @click="useSimpleEditor = !useSimpleEditor">
								<template v-if="!useSimpleEditor">
									<ToggleLeftIcon aria-hidden="true" />
									Use simple mode
								</template>
								<template v-else>
									<ToggleRightIcon aria-hidden="true" />
									Use advanced mode
								</template>
							</button>
						</ButtonStyled>
						<div class="min-h-0 flex-1 overflow-y-auto">
							<MarkdownEditor
								v-if="!useSimpleEditor"
								v-model="messageText"
								:max-height="400"
								placeholder="No message generated."
								:disabled="false"
								:heading-buttons="false"
								:on-image-upload="onUploadHandler"
							/>
							<StyledInput
								v-else
								v-model="messageText"
								multiline
								placeholder="No message generated."
								autocomplete="off"
								input-class="h-[400px] font-mono"
							/>
						</div>
					</div>
					<div v-else-if="isModpackPermissionsStage">
						<ModpackPermissionsFlow
							v-model="modpackJudgements"
							:project-id="projectV2.id"
							:project-updated="projectV2.updated"
							@complete="handleModpackPermissionsComplete"
						/>
					</div>
					<div v-else class="flex min-h-0 flex-1 flex-col">
						<NodeRenderer
							class="min-h-0 flex-1 overflow-y-auto"
							:nodes="
								resolveChildren(
									currentStageObj,
									(nodeStates[currentStageObj.id!] ?? {}) as Record<string, NodeState>,
								)
							"
							:show-context="(nodeStates[currentStageObj.id!] ?? {}) as Record<string, NodeState>"
							:on-image-upload="onUploadHandler"
						/>
					</div>
				</div>

				<!-- Stage control buttons -->
				<div class="mt-auto">
					<div
						class="mt-4 flex grow justify-between gap-2 border-0 border-t-[1px] border-solid border-surface-5 pt-4"
					>
						<div class="flex items-center gap-2">
							<ButtonStyled v-if="!done && !generatedMessage && moderationQueue.hasItems">
								<button @click="skipCurrentProject">
									<XIcon aria-hidden="true" />
									Skip ({{ moderationQueue.queueLength }} left)
								</button>
							</ButtonStyled>
						</div>

						<div class="flex items-center gap-2">
							<ButtonStyled v-if="!done" circular>
								<TeleportOverflowMenu :options="stageOptions">
									<ListBulletedIcon />
									<span class="sr-only">Stages</span>
									<template v-for="opt in stageOptions" #[opt.id] :key="opt.id">
										<component :is="opt.icon" v-if="opt.icon" class="mr-2" />
										<span>
											{{ opt.text
											}}<span v-if="opt.requiredMissing" class="font-bold text-red">*</span>
										</span>
										<span v-if="opt.messages" class="ml-auto pl-2 font-semibold opacity-75">{{
											opt.messages
										}}</span>
										<span v-if="opt.fixes" class="pl-2 font-semibold text-blue">{{
											opt.fixes
										}}</span>
									</template>
								</TeleportOverflowMenu>
							</ButtonStyled>

							<div v-if="done">
								<ButtonStyled color="brand">
									<button @click="endChecklist(undefined)">
										<template v-if="hasNextProject">
											<RightArrowIcon aria-hidden="true" />
											Next project ({{ moderationQueue.queueLength }} left)
										</template>
										<template v-else>
											<CheckIcon aria-hidden="true" />
											All done!
										</template>
									</button>
								</ButtonStyled>
							</div>

							<div v-else-if="generatedMessage" class="flex items-center gap-2">
								<ButtonStyled>
									<button :disabled="loadingModerationDecision" @click="goBackToStages">
										<LeftArrowIcon aria-hidden="true" />
										Edit
									</button>
								</ButtonStyled>
								<ButtonStyled color="red">
									<button :disabled="loadingModerationDecision" @click="sendMessage('rejected')">
										<SpinnerIcon
											v-if="moderationDecision === 'rejected'"
											class="animate-spin"
											aria-hidden="true"
										/>
										<XIcon v-else aria-hidden="true" />
										Reject
									</button>
								</ButtonStyled>
								<ButtonStyled color="orange">
									<button :disabled="loadingModerationDecision" @click="sendMessage('withheld')">
										<SpinnerIcon
											v-if="moderationDecision === 'withheld'"
											class="animate-spin"
											aria-hidden="true"
										/>
										<LinkIcon v-else aria-hidden="true" />
										Withhold
									</button>
								</ButtonStyled>
								<ButtonStyled color="green">
									<button
										:disabled="loadingModerationDecision"
										@click="sendMessage(approveSendStatus)"
									>
										<SpinnerIcon
											v-if="moderationDecision === approveSendStatus"
											class="animate-spin"
											aria-hidden="true"
										/>
										<CheckIcon v-else aria-hidden="true" />
										Approve
									</button>
								</ButtonStyled>
							</div>

							<div v-else class="flex items-center gap-2">
								<ButtonStyled>
									<button :disabled="!hasValidPreviousStage" @click="previousStage">
										<LeftArrowIcon aria-hidden="true" /> Previous
									</button>
								</ButtonStyled>
								<ButtonStyled v-if="!isLastVisibleStage" color="brand">
									<button @click="nextStage"><RightArrowIcon aria-hidden="true" /> Next</button>
								</ButtonStyled>
								<ButtonStyled v-else color="brand" :disabled="loadingMessage">
									<button @click="generateMessage">
										<CheckIcon aria-hidden="true" />
										{{ loadingMessage ? 'Generating...' : 'Generate Message' }}
									</button>
								</ButtonStyled>
							</div>
						</div>
					</div>
				</div>
			</template>
		</Collapsible>
	</div>
</template>

<script lang="ts" setup>
import {
	BrushCleaningIcon,
	CheckIcon,
	DropdownIcon,
	FileTextIcon,
	KeyboardIcon,
	LeftArrowIcon,
	LinkIcon,
	ListBulletedIcon,
	LockIcon,
	RightArrowIcon,
	ScaleIcon,
	SpinnerIcon,
	ToggleLeftIcon,
	ToggleRightIcon,
	XIcon,
} from '@modrinth/assets'
import type {
	ActionBuilder,
	IdentifiedNodeBuilder,
	NodeState,
	Priority,
	StageNodeBuilder,
	ValueNodeBuilder,
} from '@modrinth/moderation'
import {
	createTrackedPatch,
	expandVariables,
	getBooleanChildState,
	GLOBAL_STATE_KEY,
	handleKeybind,
	isNodeActive,
	kebabToTitleCase,
	loadMd,
	resolve,
	resolveChildren,
	setMessageProject,
	useStages,
	walkNodes,
} from '@modrinth/moderation'
import {
	Avatar,
	ButtonStyled,
	Collapsible,
	ConfirmModal,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	MarkdownEditor,
	StyledInput,
	useDebugLogger,
} from '@modrinth/ui'
import TeleportOverflowMenu from '@modrinth/ui/src/components/base/TeleportOverflowMenu.vue'
import type { ModerationJudgements, ModerationModpackItem, ProjectStatus } from '@modrinth/utils'
import { useQueryClient } from '@tanstack/vue-query'
import { useDebounceFn } from '@vueuse/core'
import type { Component } from 'vue'
import { computed, nextTick, provide, ref, toRaw, watch } from 'vue'

import { useGeneratedState } from '~/composables/generated'
import { useImageUpload } from '~/composables/image-upload.ts'
import { getProjectTypeForUrlShorthand } from '~/helpers/projects.js'
import {
	clearChecklistState,
	loadChecklistState,
	saveChecklistState,
} from '~/services/moderation-checklist-storage.ts'
import type { LockAcquireResponse } from '~/services/moderation-queue.ts'
import { useModerationQueue } from '~/services/moderation-queue.ts'

import { type ActiveAction, type LiveNode, NODE_META_KEY, STATE_KEY } from './checklist-context'
import KeybindsModal from './ChecklistKeybindsModal.vue'
import ModpackPermissionsFlow from './ModpackPermissionsFlow.vue'
import NodeRenderer from './NodeRenderer.vue'

const notifications = injectNotificationManager()
const { addNotification } = notifications
const debug = useDebugLogger('ModerationChecklist')
const keybinds = useModerationKeybinds()

const keybindsModal = ref<InstanceType<typeof KeybindsModal>>()
const takeOverModal = ref<InstanceType<typeof ConfirmModal>>()

const props = defineProps<{
	collapsed: boolean
}>()

const { projectV2, projectV3, versions, loadVersions, invalidate } = injectProjectPageContext()
setMessageProject(projectV3, projectV2)

const nodeStates = ref<Record<string, Record<string, NodeState>>>({})
const resolvedStages = ref(useStages(nodeStates))
const client = injectModrinthClient()

const moderationQueue = useModerationQueue()
const queryClient = useQueryClient()
const tags = useGeneratedState()
const auth = await useAuth()

const lockStatus = ref<{
	locked: boolean
	lockedBy?: { id: string; username: string; avatar_url?: string }
	lockedAt?: Date
	expiresAt?: Date
	expired?: boolean
	isOwnLock: boolean
} | null>(null)
const lockError = ref(false)
const lockCheckInterval = ref<ReturnType<typeof setInterval> | null>(null)
const lockCountdownInterval = ref<ReturnType<typeof setInterval> | null>(null)
const lockTimeRemaining = ref<string | null>(null)
const alreadyReviewed = ref(false)

// Prefetch queue for parallel lock checking and instant navigation
interface PrefetchedProject {
	projectId: string
	slug: string // For canonical URL navigation
	projectType: string // For canonical URL navigation
	validatedAt: number
	skippedIds: string[] // IDs that were locked when this was prefetched
}

const prefetchQueue = ref<PrefetchedProject[]>([])
const isPrefetching = ref(false)

const PREFETCH_STALE_MS = 30_000 // 30 seconds
const PREFETCH_TARGET_COUNT = 3 // Keep 3 unlocked projects ready
const PREFETCH_BATCH_SIZE = 5 // Check 5 at a time in parallel

async function handleVisibilityChange() {
	if (document.visibilityState === 'visible' && lockStatus.value?.isOwnLock) {
		// Immediately refresh the lock when returning to the tab
		// This handles cases where the heartbeat was throttled while backgrounded
		const refreshResult = await moderationQueue.refreshLock()
		if (!refreshResult.success) {
			handleLockLost(refreshResult)
			return
		}
		// Refresh prefetch queue when tab becomes visible (not debounced)
		maintainPrefetchQueue()
	}
}

function updateLockCountdown() {
	if (!lockStatus.value?.lockedAt || lockStatus.value?.isOwnLock) {
		lockTimeRemaining.value = null
		return
	}

	const lockedAt = new Date(lockStatus.value.lockedAt)
	const expiresAt = lockStatus.value.expiresAt
		? new Date(lockStatus.value.expiresAt)
		: new Date(lockedAt.getTime() + 15 * 60 * 1000)
	const now = new Date()
	const remainingMs = expiresAt.getTime() - now.getTime()

	if (remainingMs <= 0) {
		lockTimeRemaining.value = null
		lockStatus.value.expired = true
		clearLockCountdown()
		return
	}

	const minutes = Math.floor(remainingMs / 60000)
	const seconds = Math.floor((remainingMs % 60000) / 1000)
	lockTimeRemaining.value = `${minutes}:${seconds.toString().padStart(2, '0')}`
}

function clearLockCountdown() {
	if (lockCountdownInterval.value) {
		clearInterval(lockCountdownInterval.value)
		lockCountdownInterval.value = null
	}
	lockTimeRemaining.value = null
}

function startLockHeartbeat() {
	lockCheckInterval.value = setInterval(
		async () => {
			const result = await moderationQueue.refreshLock()
			if (!result.success) {
				handleLockLost(result)
			}
		},
		5 * 60 * 1000,
	)
}

function handleLockLost(result: LockAcquireResponse) {
	clearInterval(lockCheckInterval.value!)
	lockCheckInterval.value = null
	clearLockCountdown()

	lockStatus.value = {
		locked: result.locked_by != null,
		lockedBy: result.locked_by,
		lockedAt: result.locked_at ? new Date(result.locked_at) : undefined,
		expiresAt: result.expires_at ? new Date(result.expires_at) : undefined,
		expired: result.expired,
		isOwnLock: false,
	}
	lockError.value = false

	if (result.locked_by) {
		addNotification({
			title: 'Lock taken over',
			text: `@${result.locked_by.username} is now moderating this project.`,
			type: 'warning',
		})
		updateLockCountdown()
		lockCountdownInterval.value = setInterval(updateLockCountdown, 1000)
	} else {
		addNotification({
			title: 'Moderation lock lost',
			text: 'Your lock on this project has expired. Acquire the lock again to continue.',
			type: 'warning',
		})
	}
}

function handleLockAcquired() {
	lockStatus.value = { locked: false, isOwnLock: true }
	lockError.value = false
	clearLockCountdown()
	startLockHeartbeat()
	maintainPrefetchQueue() // Start prefetching immediately (not debounced)
}

function handleLockUnavailable() {
	lockError.value = true
	lockStatus.value = { locked: false, isOwnLock: false }
	clearLockCountdown()
	addNotification({
		title: 'Lock unavailable',
		text: 'Could not acquire moderation lock. Others may also be moderating this project.',
		type: 'warning',
	})
}

async function navigateToNextUnlockedProject(): Promise<boolean> {
	// Remove stale entries first
	const now = Date.now()
	prefetchQueue.value = prefetchQueue.value.filter((p) => now - p.validatedAt < PREFETCH_STALE_MS)

	if (prefetchQueue.value.length === 0) return false

	const next = prefetchQueue.value[0]

	// Quick re-check if close to expiry (last 5 seconds of TTL)
	if (now - next.validatedAt > PREFETCH_STALE_MS - 5000) {
		const recheckResults = await batchCheckQueueCandidates([next.projectId])
		const recheck = recheckResults.get(next.projectId)
		if (!isEligibleQueueCandidate(recheck)) {
			prefetchQueue.value.shift()
			return navigateToNextUnlockedProject()
		}
	}

	// Remove from queue after validation
	prefetchQueue.value.shift()

	// Mark skipped projects as completed
	await Promise.all(
		next.skippedIds.map((id) => moderationQueue.completeCurrentProject(id, 'skipped')),
	)

	notifySkippedQueueProjects(next.skippedIds.length)

	maintainPrefetchQueue()

	navigateToQueueProject(
		{ slug: next.slug, projectType: next.projectType, locked: false, isProcessing: true },
		next.projectId,
	)
	return true
}

const modpackPermissionsComplete = ref(false)
const modpackJudgements = ref<ModerationJudgements>({})
const isModpackPermissionsStage = computed(() => {
	return currentStageObj.value.id === 'permissions'
})

async function onUploadHandler(file: File) {
	const response = await useImageUpload(file, {
		context: 'thread_message',
		projectID: projectV2.value.id,
	})
	return response.url
}

const useSimpleEditor = ref(false)
const checklistPersistenceProjectId = projectV2.value.id
const persistedState = import.meta.client
	? await loadChecklistState(checklistPersistenceProjectId)
	: null
nodeStates.value = persistedState?.state ?? {}
const reviewedAnyway = ref(persistedState?.reviewAnyway ?? false)
const message = ref<string | null>(persistedState?.message ?? null)
const generatedActiveActions = ref<ActiveAction[] | null>(null)
const resolvedMessageAvailability = ref<Map<ActionBuilder, boolean>>(new Map())
const generatedMessage = computed(() => message.value !== null)
const loadingMessage = ref(false)
const moderationDecision = ref<ProjectStatus | null>(null)
const loadingModerationDecision = computed(() => moderationDecision.value !== null)
const approveSendStatus = computed<ProjectStatus>(() => {
	const requested = projectV2.value.requested_status
	return requested ?? 'approved'
})
const done = ref(false)
const messageText = computed({
	get: () => message.value ?? '',
	set: (v: string) => {
		message.value = v
	},
})

function clearGeneratedMessageState() {
	message.value = null
	generatedActiveActions.value = null
}

function handleModpackPermissionsComplete() {
	modpackPermissionsComplete.value = true
}

const emit = defineEmits<{
	exit: []
	toggleCollapsed: []
}>()

async function handleExit() {
	// Release if we own the lock, or if there was an error checking (we might still own it)
	const projectId = projectV2.value?.id
	if (projectId && (lockStatus.value?.isOwnLock || lockError.value)) {
		const released = await moderationQueue.releaseLock(projectId)
		if (!released && lockStatus.value?.isOwnLock) {
			console.warn('Failed to release moderation lock for project:', projectId)
		}
	}
	await persistStateImmediately(false, true)
	persistenceEnabled = false
	emit('exit')
}

function openTakeOverModal() {
	takeOverModal.value?.show()
}

async function confirmTakeOverOverride() {
	const projectId = projectV2.value?.id
	if (!projectId) {
		console.warn('[confirmTakeOverOverride] No project ID available')
		return
	}
	const result = await moderationQueue.overrideLock(projectId)

	if (result.success) {
		addNotification({
			title: 'Moderation lock overridden',
			text: 'You are now moderating this project.',
			type: 'success',
		})
		handleLockAcquired()
	} else if (result.locked_by) {
		lockStatus.value = {
			locked: true,
			lockedBy: result.locked_by,
			lockedAt: result.locked_at ? new Date(result.locked_at) : undefined,
			expiresAt: result.expires_at ? new Date(result.expires_at) : undefined,
			expired: result.expired,
			isOwnLock: false,
		}
		lockError.value = false

		updateLockCountdown()
		if (!lockCountdownInterval.value) {
			lockCountdownInterval.value = setInterval(updateLockCountdown, 1000)
		}
	} else {
		handleLockUnavailable()
	}
}

function reviewAnyway() {
	alreadyReviewed.value = false
	reviewedAnyway.value = true
	persistState()
	// Start prefetching the next project in the background
	maintainPrefetchQueue()
}

// Batch check locks, processing status, and fetch project metadata in parallel
interface QueueCandidateCheck {
	locked: boolean
	expired?: boolean
	isOwnLock?: boolean
	slug?: string
	projectType?: string
	status?: string
	isProcessing: boolean
}

function isEligibleQueueCandidate(result: QueueCandidateCheck | undefined): boolean {
	if (!result?.isProcessing) return false
	return !result.locked || !!result.expired || !!result.isOwnLock
}

function notifySkippedQueueProjects(count: number) {
	if (count <= 0) return
	addNotification({
		title: 'Skipped projects',
		text: `Skipped ${count} project(s) already moderated or locked by others.`,
		type: 'info',
		autoCloseMs: 2000,
	})
}

function navigateToQueueProject(result: QueueCandidateCheck, projectId: string) {
	if (result.slug && result.projectType) {
		const urlType = getProjectTypeForUrlShorthand(result.projectType, [], tags.value)
		navigateTo({
			path: `/${urlType}/${result.slug}`,
			state: { showChecklist: true },
		})
	} else {
		navigateTo({
			name: 'type-project',
			params: { type: 'project', project: projectId },
			state: { showChecklist: true },
		})
	}
}

async function batchCheckQueueCandidates(
	projectIds: string[],
): Promise<Map<string, QueueCandidateCheck>> {
	const results = new Map<string, QueueCandidateCheck>()

	const checks = await Promise.allSettled(
		projectIds.map(async (id) => {
			const [lockResponse, projectData] = await Promise.all([
				moderationQueue.checkLock(id),
				useBaseFetch(`project/${id}`, { method: 'GET' }).catch(() => null),
			])

			const status = (projectData as { status?: string } | null)?.status

			return {
				id,
				locked: lockResponse.locked,
				expired: lockResponse.expired,
				isOwnLock: lockResponse.is_own_lock,
				slug: (projectData as { slug?: string } | null)?.slug,
				projectType: (projectData as { project_type?: string } | null)?.project_type,
				status,
				isProcessing: projectData === null ? true : status === 'processing',
			}
		}),
	)

	checks.forEach((result, index) => {
		if (result.status === 'fulfilled') {
			results.set(result.value.id, result.value)
		} else {
			results.set(projectIds[index], { locked: false, isProcessing: true })
		}
	})

	return results
}

async function findNextEligibleQueueProject(candidateIds: string[]) {
	const skippedIds: string[] = []
	let checkedCount = 0

	while (checkedCount < candidateIds.length) {
		const batch = candidateIds.slice(checkedCount, checkedCount + PREFETCH_BATCH_SIZE)
		checkedCount += batch.length

		const results = await batchCheckQueueCandidates(batch)

		for (const id of batch) {
			const result = results.get(id)
			if (isEligibleQueueCandidate(result)) {
				return { projectId: id, result: result!, skippedIds: [...skippedIds] }
			}
			skippedIds.push(id)
		}
	}

	return null
}

// Maintain a queue of prefetched unlocked projects for instant navigation
async function maintainPrefetchQueue() {
	if (isPrefetching.value) return
	if (!moderationQueue.isQueueMode) return

	const currentProjectId = projectV2.value?.id

	isPrefetching.value = true

	try {
		// 1. Remove stale entries (validated > 30s ago)
		const now = Date.now()
		prefetchQueue.value = prefetchQueue.value.filter((p) => now - p.validatedAt < PREFETCH_STALE_MS)

		// 2. Remove entries for current project
		if (currentProjectId) {
			prefetchQueue.value = prefetchQueue.value.filter((p) => p.projectId !== currentProjectId)
		}

		// 3. If queue is full enough, exit early
		if (prefetchQueue.value.length >= PREFETCH_TARGET_COUNT) {
			return
		}

		// 4. Get remaining queue items (excluding current and already prefetched)
		const prefetchedIds = new Set(prefetchQueue.value.map((p) => p.projectId))
		const queueItems = [...moderationQueue.currentQueue.items]
		const currentIndex = currentProjectId ? queueItems.indexOf(currentProjectId) : -1
		const remainingItems =
			currentIndex >= 0 ? queueItems.slice(currentIndex + 1) : queueItems.slice(1)

		const candidateIds = remainingItems.filter((id) => !prefetchedIds.has(id))

		if (candidateIds.length === 0) return

		const skippedIds: string[] = []
		let checkedCount = 0

		while (
			prefetchQueue.value.length < PREFETCH_TARGET_COUNT &&
			checkedCount < candidateIds.length
		) {
			const batch = candidateIds.slice(checkedCount, checkedCount + PREFETCH_BATCH_SIZE)
			checkedCount += batch.length

			const results = await batchCheckQueueCandidates(batch)

			for (const id of batch) {
				const result = results.get(id)
				if (isEligibleQueueCandidate(result)) {
					prefetchQueue.value.push({
						projectId: id,
						slug: result?.slug ?? '',
						projectType: result?.projectType ?? '',
						validatedAt: Date.now(),
						skippedIds: [...skippedIds],
					})

					if (prefetchQueue.value.length >= PREFETCH_TARGET_COUNT) break
				} else {
					skippedIds.push(id)
				}
			}
		}
	} finally {
		isPrefetching.value = false
	}
}

// Debounced prefetch to prevent spam from rapid stage changes
const debouncedPrefetch = useDebounceFn(maintainPrefetchQueue, 300)

async function skipToNextProject() {
	// Skip the current project
	const currentProjectId = projectV2.value?.id
	if (!currentProjectId) {
		console.warn('[skipToNextProject] No current project ID, aborting')
		return
	}
	debug('[skipToNextProject] Starting. Current project:', currentProjectId)
	debug('[skipToNextProject] Queue before complete:', [...moderationQueue.currentQueue.items])

	await moderationQueue.completeCurrentProject(currentProjectId, 'skipped')

	debug('[skipToNextProject] Queue after complete:', [...moderationQueue.currentQueue.items])
	debug('[skipToNextProject] hasItems:', moderationQueue.hasItems)

	// Use prefetched data if available
	if (await navigateToNextUnlockedProject()) {
		debug('[skipToNextProject] Used prefetch, returning')
		return
	}

	debug('[skipToNextProject] No prefetch, entering fallback with batch checking')

	const remainingIds = moderationQueue.currentQueue.items.filter((id) => id !== currentProjectId)

	if (remainingIds.length > 0) {
		const next = await findNextEligibleQueueProject(remainingIds)

		if (next) {
			await Promise.all(
				next.skippedIds.map((id) => moderationQueue.completeCurrentProject(id, 'skipped')),
			)
			notifySkippedQueueProjects(next.skippedIds.length)
			navigateToQueueProject(next.result, next.projectId)
			return
		}

		await Promise.all(
			remainingIds.map((id) => moderationQueue.completeCurrentProject(id, 'skipped')),
		)

		debug('[skipToNextProject] No eligible projects in queue')
		addNotification({
			title: 'No projects available',
			text: 'All remaining projects are already moderated or locked by others.',
			type: 'warning',
		})
	}

	debug('[skipToNextProject] Emitting exit')
	emit('exit')
}

function resetProgress() {
	currentStage.value = findFirstValidStage()
	nodeStates.value = {}

	done.value = false
	clearGeneratedMessageState()
	loadingMessage.value = false
	moderationDecision.value = null

	localStorage.removeItem(`modpack-permissions-${projectV2.value.id}`)
	localStorage.removeItem(`modpack-permissions-index-${projectV2.value.id}`)

	sessionStorage.removeItem(`modpack-permissions-data-${projectV2.value.id}`)
	sessionStorage.removeItem(`modpack-permissions-permanent-no-${projectV2.value.id}`)
	sessionStorage.removeItem(`modpack-permissions-updated-${projectV2.value.id}`)

	modpackPermissionsComplete.value = false
	modpackJudgements.value = {}
}

function findFirstValidStage(): number {
	for (let i = 0; i < resolvedStages.value.length; i++) {
		if (shouldShowStageIndex(i)) {
			return i
		}
	}
	return 0
}

const currentStageObj = computed(() => resolvedStages.value[currentStage.value])
const isLockedByOther = computed(() => lockStatus.value?.locked && !lockStatus.value?.isOwnLock)
const canOpenStageSelectorFromTitle = computed(
	() => !alreadyReviewed.value && !done.value && !isLockedByOther.value,
)

const checklistTitleText = computed(() => {
	if (alreadyReviewed.value || done.value) return 'Moderation'
	if (generatedMessage.value) return 'Generated Message'

	return currentStageObj.value.label ?? kebabToTitleCase(currentStageObj.value.id)
})
const checklistLive = computed<Map<IdentifiedNodeBuilder, LiveNode>>(() => {
	const map = new Map<IdentifiedNodeBuilder, LiveNode>()

	for (const stage of resolvedStages.value) {
		const stageState = (nodeStates.value[stage.id!] ?? {}) as Record<string, NodeState>

		let isVisible = false
		let messageCount = 0
		let fixCount = 0
		let hasRequiredMissing = false
		const stageActiveActions: ActiveAction[] = []

		if (stage._shown === undefined || resolve(stage._shown)) {
			if (stage._action) {
				messageCount++
				stageActiveActions.push({ action: stage._action, state: stageState, statePath: [] })
			}

			walkNodes(resolveChildren(stage, stageState), stageState, (node, nodeState, localState) => {
				isVisible = true
				const active = isNodeActive(node, nodeState)
				const actionState =
					node.type === 'toggle' || node.type === 'check' || node.type === 'option'
						? (getBooleanChildState(nodeState) as Record<string, NodeState>)
						: localState
				const isRequired = !!(node as ValueNodeBuilder)._required
				const nodeActiveActions: ActiveAction[] = []

				let isFixActionable = false
				if (active) {
					if (node._action) {
						messageCount++
						nodeActiveActions.push({
							action: node._action,
							state: actionState,
							statePath: node._statePath ?? [],
						})
						stageActiveActions.push({
							action: node._action,
							state: actionState,
							statePath: node._statePath ?? [],
						})
					}
					isFixActionable = (node._action?._fixes ?? []).some((f) => {
						if (f._projectFn) {
							const { proxy, changes } = createTrackedPatch(projectV3.value as any)
							f._projectFn(proxy as any, actionState)
							return Object.keys(changes()).length > 0
						}
						if (f._versionFn) {
							const version = versions.value?.[0]
							if (!version) return true
							const { proxy, changes } = createTrackedPatch(version as any)
							f._versionFn(proxy as any, actionState)
							return Object.keys(changes()).length > 0
						}
						return false
					})
					if (isFixActionable) fixCount++
				}

				if (isRequired && !active) hasRequiredMissing = true
				map.set(node, {
					isActive: active,
					isVisible: node._shown === undefined || resolve(node._shown),
					isFixActionable,
					messageCount: active && node._action ? 1 : 0,
					fixCount: isFixActionable ? 1 : 0,
					hasRequiredMissing: isRequired && !active,
					activeActions: nodeActiveActions,
				})
			})
		}

		map.set(stage, {
			isActive: true,
			isVisible,
			isFixActionable: false,
			messageCount,
			fixCount,
			hasRequiredMissing,
			activeActions: stageActiveActions,
		})
	}

	return map
})

const restoredStage = persistedState
	? resolvedStages.value.findIndex((s) => s.id === persistedState.stage)
	: -1
const currentStage = ref(restoredStage >= 0 ? restoredStage : findFirstValidStage())

const router = useRouter()

let persistenceEnabled = true
let persistenceTimer: ReturnType<typeof setTimeout> | null = null

function cancelPendingPersistence() {
	if (persistenceTimer === null) return
	clearTimeout(persistenceTimer)
	persistenceTimer = null
}

function savePersistedState(open: boolean, resetReviewAnyway = false) {
	return saveChecklistState(checklistPersistenceProjectId, {
		open,
		reviewAnyway: resetReviewAnyway ? undefined : reviewedAnyway.value || undefined,
		stage: currentStageObj.value.id,
		message: message.value,
		state: toRaw(nodeStates.value),
	})
}

function persistState() {
	if (!persistenceEnabled || !import.meta.client) return
	cancelPendingPersistence()
	persistenceTimer = setTimeout(() => {
		persistenceTimer = null
		void savePersistedState(true)
	}, 150)
}

async function persistStateImmediately(open: boolean, resetReviewAnyway = false) {
	if (!import.meta.client) return
	cancelPendingPersistence()
	await savePersistedState(open, resetReviewAnyway)
}

watch(currentStage, persistState)
watch(nodeStates, persistState, { deep: true })
watch(message, persistState)

watch(
	nodeStates,
	async () => {
		const active = collectActiveActions()
		const newMap = new Map<ActionBuilder, boolean>()
		await Promise.all(
			active
				.filter((a) => a.action._message || a.action._autoMessage)
				.map(async ({ action, state, statePath }) => {
					try {
						const msg = action._autoMessage
							? await loadMd(
									`checklist/messages/${statePath.join('/')}`,
									state,
									projectV3.value,
									projectV2.value,
									action._autoMessageVars,
								)
							: await action._message!(state)
						newMap.set(action, !!(msg?.trim()))
					} catch {
						newMap.set(action, false)
					}
				}),
		)
		resolvedMessageAvailability.value = newMap
	},
	{ deep: true, immediate: true },
)

interface MessagePart {
	priority?: Priority
	content: string
}

function ignoreLegacyActionKeybind() {
	return undefined
}

function handleKeybinds(event: KeyboardEvent) {
	handleKeybind(
		event,
		{
			project: projectV2.value,
			state: {
				currentStage: currentStage.value,
				totalStages: resolvedStages.value.length,
				currentStageId: currentStageObj.value.id,
				currentStageTitle: currentStageObj.value.label,

				isCollapsed: props.collapsed,
				isDone: done.value,
				hasGeneratedMessage: generatedMessage.value,
				isLoadingMessage: loadingMessage.value,
				isModpackPermissionsStage: isModpackPermissionsStage.value,

				futureProjectCount: moderationQueue.queueLength,
				visibleActionsCount: resolveChildren(
					currentStageObj.value,
					nodeStates.value[currentStageObj.value.id!] ?? {},
				).length,

				focusedActionIndex: null,
				focusedActionType: null,
			},
			actions: {
				tryGoNext: nextStage,
				tryGoBack: previousStage,
				tryGenerateMessage: generateMessage,
				trySkipProject: skipCurrentProject,

				tryToggleCollapse: () => emit('toggleCollapsed'),
				tryResetProgress: resetProgress,
				tryExitModeration: handleExit,

				tryApprove: () => sendMessage(approveSendStatus.value),
				tryReject: () => sendMessage('rejected'),
				tryWithhold: () => sendMessage('withheld'),
				tryEditMessage: goBackToStages,

				tryCopyLink: async (permalink: boolean, relative: boolean, page: boolean) => {
					let url = ``
					if (relative) {
						url += `${globalThis.location.origin}`
					} else {
						url += `https://modrinth.com`
					}

					if (permalink) {
						url += `/project/${projectV2.value.id}`
					} else {
						url += `/${projectV2.value.project_type}/${projectV2.value.slug}`
					}

					if (page) {
						url += `/${globalThis.location.pathname.split('/').slice(3).join('/')}`
					}

					await navigator.clipboard.writeText(url)
				},

				tryCopyId: async () => await navigator.clipboard.writeText(projectV2.value.id),

				tryToggleAction: ignoreLegacyActionKeybind,
				trySelectDropdownOption: ignoreLegacyActionKeybind,
				tryToggleChip: ignoreLegacyActionKeybind,
				tryFocusNextAction: ignoreLegacyActionKeybind,
				tryFocusPreviousAction: ignoreLegacyActionKeybind,
				tryActivateFocusedAction: ignoreLegacyActionKeybind,
			},
		},
		Object.values(keybinds.value),
	)
}

// Trigger debounced prefetch when user progresses through stages
watch(currentStage, () => {
	// Only prefetch if we're past the first stage (user is actively moderating)
	if (currentStage.value > 0) {
		debouncedPrefetch() // Use debounced version to prevent spam
	}
})

onMounted(async () => {
	void persistStateImmediately(true)
	window.addEventListener('keydown', handleKeybinds)
	window.addEventListener('beforeunload', handleBeforeUnload)
	document.addEventListener('visibilitychange', handleVisibilityChange)
	notifications.setNotificationLocation('left')

	if (projectV2.value.status !== 'processing' && !reviewedAnyway.value) {
		alreadyReviewed.value = true
		return
	}

	const initialStage = resolvedStages.value[currentStage.value]
	if (initialStage?._navigate) {
		router.push(`/${projectV2.value.project_type}/${projectV2.value.slug}${initialStage._navigate}`)
	}

	const result = await moderationQueue.acquireLock(projectV2.value.id)

	if (result.success) {
		handleLockAcquired()
	} else if (result.locked_by) {
		lockStatus.value = {
			locked: true,
			lockedBy: result.locked_by,
			lockedAt: result.locked_at ? new Date(result.locked_at) : undefined,
			expiresAt: result.expires_at ? new Date(result.expires_at) : undefined,
			expired: result.expired,
			isOwnLock: false,
		}
		lockError.value = false

		// Start countdown timer
		updateLockCountdown()
		lockCountdownInterval.value = setInterval(updateLockCountdown, 1000)
	} else {
		handleLockUnavailable()
	}
})

function handleBeforeUnload() {
	const projectId = projectV2.value?.id
	if (!projectId || !lockStatus.value?.isOwnLock) return

	const config = useRuntimeConfig()
	const base = config.public.apiBaseUrl.replace(/\/v\d\/?$/, '/_internal/')
	const token = (auth as unknown as { value?: { token?: string } }).value?.token
	if (!token) return

	// sendBeacon is POST-only and cannot set Authorization. The internal POST /release endpoint
	// accepts the same token as text/plain (matches useBaseFetch's Authorization value).
	void navigator.sendBeacon(
		`${base}moderation/lock/${projectId}/release`,
		new Blob([token], { type: 'text/plain' }),
	)
}

onUnmounted(() => {
	cancelPendingPersistence()
	if (persistenceEnabled) {
		void savePersistedState(true)
	}
	window.removeEventListener('beforeunload', handleBeforeUnload)
	window.removeEventListener('keydown', handleKeybinds)
	document.removeEventListener('visibilitychange', handleVisibilityChange)
	notifications.setNotificationLocation('right')

	if (lockCheckInterval.value) {
		clearInterval(lockCheckInterval.value)
	}
	clearLockCountdown()

	// Release lock if we own it (navigation away without explicit exit)
	const projectId = projectV2.value?.id
	if (projectId && lockStatus.value?.isOwnLock) {
		void moderationQueue.releaseLock(projectId)
	}

	// Clear prefetch state to prevent memory leaks
	prefetchQueue.value = []
	isPrefetching.value = false
})

watch(
	currentStage,
	(newIndex, oldIndex) => {
		const stage = resolvedStages.value[newIndex]
		// only navigate when the stage actually changes (not on initial mount/remount)
		if (oldIndex !== undefined && newIndex !== oldIndex && stage?._navigate) {
			router.push(`/${projectV2.value.project_type}/${projectV2.value.slug}${stage._navigate}`)
		}
	},
	{ immediate: true },
)

watch(
	() => currentStageObj.value.id,
	(stageId) => {
		if (stageId === 'versions') {
			loadVersions()
		}
	},
	{ immediate: true },
)

function getModpackFilesFromStorage(): {
	interactive: ModerationModpackItem[]
	permanentNo: ModerationModpackItem[]
} {
	try {
		const sessionData = sessionStorage.getItem(`modpack-permissions-data-${projectV2.value.id}`)
		const interactive = sessionData ? (JSON.parse(sessionData) as ModerationModpackItem[]) : []

		const permanentNoData = sessionStorage.getItem(
			`modpack-permissions-permanent-no-${projectV2.value.id}`,
		)
		const permanentNo = permanentNoData
			? (JSON.parse(permanentNoData) as ModerationModpackItem[])
			: []

		return {
			interactive: interactive || [],
			permanentNo: permanentNo || [],
		}
	} catch (error) {
		console.warn('Failed to parse session storage modpack data:', error)
		return { interactive: [], permanentNo: [] }
	}
}

function countStageActions(stage: StageNodeBuilder): number {
	const actions = checklistLive.value.get(stage)?.activeActions ?? []
	const resolved = resolvedMessageAvailability.value
	return actions.filter((a) => {
		if (!a.action._message && !a.action._autoMessage) return false
		return resolved.get(a.action) ?? true
	}).length
}

function countStageFixes(stage: StageNodeBuilder): number {
	return checklistLive.value.get(stage)?.fixCount ?? 0
}

function hasRequiredMissing(stage: StageNodeBuilder): boolean {
	return checklistLive.value.get(stage)?.hasRequiredMissing ?? false
}

function collectActiveActions(): ActiveAction[] {
	return resolvedStages.value.flatMap((s) => checklistLive.value.get(s)?.activeActions ?? [])
}

async function assembleFullMessage() {
	const active = collectActiveActions()
	generatedActiveActions.value = active
	const parts = (
		await Promise.all(
			active
				.filter((a) => a.action._message || a.action._autoMessage)
				.map(async (a) => {
					const msg = a.action._autoMessage
						? await loadMd(
								`checklist/messages/${a.statePath.join('/')}`,
								a.state,
								projectV3.value,
								projectV2.value,
								a.action._autoMessageVars,
							)
						: await a.action._message!(a.state)
					return msg ? { priority: a.action._priority, content: msg } : null
				}),
		)
	).filter((part): part is MessagePart => part !== null)

	parts.sort((a, b) => {
		if (!a.priority && !b.priority) return 0
		if (!a.priority) return 1
		if (!b.priority) return -1
		return a.priority.compareTo(b.priority)
	})

	return expandVariables(
		parts
			.map((p) => p.content)
			.filter((c) => c.trim().length > 0)
			.join('\n\n'),
		projectV2.value,
		projectV3.value,
	)
}

provide(NODE_META_KEY, checklistLive)
provide(STATE_KEY, nodeStates)
provide(GLOBAL_STATE_KEY, nodeStates)

function shouldShowStage(stage: StageNodeBuilder): boolean {
	return checklistLive.value.get(stage)?.isVisible ?? false
}

function shouldShowStageIndex(stageIndex: number): boolean {
	return shouldShowStage(resolvedStages.value[stageIndex])
}

function previousStage() {
	let targetStage = currentStage.value - 1

	while (targetStage >= 0) {
		if (shouldShowStageIndex(targetStage)) {
			currentStage.value = targetStage
			return
		}
		targetStage--
	}
}

function nextStage() {
	if (isModpackPermissionsStage.value && !modpackPermissionsComplete.value) {
		addNotification({
			title: 'Modpack permissions stage unfinished',
			text: 'Please complete the modpack permissions stage before proceeding.',
			type: 'error',
		})

		return
	}

	let targetStage = currentStage.value + 1

	while (targetStage < resolvedStages.value.length) {
		if (shouldShowStageIndex(targetStage)) {
			currentStage.value = targetStage
			if (!isModpackPermissionsStage.value) {
				modpackPermissionsComplete.value = false
			}

			return
		}
		targetStage++
	}
}

function goBackToStages() {
	clearGeneratedMessageState()

	let targetStage = resolvedStages.value.length - 1
	while (targetStage >= 0) {
		if (shouldShowStageIndex(targetStage)) {
			currentStage.value = targetStage
			return
		}
		targetStage--
	}
}

async function generateMessage() {
	if (loadingMessage.value) return

	loadingMessage.value = true

	router.push(`/${projectV2.value.project_type}/${projectV2.value.slug}/moderation`)

	try {
		const baseMessage = await assembleFullMessage()
		let fullMessage = baseMessage

		if (projectV2.value.project_type === 'modpack') {
			const modpackFilesData = getModpackFilesFromStorage()

			if (modpackFilesData.interactive.length > 0 || modpackFilesData.permanentNo.length > 0) {
				const modpackMessage = generateModpackMessage(modpackFilesData)
				if (modpackMessage) {
					fullMessage = baseMessage ? `${baseMessage}\n\n${modpackMessage}` : modpackMessage
				}
			}
		}

		message.value = fullMessage
	} catch (error) {
		console.error('Error generating message:', error)
		addNotification({
			title: 'Error generating message',
			text: 'Failed to generate moderation message. Please try again.',
			type: 'error',
		})
	} finally {
		loadingMessage.value = false
	}
}

const finalPermissionMessages = {
	'with-attribution': `The following content has attribution requirements, meaning that you must link back to the page where you originally found this content in your Modpack's description or version changelog (e.g. linking a mod's CurseForge page if you got it from CurseForge):`,
	no: 'The following content is not allowed in Modrinth modpacks due to licensing restrictions. Please contact the author(s) directly for permission or remove the content from your modpack:',
	'permanent-no': `The following content is not allowed in Modrinth modpacks, regardless of permission obtained. This may be because it breaks Modrinth's content rules or because the authors, upon being contacted for permission, have declined. Please remove the content from your modpack:`,
	unidentified: `The following content could not be identified. Please provide proof of its origin along with proof that you have permission to include it:`,
}

function generateModpackMessage(allFiles: {
	interactive: ModerationModpackItem[]
	permanentNo: ModerationModpackItem[]
}) {
	const issues = []

	const attributeMods: string[] = []
	const noMods: string[] = []
	const permanentNoMods: string[] = []
	const unidentifiedMods: string[] = []

	allFiles.interactive.forEach((file) => {
		if (file.status === 'unidentified') {
			if (file.approved === 'no') {
				unidentifiedMods.push(file.file_name)
			}
		} else if (file.status === 'with-attribution' && file.approved === 'no') {
			attributeMods.push(file.file_name)
		} else if (file.status === 'no' && file.approved === 'no') {
			noMods.push(file.file_name)
		} else if (file.status === 'permanent-no') {
			permanentNoMods.push(file.file_name)
		}
	})

	allFiles.permanentNo.forEach((file) => {
		permanentNoMods.push(file.file_name)
	})

	if (
		attributeMods.length > 0 ||
		noMods.length > 0 ||
		permanentNoMods.length > 0 ||
		unidentifiedMods.length > 0
	) {
		issues.push('## Copyrighted content')

		if (unidentifiedMods.length > 0) {
			issues.push(
				`${finalPermissionMessages.unidentified}\n${unidentifiedMods.map((mod) => `- ${mod}`).join('\n')}`,
			)
		}

		if (attributeMods.length > 0) {
			issues.push(
				`${finalPermissionMessages['with-attribution']}\n${attributeMods.map((mod) => `- ${mod}`).join('\n')}`,
			)
		}

		if (noMods.length > 0) {
			issues.push(`${finalPermissionMessages.no}\n${noMods.map((mod) => `- ${mod}`).join('\n')}`)
		}

		if (permanentNoMods.length > 0) {
			issues.push(
				`${finalPermissionMessages['permanent-no']}\n${permanentNoMods.map((mod) => `- ${mod}`).join('\n')}`,
			)
		}
	}

	return issues.join('\n\n')
}

const hasNextProject = ref(false)
async function refreshModerationCaches(threadId?: string) {
	const refreshes: Promise<unknown>[] = [
		invalidate(),
		queryClient.invalidateQueries({ queryKey: ['moderation-projects'] }),
	]

	if (threadId) {
		refreshes.push(queryClient.invalidateQueries({ queryKey: ['thread', threadId] }))
	}

	await Promise.allSettled(refreshes)
}

async function sendMessage(status: ProjectStatus) {
	// Capture project data upfront to avoid null issues during async operations
	const projectId = projectV2.value?.id
	const threadId = projectV2.value?.thread_id
	const projectType = projectV2.value?.project_type

	if (!projectId) {
		addNotification({
			title: 'Error submitting moderation',
			text: 'Project data unavailable. Please try again.',
			type: 'error',
		})
		return
	}

	const active = generatedActiveActions.value ?? collectActiveActions()
	const shouldApplyFixes = active.some((a) => a.action._applyFixes)

	moderationDecision.value = status
	try {
		if (shouldApplyFixes) {
			const { proxy: projectProxy, changes: projectChanges } = createTrackedPatch(
				projectV3.value as any,
			)
			for (const { action, state } of active) {
				for (const f of action._fixes) {
					f._projectFn?.(projectProxy, state)
				}
			}
			const projectFixChanges = projectChanges()
			if (Object.keys(projectFixChanges).length > 0) {
				await client.labrinth.projects_v3.edit(projectId, projectFixChanges)
			}
		}

		await useBaseFetch(`project/${projectId}`, {
			method: 'PATCH',
			body: { status },
		})

		if (shouldApplyFixes && versions.value) {
			const versionFixes = active.flatMap(({ action, state }) =>
				action._fixes.filter((f) => f._versionFn).map((f) => ({ fix: f, state })),
			)
			if (versionFixes.length > 0) {
				await Promise.all(
					versions.value.map(async (version) => {
						const { proxy, changes } = createTrackedPatch(version as any)
						for (const { fix, state } of versionFixes) {
							fix._versionFn!(proxy, state)
						}
						const changed = changes()
						if (Object.keys(changed).length > 0) {
							await client.labrinth.versions_v3.modifyVersion(version.id, changed)
						}
					}),
				)
			}
		}

		if (message.value && threadId) {
			await useBaseFetch(`thread/${threadId}`, {
				method: 'POST',
				body: {
					body: {
						type: 'text',
						body: message.value,
					},
				},
			})
		}

		if (projectType === 'modpack' && Object.keys(modpackJudgements.value).length > 0) {
			await useBaseFetch(`moderation/project`, {
				internal: true,
				method: 'POST',
				body: modpackJudgements.value,
			})
		}

		await refreshModerationCaches(threadId)

		const willHaveNext = await moderationQueue.completeCurrentProject(projectId, 'completed')

		await Promise.race([
			moderationQueue.releaseLock(projectId),
			new Promise((r) => setTimeout(r, 2000)),
		])

		// Set both states together - hasNextProject MUST be set before done
		// to avoid the race condition where done=true renders with hasNextProject=false
		hasNextProject.value = willHaveNext
		done.value = true
		clearGeneratedMessageState()
	} catch (error) {
		console.error('Error submitting moderation:', error)
		addNotification({
			title: 'Error submitting moderation',
			text: 'Failed to submit moderation decision. Please try again.',
			type: 'error',
		})
	} finally {
		moderationDecision.value = null
	}
}

async function endChecklist(status?: string) {
	await clearProjectLocalStorage()

	if (!hasNextProject.value) {
		await navigateTo({
			name: 'moderation',
			state: {
				confetti: true,
			},
		})

		await nextTick()

		if (moderationQueue.currentQueue.total > 1) {
			addNotification({
				title: 'Moderation completed',
				text: `You have completed the moderation queue.`,
				type: 'success',
			})
		} else {
			addNotification({
				title: 'Moderation submitted',
				text: `Project ${status ?? 'completed successfully'}.`,
				type: 'success',
			})
		}
	} else {
		if (!(await navigateToNextUnlockedProject())) {
			const currentProjectId = projectV2.value?.id
			const remainingIds = moderationQueue.currentQueue.items.filter(
				(id) => id !== currentProjectId,
			)

			let foundEligible = false
			if (remainingIds.length > 0) {
				const next = await findNextEligibleQueueProject(remainingIds)

				if (next) {
					await Promise.all(
						next.skippedIds.map((id) => moderationQueue.completeCurrentProject(id, 'skipped')),
					)
					notifySkippedQueueProjects(next.skippedIds.length)
					navigateToQueueProject(next.result, next.projectId)
					foundEligible = true
				} else {
					await Promise.all(
						remainingIds.map((id) => moderationQueue.completeCurrentProject(id, 'skipped')),
					)
					addNotification({
						title: 'No projects available',
						text: 'All remaining projects are already moderated or locked by others.',
						type: 'warning',
					})
				}
			}

			if (!foundEligible) {
				await navigateTo({
					name: 'moderation',
				})
			}
		}
	}
}

async function skipCurrentProject() {
	const projectId = projectV2.value?.id
	if (!projectId) {
		addNotification({
			title: 'Error skipping project',
			text: 'Project data unavailable. Please try again.',
			type: 'error',
		})
		return
	}

	await Promise.race([
		moderationQueue.releaseLock(projectId),
		new Promise((r) => setTimeout(r, 2000)),
	])

	hasNextProject.value = await moderationQueue.completeCurrentProject(projectId, 'skipped')

	await endChecklist('skipped')
}

async function clearProjectLocalStorage() {
	persistenceEnabled = false
	cancelPendingPersistence()

	localStorage.removeItem(`modpack-permissions-${projectV2.value.id}`)
	localStorage.removeItem(`modpack-permissions-index-${projectV2.value.id}`)

	sessionStorage.removeItem(`modpack-permissions-data-${projectV2.value.id}`)
	sessionStorage.removeItem(`modpack-permissions-permanent-no-${projectV2.value.id}`)
	sessionStorage.removeItem(`modpack-permissions-updated-${projectV2.value.id}`)

	nodeStates.value = {}
	message.value = null
	await clearChecklistState(checklistPersistenceProjectId)
}

const isLastVisibleStage = computed(() => {
	for (let i = currentStage.value + 1; i < resolvedStages.value.length; i++) {
		if (shouldShowStageIndex(i)) {
			return false
		}
	}
	return true
})

const hasValidPreviousStage = computed(() => {
	for (let i = currentStage.value - 1; i >= 0; i--) {
		if (shouldShowStageIndex(i)) {
			return true
		}
	}
	return false
})

interface StageOption {
	id: string
	action: () => void
	text: string
	color?: 'green'
	icon?: Component
	messages?: number
	fixes?: number
	requiredMissing?: boolean
}

const stageOptions = computed<StageOption[]>(() => {
	const options = resolvedStages.value
		.map((stage, index) => {
			if (!shouldShowStage(stage)) return null

			return {
				id: String(index),
				action: () => {
					clearGeneratedMessageState()
					currentStage.value = index
				},
				text: stage.label ?? kebabToTitleCase(stage.id),
				color: index === currentStage.value && !generatedMessage.value ? 'green' : undefined,
				icon: stage._icon ?? undefined,
				messages: countStageActions(stage) || undefined,
				fixes: countStageFixes(stage) || undefined,
				requiredMissing: hasRequiredMissing(stage) || undefined,
			}
		})
		.filter((opt): opt is StageOption => opt !== null)

	options.push({
		id: 'generate-message',
		action: () => generateMessage(),
		text: 'Generate Message',
		color: generatedMessage.value ? 'green' : undefined,
		icon: CheckIcon,
	})

	return options
})
</script>

<style scoped lang="scss">
.moderation-checklist {
	@media (prefers-reduced-motion) {
		transition: none !important;
	}

	&.locked {
		animation: pulse-border 2s ease-in-out infinite;
	}

	@keyframes pulse-border {
		0%,
		100% {
			border-color: var(--color-orange);
		}
		50% {
			border-color: color-mix(in srgb, var(--color-orange) 40%, transparent);
		}
	}

	.button-actions-group,
	.toggle-actions-group,
	.dropdown-actions-group,
	.multi-select-actions-group {
		animation: fadeIn 0.2s ease-in-out;
	}

	@keyframes fadeIn {
		from {
			opacity: 0;
			transform: translateY(-5px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}
}

// Tooltip styling for button action message previews.
// Must use :global since floating-vue teleports tooltips outside the component DOM.
:global(.v-popper--theme-tooltip .v-popper__inner) {
	max-width: 400px;
	word-wrap: break-word;
	overflow-wrap: break-word;
	white-space: normal;
}

:global(.v-popper--theme-tooltip .moderation-tooltip-markdown) {
	line-height: 1.45;
	font-size: 0.9rem;
}

:global(.v-popper--theme-tooltip .moderation-tooltip-markdown p) {
	margin: 0.35rem 0;
}

:global(.v-popper--theme-tooltip .moderation-tooltip-markdown ul),
:global(.v-popper--theme-tooltip .moderation-tooltip-markdown ol) {
	margin: 0.35rem 0;
	padding-left: 1.15rem;
}

:global(.v-popper--theme-tooltip .moderation-tooltip-markdown pre) {
	max-width: 100%;
	overflow-x: auto;
	margin: 0.4rem 0;
}

:global(.v-popper--theme-tooltip .moderation-tooltip-markdown code) {
	background-color: rgba(255, 255, 255, 0.15);
	padding: 0.1rem 0.3rem;
	border-radius: 0.25rem;
	font-family: monospace;
	font-size: 0.85em;
}

:global(.v-popper--theme-tooltip .moderation-tooltip-markdown strong) {
	font-weight: 700;
}

:global(.v-popper--theme-tooltip .moderation-tooltip-markdown em) {
	font-style: italic;
}
</style>
