<template>
	<ConfirmModal
		v-if="isLockedByOther"
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
		:class="{
			'!w-fit': collapsed,
			locked: isLockedByOther,
			'right-4': settings.get(moderationSettings.General.ChecklistPosition) === 'right',
			'left-4': settings.get(moderationSettings.General.ChecklistPosition) === 'left',
		}"
	>
		<div class="flex grow-0 flex-col gap-1">
			<div class="flex items-center gap-2">
				<h1 class="m-0 mr-auto">
					<TeleportOverflowMenu
						v-if="canOpenStageSelectorFromTitle"
						:options="stageOptions"
						placement="center"
						btn-class="inline-flex items-center gap-2 bg-transparent p-0 text-2xl font-extrabold text-contrast"
					>
						<component
							:is="isPseudoStage ? ScaleIcon : (currentStageObj._icon ?? ScaleIcon)"
							class="text-orange"
						/>
						{{ checklistTitleText }}
						<template v-for="opt in stageOptions" #[opt.id] :key="opt.id">
							<component
								:is="opt.icon"
								v-if="opt.icon"
								class="mr-2"
								:class="{ 'opacity-50': opt.visited }"
							/>
							<span :class="{ 'opacity-50': opt.visited }">
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
						<component
							:is="isPseudoStage ? ScaleIcon : (currentStageObj._icon ?? ScaleIcon)"
							class="text-orange"
						/>
						{{ checklistTitleText }}
					</button>
				</h1>
				<ButtonStyled v-if="!isPseudoStage && currentStageObj._guidanceUrl" circular>
					<a v-tooltip="`Stage guidance`" target="_blank" :href="currentStageObj._guidanceUrl">
						<FileTextIcon />
					</a>
				</ButtonStyled>
				<ButtonStyled
					circular
					:color="!isPseudoStage && currentStageHasState ? 'orange' : 'red'"
					color-fill="none"
					hover-color-fill="background"
				>
					<button
						v-tooltip="
							!isPseudoStage && currentStageHasState
								? 'Reset Stage'
								: !isPseudoStage && !checklistHasState
									? 'Return to Start'
									: 'Reset Checklist'
						"
						:disabled="!isPseudoStage && !checklistHasState && isOnFirstStage"
						@click="resetProgress"
					>
						<UndoIcon v-if="!isPseudoStage && !checklistHasState" />
						<BrushCleaningIcon v-else />
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
			v-if="currentStageObj._hint && !collapsed && !isPseudoStage"
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

			<div v-if="isLockedByOther" class="flex flex-1 flex-col">
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
					<div v-else class="flex min-h-0 flex-1 flex-col">
						<NodeRenderer
							class="min-h-0 flex-1 overflow-y-auto p-1"
							:nodes="stageNodes"
							:state="stageState"
							:write="stageWriter"
							:on-image-upload="onUploadHandler"
							:app-components="appComponentsByKey"
							:global-state="nodeStates"
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
								<TeleportOverflowMenu :options="stageOptions" placement="center">
									<ListBulletedIcon />
									<span class="sr-only">Stages</span>
									<template v-for="opt in stageOptions" #[opt.id] :key="opt.id">
										<component
											:is="opt.icon"
											v-if="opt.icon"
											class="mr-2"
											:class="{ 'opacity-50': opt.visited }"
										/>
										<span :class="{ 'opacity-50': opt.visited }">
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
									<button :disabled="loadingModerationDecision" @click="previousStage">
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
								<ButtonStyled color="brand" :disabled="isLastVisibleStage && loadingMessage">
									<button @click="nextStage">
										<template v-if="isLastVisibleStage">
											<CheckIcon aria-hidden="true" />
											{{ loadingMessage ? 'Generating...' : 'Generate Message' }}
										</template>
										<template v-else> <RightArrowIcon aria-hidden="true" /> Next </template>
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
	LeftArrowIcon,
	LinkIcon,
	ListBulletedIcon,
	LockIcon,
	RightArrowIcon,
	ScaleIcon,
	SpinnerIcon,
	ToggleLeftIcon,
	ToggleRightIcon,
	UndoIcon,
	XIcon,
} from '@modrinth/assets'
import type { Priority } from '@modrinth/moderation'
import {
	expandVariables,
	kebabToTitleCase,
	moderationSettings,
	useStages,
} from '@modrinth/moderation'
import type { ActiveAction2, NodeState, StageNode } from '@modrinth/moderation/src/types/node'
import {
	CHECKLIST_META_KEY,
	collectActiveActions,
	collectMessageNodes,
	computeAttentionMap,
	computeNodeMeta,
	createTrackedPatch,
	evalActiveAction,
	evalSegment,
	resolve,
	resolveChildren,
	setMessageProject,
	setMissingMdHandler,
} from '@modrinth/moderation/src/types/node'
import NodeRenderer from '@modrinth/moderation/src/types/node/components/NodeRenderer.vue'
import type { FixBuilder } from '@modrinth/moderation/src/types/node/fix'
import type { Writer } from '@modrinth/moderation/src/types/node/mutate'
import LoaderPicker from '~/components/ui/create-project-version/components/LoaderPicker.vue'
import McVersionPicker from '~/components/ui/create-project-version/components/McVersionPicker.vue'
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
import { renderHighlightedString } from '@modrinth/utils'
import type { ProjectStatus } from '@modrinth/utils'
import { useQueryClient } from '@tanstack/vue-query'
import { useDebounceFn } from '@vueuse/core'
import type { Component } from 'vue'
import { computed, nextTick, provide, ref, toRaw, watch, watchEffect } from 'vue'

import { useGeneratedState } from '~/composables/generated'
import { useImageUpload } from '~/composables/image-upload.ts'
import { getProjectTypeForUrlShorthand } from '~/helpers/projects.js'
import {
	getSessionChecklistState,
	patchSessionChecklistState,
} from '~/services/moderation-checklist-session-storage.ts'
import {
	clearChecklistState,
	loadChecklistState,
	saveChecklistState,
} from '~/services/moderation-checklist-storage.ts'
import type { LockAcquireResponse } from '~/services/moderation-queue.ts'
import { useModerationQueue } from '~/services/moderation-queue.ts'

import { type LiveNode, STATE_KEY } from './checklist-context'

const notifications = injectNotificationManager()
const { addNotification } = notifications
const debug = useDebugLogger('ModerationChecklist')
const keybinds = useModerationKeybinds()
const settings = useModerationSettings()

const takeOverModal = ref<InstanceType<typeof ConfirmModal>>()

const props = defineProps<{
	collapsed: boolean
}>()

const { projectV2, projectV3, versions, loadVersions, invalidate, thread } =
	injectProjectPageContext()
setMessageProject(projectV3, projectV2)
const missingMdPaths = new Set<string>()
setMissingMdHandler((path) => missingMdPaths.add(path))

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

	prefetchQueue.value.shift()

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
const activatedStages = ref<Set<string>>(new Set(persistedState?.activatedStages ?? []))
const visitedStages = ref<Set<string>>(
	new Set(
		import.meta.client
			? (getSessionChecklistState(checklistPersistenceProjectId).visitedStages ?? [])
			: [],
	),
)

function markStageVisited(stageId: string | undefined) {
	if (!stageId || visitedStages.value.has(stageId)) return
	visitedStages.value.add(stageId)
	patchSessionChecklistState(checklistPersistenceProjectId, {
		visitedStages: [...visitedStages.value],
	})
}
const reviewedAnyway = ref(persistedState?.reviewAnyway ?? false)
const message = ref<string | null>(persistedState?.message ?? null)
const generatedActiveActions = ref<ActiveAction2[] | null>(null)
const resolvedMessageAvailability = ref<Map<object, boolean>>(new Map())
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
	maintainPrefetchQueue()
}

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

	const projects = await client.labrinth.projects_v3.getMultiple(projectIds).catch(() => [])
	const projectsById = new Map(projects.map((project) => [project.id, project]))

	const checks = await Promise.allSettled(
		projectIds.map(async (id) => {
			const lockResponse = await moderationQueue.checkLock(id)
			const project = projectsById.get(id) ?? null

			return {
				id,
				locked: lockResponse.locked,
				expired: lockResponse.expired,
				isOwnLock: lockResponse.is_own_lock,
				slug: project?.slug,
				projectType: project?.project_types[0],
				status: project?.status,
				isProcessing: project === null ? true : project.status === 'processing',
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

async function maintainPrefetchQueue() {
	if (isPrefetching.value) return
	if (!moderationQueue.isQueueMode) return

	const currentProjectId = projectV2.value?.id

	isPrefetching.value = true

	try {
		const now = Date.now()
		prefetchQueue.value = prefetchQueue.value.filter((p) => now - p.validatedAt < PREFETCH_STALE_MS)

		if (currentProjectId) {
			prefetchQueue.value = prefetchQueue.value.filter((p) => p.projectId !== currentProjectId)
		}

		if (prefetchQueue.value.length >= PREFETCH_TARGET_COUNT) {
			return
		}

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

const debouncedPrefetch = useDebounceFn(maintainPrefetchQueue, 300)

async function skipToNextProject() {
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

const currentStageHasState = computed(() => {
	const stageId = currentStageObj.value.id
	if (!stageId) return false
	const stageState = nodeStates.value[stageId]
	return !!stageState && Object.keys(stageState).length > 0
})

const checklistHasState = computed(() =>
	Object.values(nodeStates.value).some((s) => s && Object.keys(s).length > 0),
)

function resetProgress() {
	if (!isPseudoStage.value && currentStageHasState.value) {
		Reflect.deleteProperty(nodeStates.value, currentStageObj.value.id!)
		activatedStages.value.delete(currentStageObj.value.id!)
		clearGeneratedMessageState()
		return
	}

	currentStage.value = findFirstValidStage()
	nodeStates.value = {}
	activatedStages.value = new Set()

	done.value = false
	clearGeneratedMessageState()
	loadingMessage.value = false
	moderationDecision.value = null
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
const isOnFirstStage = computed(() => currentStage.value === findFirstValidStage())
const isLockedByOther = computed(() => lockStatus.value?.locked && !lockStatus.value?.isOwnLock)
const isPseudoStage = computed(
	() => alreadyReviewed.value || done.value || generatedMessage.value || isLockedByOther.value,
)
const canOpenStageSelectorFromTitle = computed(
	() => !alreadyReviewed.value && !done.value && !isLockedByOther.value,
)

const checklistTitleText = computed(() => {
	if (alreadyReviewed.value || done.value) return 'Moderation'
	if (generatedMessage.value) return 'Generated Message'

	return currentStageObj.value.label ?? kebabToTitleCase(currentStageObj.value.id)
})
function isStageLive(stage: StageNode): boolean {
	return stage._shown === undefined || resolve(stage._shown)
}

function isStageEffectivelyShown(stage: StageNode): boolean {
	if (isStageLive(stage)) return true
	return stage._shownSticky === true && activatedStages.value.has(stage.id)
}

watchEffect(() => {
	for (const stage of resolvedStages.value) {
		if (stage._shownSticky && isStageLive(stage) && !activatedStages.value.has(stage.id)) {
			activatedStages.value.add(stage.id)
		}
	}
})

function isFixActionable(fixes: FixBuilder[], state: Record<string, NodeState>): boolean {
	return fixes.some((f) => {
		if (f._projectFn) {
			const { proxy, changes } = createTrackedPatch(projectV3.value as any)
			f._projectFn(proxy as any, state)
			return Object.keys(changes()).length > 0
		}
		if (f._versionFn) {
			const version = versions.value?.[0]
			if (!version) return true
			const { proxy, changes } = createTrackedPatch(version as any)
			f._versionFn(proxy as any, state)
			return Object.keys(changes()).length > 0
		}
		return false
	})
}

function computeStageLiveNode(stage: StageNode, stageState: Record<string, NodeState>): LiveNode {
	if (!isStageEffectivelyShown(stage)) {
		return {
			isActive: true,
			isVisible: false,
			isFixActionable: false,
			messageCount: 0,
			fixCount: 0,
			hasRequiredMissing: false,
			activeActions: [],
		}
	}

	const stageChildren = resolveChildren(stage, stageState)
	const metaMap = computeNodeMeta(stageChildren, stageState, isFixActionable)
	const attentionMap = computeAttentionMap(stageChildren, stageState, metaMap)
	const actions = collectActiveActions(stageChildren, stageState, [stage.id])

	if (stage._segments.length > 0) {
		actions.unshift({ node: stage, state: stageState, statePath: [stage.id], active: true })
	}

	return {
		isActive: true,
		isVisible: metaMap.size > 0,
		isFixActionable: false,
		messageCount: actions.length,
		fixCount: [...metaMap.values()].filter((m) => m.isFixActionable).length,
		hasRequiredMissing: stageChildren.some(
			(child) => typeof child === 'object' && child !== null && attentionMap.get(child) === true,
		),
		activeActions: actions,
	}
}

const checklistLive = computed<Map<object, LiveNode>>(() => {
	const map = new Map<object, LiveNode>()

	for (const stage of resolvedStages.value) {
		const stageState = (nodeStates.value[stage.id] ?? {}) as Record<string, NodeState>
		map.set(stage, computeStageLiveNode(stage, stageState))
	}

	return map
})

const restoredStage = persistedState
	? resolvedStages.value.findIndex((s) => s.id === persistedState.stage)
	: -1
const currentStage = ref(restoredStage >= 0 ? restoredStage : findFirstValidStage())
const initialAutoStage = currentStage.value
const needsInitialStageSettle = !persistedState && thread.value === undefined

// Thread data may not be loaded when currentStage is first set, so stages that depend on it
// (like re-review) may be invisible initially. Re-evaluate once thread loads.
if (!persistedState) {
	watch(
		thread,
		() => {
			if (thread.value === undefined) return
			if (currentStage.value === initialAutoStage) {
				const firstValid = findFirstValidStage()
				if (firstValid !== currentStage.value) {
					currentStage.value = firstValid
				} else if (needsInitialStageSettle) {
					markStageVisited(currentStageObj.value.id)
				}
			}
		},
		{ once: true },
	)
}

const router = useRouter()

let persistenceEnabled = true
let persistenceTimer: ReturnType<typeof setTimeout> | null = null

function cancelPendingPersistence() {
	if (persistenceTimer === null) return
	clearTimeout(persistenceTimer)
	persistenceTimer = null
}

function savePersistedState(open: boolean, resetReviewAnyway = false) {
	const rawState = toRaw(nodeStates.value)
	const openVal = open || undefined
	const reviewAnywayVal = resetReviewAnyway ? undefined : reviewedAnyway.value || undefined
	const stageVal =
		currentStage.value !== findFirstValidStage() ? currentStageObj.value.id : undefined
	const messageVal = message.value ?? undefined
	const stateVal = Object.keys(rawState).length > 0 ? rawState : undefined
	const activatedStagesVal = activatedStages.value.size > 0 ? [...activatedStages.value] : undefined
	if (
		!openVal &&
		!reviewAnywayVal &&
		!stageVal &&
		!messageVal &&
		!stateVal &&
		!activatedStagesVal
	) {
		return clearChecklistState(checklistPersistenceProjectId)
	}
	return saveChecklistState(checklistPersistenceProjectId, {
		...(openVal && { open: openVal }),
		...(reviewAnywayVal && { reviewAnyway: reviewAnywayVal }),
		...(stageVal && { stage: stageVal }),
		...(messageVal && { message: messageVal }),
		...(stateVal && { state: stateVal }),
		...(activatedStagesVal && { activatedStages: activatedStagesVal }),
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
watch(activatedStages, persistState, { deep: true })
watch(message, persistState)
watch(currentStageObj, (stage) => markStageVisited(stage.id), {
	immediate: !needsInitialStageSettle,
})

watch(
	nodeStates,
	async () => {
		const active = collectAllActiveActions()
		const newMap = new Map<object, boolean>()
		await Promise.all(
			active
				.filter((a) => (a.node as any)._segments.some((s: any) => s.type !== 'collect'))
				.map(async ({ node, state, statePath }) => {
					try {
						let hasContent = false
						for (const seg of (node as any)._segments) {
							if (seg.type === 'collect') continue
							const text = await evalSegment(seg, state, statePath)
							if (text?.trim()) {
								hasContent = true
								break
							}
						}
						newMap.set(node, hasContent)
					} catch {
						newMap.set(node, false)
					}
				}),
		)
		resolvedMessageAvailability.value = newMap
	},
	{ deep: true, immediate: true },
)

function handleKeybinds(event: KeyboardEvent) {
	keybinds.value.handle(event, {
		project: projectV2.value,
		scope: 'checklist',
		state: {
			currentStage: currentStage.value,
			totalStages: resolvedStages.value.length,
			currentStageId: currentStageObj.value.id,
			currentStageTitle: currentStageObj.value.label,

			isCollapsed: props.collapsed,
			isDone: done.value,
			hasGeneratedMessage: generatedMessage.value,
			isLoadingMessage: loadingMessage.value,
			isModpackPermissionsStage: false,

			futureProjectCount: moderationQueue.queueLength,
			visibleActionsCount: resolveChildren(
				currentStageObj.value,
				nodeStates.value[currentStageObj.value.id] ?? {},
			).filter((c) => typeof c === 'object' && c !== null).length,
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
			tryEditMessage: previousStage,
		},
	})
}

watch(currentStage, () => {
	// Only prefetch if we're past the first stage (user is actively moderating)
	if (currentStage.value > 0) {
		debouncedPrefetch()
	}
})

onMounted(async () => {
	void persistStateImmediately(true)
	window.addEventListener('keydown', handleKeybinds)
	window.addEventListener('beforeunload', handleBeforeUnload)
	document.addEventListener('visibilitychange', handleVisibilityChange)
	if (settings.value.get(moderationSettings.General.ChecklistPosition) === 'right') {
		notifications.setNotificationLocation('left')
	}

	if (done.value || alreadyReviewed.value) return

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

loadVersions()

function countStageActions(stage: StageNode): number {
	const actions = checklistLive.value.get(stage)?.activeActions ?? []
	const resolved = resolvedMessageAvailability.value
	return actions.filter((a) => {
		if ((a.node as any)._segments.every((s: any) => s.type === 'collect')) return false
		return resolved.get(a.node) ?? true
	}).length
}

function countStageFixes(stage: StageNode): number {
	return checklistLive.value.get(stage)?.fixCount ?? 0
}

function hasRequiredMissing(stage: StageNode): boolean {
	return checklistLive.value.get(stage)?.hasRequiredMissing ?? false
}

function collectAllActiveActions(): ActiveAction2[] {
	return resolvedStages.value.flatMap((s) => checklistLive.value.get(s)?.activeActions ?? [])
}

function byPriority(a: ActiveAction2, b: ActiveAction2): number {
	return ((a.node as any)._priority as Priority).compareTo((b.node as any)._priority as Priority)
}

async function assembleFullMessage() {
	const allEntries = collectAllActiveActions()
	generatedActiveActions.value = allEntries

	const consumed = new Set<object>()

	const parts: { entry: ActiveAction2; content: string }[] = []
	for (const entry of allEntries) {
		if (consumed.has(entry.node)) continue
		const content = await evalActiveAction(entry, allEntries, consumed)
		if (content.trim()) {
			parts.push({ entry, content })
		}
	}

	parts.sort((a, b) => byPriority(a.entry, b.entry))

	return expandVariables(
		parts
			.map((p) => p.content.trim())
			.filter((c) => c.length > 0)
			.join('\n\n'),
		projectV2.value,
		projectV3.value,
	)
}

const tooltipHtmlMap = ref(new Map<object, string>())

watchEffect(async () => {
	const stage = currentStageObj.value
	const stageState = (nodeStates.value[stage.id] ?? {}) as Record<string, NodeState>
	const stageChildren = resolveChildren(stage, stageState)
	const actions = collectMessageNodes(stageChildren, stageState, [stage.id])

	const newMap = new Map<object, string>()
	await Promise.all(
		actions.map(async (entry) => {
			try {
				const raw = await evalActiveAction(entry, actions, new Set())
				const expanded = expandVariables(raw, projectV2.value, projectV3.value).trim()
				newMap.set(
					entry.node,
					expanded
						? `<div class="markdown-body moderation-tooltip-markdown">${renderHighlightedString(expanded)}</div>`
						: '',
				)
			} catch {
				newMap.set(entry.node, '')
			}
		}),
	)
	tooltipHtmlMap.value = newMap
})

const stageMeta = computed(() => {
	const stage = currentStageObj.value
	const stageState = (nodeStates.value[stage.id] ?? {}) as Record<string, NodeState>
	const stageChildren = resolveChildren(stage, stageState)
	const metaMap = computeNodeMeta(stageChildren, stageState, isFixActionable)
	const attentionMap = computeAttentionMap(stageChildren, stageState, metaMap)
	return { metaMap, attentionMap, tooltipHtml: tooltipHtmlMap.value }
})

const appComponentsByKey: Record<string, Component> = {
	'loader-picker': LoaderPicker,
	'game-version-picker': McVersionPicker,
}

const stageState = computed(
	() => (nodeStates.value[currentStageObj.value.id] ?? {}) as Record<string, NodeState>,
)
const stageNodes = computed(() => resolveChildren(currentStageObj.value, stageState.value))

const stageWriter: Writer = (id, value) => {
	const stageId = currentStageObj.value.id
	const existing = nodeStates.value[stageId]
	const next: Record<string, NodeState> = existing ? { ...existing } : {}
	if (value === undefined) delete next[id]
	else next[id] = value
	if (Object.keys(next).length === 0) {
		if (existing !== undefined) delete nodeStates.value[stageId]
	} else {
		nodeStates.value[stageId] = next
	}
}

provide(CHECKLIST_META_KEY, stageMeta)
provide(STATE_KEY, nodeStates)

function shouldShowStage(stage: StageNode): boolean {
	return checklistLive.value.get(stage)?.isVisible ?? false
}

function shouldShowStageIndex(stageIndex: number): boolean {
	return shouldShowStage(resolvedStages.value[stageIndex])
}

function previousStage() {
	if (generatedMessage.value) {
		goBackToStages()
		return
	}

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
	if (done.value) {
		endChecklist(undefined)
		return
	}

	if (alreadyReviewed.value || isLockedByOther.value) {
		if (moderationQueue.isQueueMode && moderationQueue.queueLength > 1) skipToNextProject()
		return
	}

	if (generatedMessage.value) return

	let targetStage = currentStage.value + 1
	while (targetStage < resolvedStages.value.length) {
		if (shouldShowStageIndex(targetStage)) {
			currentStage.value = targetStage
			return
		}
		targetStage++
	}

	generateMessage()
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
		missingMdPaths.clear()
		const baseMessage = await assembleFullMessage()
		if (missingMdPaths.size > 0) {
			addNotification({
				title: 'Missing message files',
				text: [...missingMdPaths].join('\n'),
				type: 'warning',
			})
		}
		message.value = baseMessage
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

const hasNextProject = ref(false)

const finishedId = localStorage.getItem('moderation-checklist-finished')
if (finishedId === projectV2.value.id) {
	localStorage.removeItem('moderation-checklist-finished')
	hasNextProject.value = moderationQueue.queueLength > 0
	done.value = true
} else if (projectV2.value.status !== 'processing' && !reviewedAnyway.value) {
	alreadyReviewed.value = true
} else {
	const initialStage = resolvedStages.value[currentStage.value]
	if (initialStage?._navigate) {
		navigateTo(`/${projectV2.value.project_type}/${projectV2.value.slug}${initialStage._navigate}`)
	}
}

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
	const projectId = projectV2.value?.id
	const threadId = projectV2.value?.thread_id

	if (!projectId) {
		addNotification({
			title: 'Error submitting moderation',
			text: 'Project data unavailable. Please try again.',
			type: 'error',
		})
		return
	}

	const active = [...(generatedActiveActions.value ?? collectAllActiveActions())].sort(byPriority)
	const shouldApplyFixes = active.some((a) => (a.node as any)._applyFixes)

	moderationDecision.value = status
	try {
		await useBaseFetch(`project/${projectId}`, {
			method: 'PATCH',
			body: { status },
		})

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

		let projectFixChanges: Partial<typeof projectV3.value> = {}

		if (shouldApplyFixes) {
			const { proxy: projectProxy, changes: projectChanges } = createTrackedPatch(
				projectV3.value as any,
			)
			for (const { node, state } of active) {
				if (!('_fixes' in (node as object))) continue
				for (const f of (node as any)._fixes) {
					f._projectFn?.(projectProxy, state)
				}
			}
			projectFixChanges = projectChanges()
			if (Object.keys(projectFixChanges).length > 0) {
				await client.labrinth.projects_v3.edit(projectId, projectFixChanges)
			}

			if (versions.value) {
				const versionFixes = active.flatMap(({ node, state }) =>
					'_fixes' in (node as object)
						? (node as any)._fixes
								.filter((f: FixBuilder) => f._versionFn)
								.map((f: FixBuilder) => ({ fix: f, state }))
						: [],
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
		}

		await refreshModerationCaches(threadId)

		const willHaveNext = await moderationQueue.completeCurrentProject(projectId, 'completed')

		await Promise.race([
			moderationQueue.releaseLock(projectId),
			new Promise((r) => setTimeout(r, 2000)),
		])

		if (projectFixChanges?.slug) {
			const urlType = getProjectTypeForUrlShorthand(projectV2.value.project_type, [], tags.value)
			localStorage.setItem('moderation-checklist-finished', projectId)
			clearGeneratedMessageState()
			await navigateTo(`/${urlType}/${projectFixChanges.slug}/moderation`, { replace: true })
			return
		}

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

	nodeStates.value = {}
	activatedStages.value = new Set()
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
	visited?: boolean
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
				visited:
					(index !== currentStage.value && stage.id && visitedStages.value.has(stage.id)) ||
					undefined,
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
	position: fixed;
	bottom: 1rem;
	overflow-y: auto;
	z-index: 50;
	transition: bottom 0.25s ease-in-out;

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
