<template>
	<ConfirmModal
		v-if="isLockedByOther"
		ref="takeOverModal"
		title="Override moderation lock"
		description="Are you sure you want to take over moderation of this project?"
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
						:label="checklistTitleText"
						:options="stageOptions"
						:icon-only="false"
						type="quiet"
						size="xl"
						placement="bottom-center"
						class="!h-auto !w-auto !gap-2 !bg-transparent !p-0 !text-2xl !font-extrabold !text-contrast [&>svg]:!text-orange"
					>
						<component
							:is="isPseudoStage ? ScaleIcon : (currentStageObj._icon ?? ScaleIcon)"
							class="text-orange"
							aria-hidden="true"
						/>
						{{ checklistTitleText }}
						<template v-for="opt in stageOptions" #[opt.id] :key="opt.id">
							<component
								:is="opt.icon"
								v-if="opt.icon"
								class="mr-2"
								:class="{ 'opacity-50': opt.visited }"
								aria-hidden="true"
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
					<span
						v-else
						class="inline-flex cursor-default items-center gap-2 bg-transparent p-0 text-2xl font-extrabold text-contrast"
					>
						<component
							:is="isPseudoStage ? ScaleIcon : (currentStageObj._icon ?? ScaleIcon)"
							class="text-orange"
							aria-hidden="true"
						/>
						{{ checklistTitleText }}
					</span>
				</h1>
				<IconButton
					v-if="!isPseudoStage && stageNavigateTarget"
					v-tooltip="stageNavigateButtonLabel"
					type="quiet"
					:label="stageNavigateButtonLabel"
					class="!bg-button-bg !text-primary ![box-shadow:var(--shadow-button)]"
					:disabled="route.path === stageNavigateTarget"
					@click="navigateToStagePage"
				>
					<MapPinIcon aria-hidden="true" />
				</IconButton>
				<ButtonLink
					v-if="!isPseudoStage && currentStageObj._guidanceUrl"
					v-tooltip="`Stage guidance`"
					type="quiet"
					target="_blank"
					:href="currentStageObj._guidanceUrl"
					class="!w-9 !rounded-full !bg-button-bg !px-0 !text-primary ![box-shadow:var(--shadow-button)]"
				>
					<FileTextIcon aria-hidden="true" />
					<span class="sr-only">Stage guidance</span>
				</ButtonLink>
				<IconButton
					v-tooltip="resetLabel"
					type="quiet"
					interaction="filled"
					:color="!isPseudoStage && currentStageHasState ? 'orange' : 'red'"
					:label="resetLabel"
					class="!bg-button-bg !text-primary ![box-shadow:var(--shadow-button)]"
					:disabled="!isPseudoStage && !checklistHasState && isOnFirstStage"
					@click="resetProgress"
				>
					<UndoIcon v-if="!isPseudoStage && !checklistHasState" aria-hidden="true" />
					<BrushCleaningIcon v-else aria-hidden="true" />
				</IconButton>
				<IconButton
					v-tooltip="`Exit moderation`"
					type="quiet"
					interaction="filled"
					color="red"
					label="Exit moderation"
					class="!bg-button-bg !text-primary ![box-shadow:var(--shadow-button)]"
					@click="handleExit"
				>
					<XIcon aria-hidden="true" />
				</IconButton>
				<IconButton
					v-tooltip="collapseLabel"
					type="quiet"
					:label="collapseLabel"
					class="!bg-button-bg !text-primary ![box-shadow:var(--shadow-button)]"
					@click="emit('toggleCollapsed')"
				>
					<DropdownIcon
						class="transition-transform"
						:class="{ 'rotate-180': collapsed }"
						aria-hidden="true"
					/>
				</IconButton>
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
					<LockIcon class="size-8 text-orange" aria-hidden="true" />
					<span class="text-secondary">{{ lockDescription }}</span>
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
							<Button @click="openTakeOverModal">
								<LockIcon aria-hidden="true" />
								Take over
							</Button>
						</div>
						<div class="flex items-center gap-2">
							<Button
								v-if="moderationQueue.isQueueMode && moderationQueue.queueLength > 1"
								type="colored"
								color="brand"
								@click="skipToNextProject"
							>
								<RightArrowIcon aria-hidden="true" />
								Next project ({{ moderationQueue.queueLength }} left)
							</Button>
						</div>
					</div>
				</div>
			</div>

			<div v-else-if="alreadyReviewed" class="flex flex-1 flex-col">
				<div class="flex flex-1 flex-col items-center justify-center gap-4 py-8 text-center">
					<CheckIcon class="size-8 text-green" aria-hidden="true" />
					<span class="text-secondary">This project was already moderated.</span>
				</div>
				<div class="mt-auto">
					<div
						class="mt-4 flex grow justify-between gap-2 border-0 border-t-[1px] border-solid border-surface-5 pt-4"
					>
						<div class="flex items-center gap-2">
							<Button @click="reviewAnyway">
								<ScaleIcon aria-hidden="true" />
								Review anyway
							</Button>
						</div>
						<div class="flex items-center gap-2">
							<Button
								v-if="moderationQueue.isQueueMode && moderationQueue.queueLength > 1"
								type="colored"
								color="brand"
								@click="skipToNextProject"
							>
								<RightArrowIcon aria-hidden="true" />
								Next project ({{ moderationQueue.queueLength }} left)
							</Button>
						</div>
					</div>
				</div>
			</div>

			<template v-else>
				<div class="flex min-h-0 flex-1 flex-col">
					<div v-if="done">
						<p>{{ completionMessage }}</p>
					</div>
					<div v-else-if="generatedMessage" class="flex min-h-0 flex-1 flex-col gap-2">
						<Button class="shrink-0 self-start" @click="useSimpleEditor = !useSimpleEditor">
							<template v-if="!useSimpleEditor">
								<ToggleLeftIcon aria-hidden="true" />
								Use simple mode
							</template>
							<template v-else>
								<ToggleRightIcon aria-hidden="true" />
								Use advanced mode
							</template>
						</Button>
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
							:meta="stageMeta"
							:on-image-upload="onUploadHandler"
							:global-state="nodeStates"
						/>
					</div>
				</div>

				<div class="mt-auto">
					<div
						class="mt-4 flex grow justify-between gap-2 border-0 border-t-[1px] border-solid border-surface-5 pt-4"
					>
						<div class="flex items-center gap-2">
							<Button
								v-if="!done && !generatedMessage && moderationQueue.hasItems"
								@click="skipCurrentProject"
							>
								<XIcon aria-hidden="true" />
								Skip ({{ moderationQueue.queueLength }} left)
							</Button>
						</div>

						<div class="flex items-center gap-2">
							<TeleportOverflowMenu
								v-if="!done"
								label="More options"
								:options="stageOptions"
								placement="bottom-end"
							>
								<ListBulletedIcon aria-hidden="true" />
								<span class="sr-only">Stages</span>
								<template v-for="opt in stageOptions" #[opt.id] :key="opt.id">
									<component
										:is="opt.icon"
										v-if="opt.icon"
									class="mr-2"
									:class="{ 'opacity-50': opt.visited }"
									aria-hidden="true"
									/>
									<span :class="{ 'opacity-50': opt.visited }">
										{{ opt.text
										}}<span v-if="opt.requiredMissing" class="font-bold text-red">*</span>
									</span>
									<span v-if="opt.messages" class="ml-auto pl-2 font-semibold opacity-75">{{
										opt.messages
									}}</span>
									<span v-if="opt.fixes" class="pl-2 font-semibold text-blue">{{ opt.fixes }}</span>
								</template>
							</TeleportOverflowMenu>

							<div v-if="done">
								<Button type="colored" color="brand" @click="endChecklist(undefined)">
									<template v-if="hasNextProject">
										<RightArrowIcon aria-hidden="true" />
										Next project ({{ moderationQueue.queueLength }} left)
									</template>
									<template v-else>
										<CheckIcon aria-hidden="true" />
										All done!
									</template>
								</Button>
							</div>

							<div v-else-if="generatedMessage" class="flex items-center gap-2">
								<Button :disabled="loadingModerationDecision" @click="previousStage">
									<LeftArrowIcon aria-hidden="true" />
									Edit
								</Button>
								<Button
									type="colored"
									color="red"
									:disabled="loadingModerationDecision"
									:loading="moderationDecision === 'rejected'"
									@click="sendMessage('rejected')"
								>
									<XIcon aria-hidden="true" />
									Reject
								</Button>
								<Button
									type="colored"
									color="orange"
									:disabled="loadingModerationDecision"
									:loading="moderationDecision === 'withheld'"
									@click="sendMessage('withheld')"
								>
									<LinkIcon aria-hidden="true" />
									Withhold
								</Button>
								<Button
									type="colored"
									color="green"
									:disabled="loadingModerationDecision"
									:loading="moderationDecision === approveSendStatus"
									@click="sendMessage(approveSendStatus)"
								>
									<CheckIcon aria-hidden="true" />
									Approve
								</Button>
							</div>

							<div v-else class="flex items-center gap-2">
								<Button :disabled="!hasValidPreviousStage" @click="previousStage">
									<LeftArrowIcon aria-hidden="true" />
									Previous
								</Button>
								<Button
									type="colored"
									color="brand"
									:disabled="isLastVisibleStage && loadingMessage"
									:loading="isLastVisibleStage && loadingMessage"
									@click="nextStage"
								>
									<template v-if="isLastVisibleStage">
										<CheckIcon aria-hidden="true" />
										Generate message
									</template>
									<template v-else>
										<RightArrowIcon aria-hidden="true" />
										Next
									</template>
								</Button>
							</div>
						</div>
					</div>
				</div>
			</template>
		</Collapsible>
	</div>
</template>

<script lang="ts" setup>
import type { Labrinth } from '@modrinth/api-client'
import {
	BrushCleaningIcon,
	CheckIcon,
	DropdownIcon,
	FileTextIcon,
	LeftArrowIcon,
	LinkIcon,
	ListBulletedIcon,
	LockIcon,
	MapPinIcon,
	RightArrowIcon,
	ScaleIcon,
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
import type {
	ActiveAction,
	MessageSegment,
	NodeState,
	StageNode,
} from '@modrinth/moderation/src/types/node'
import {
	collectActiveActions,
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
import type { FixBuilder } from '@modrinth/moderation/src/types/node/fix'
import {
	Avatar,
	Button,
	ButtonLink,
	Collapsible,
	ConfirmModal,
	IconButton,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	MarkdownEditor,
	StyledInput,
	TeleportOverflowMenu,
	useDebugLogger,
} from '@modrinth/ui'
import type { ProjectStatus } from '@modrinth/utils'
import { useQueryClient } from '@tanstack/vue-query'
import { useDebounceFn } from '@vueuse/core'
import { computed, nextTick, ref, watch, watchEffect } from 'vue'

import { useGeneratedState } from '~/composables/generated'
import { useImageUpload } from '~/composables/image-upload.ts'
import { getProjectTypeForUrlShorthand } from '~/helpers/projects.js'
import { clearSessionChecklistState } from '~/services/moderation/checklist-session-storage.ts'
import { clearChecklistState } from '~/services/moderation/checklist-storage.ts'
import { useModerationQueue } from '~/services/moderation/queue.ts'
import {
	batchCheckQueueCandidates,
	findNextEligibleQueueProject,
	isEligibleQueueCandidate,
	type QueueCandidateCheck,
} from '~/services/moderation/queue-eligibility.ts'

import NodeRenderer from './node-renderer/index.vue'
import type { LiveNode } from './types'
import { useChecklistLock } from './use-lock'
import { useNodeRendererState } from './use-node-renderer-state'
import { loadChecklistPersistence, useChecklistPersistence } from './use-persistence'
import { useModerationSubmission } from './use-submission'

const notifications = injectNotificationManager()
const { addNotification } = notifications
const debug = useDebugLogger('ModerationChecklist')
const keybinds = useModerationKeybinds()
const settings = useModerationSettings()

const takeOverModal = ref<InstanceType<typeof ConfirmModal>>()

const props = defineProps<{
	collapsed: boolean
}>()

const collapseLabel = computed(() => (props.collapsed ? 'Expand' : 'Collapse'))

const { projectV2, projectV3, versions, loadVersions, invalidate, thread } =
	injectProjectPageContext()
setMessageProject(projectV3, projectV2)
const missingMdPaths = new Set<string>()
setMissingMdHandler((path) => missingMdPaths.add(path))

const nodeStates = ref<Record<string, Record<string, NodeState>>>({})
const resolvedStages = ref(useStages(nodeStates))
const client = injectModrinthClient()

const moderationQueue = useModerationQueue()
const completionMessage = computed(() => {
	const remaining = moderationQueue.queueLength
	return moderationQueue.hasItems
		? `You are done moderating this project! There ${remaining === 1 ? 'is' : 'are'} ${remaining} ${remaining === 1 ? 'project' : 'projects'} left.`
		: 'You are done moderating this project!'
})
const queryClient = useQueryClient()
const tags = useGeneratedState()
const auth = await useAuth()

const alreadyReviewed = ref(false)

interface PrefetchedProject {
	project: string
	slug: string
	projectType: string
	validatedAt: number
}

const prefetchQueue = ref<PrefetchedProject[]>([])
const isPrefetching = ref(false)

const PREFETCH_STALE_MS = 30_000
const PREFETCH_TARGET_COUNT = 3
const PREFETCH_BATCH_SIZE = 5

async function navigateToNextUnlockedProject(): Promise<boolean> {
	const now = Date.now()
	prefetchQueue.value = prefetchQueue.value.filter((p) => now - p.validatedAt < PREFETCH_STALE_MS)

	if (prefetchQueue.value.length === 0) return false

	const next = prefetchQueue.value[0]

	// Quick re-check if close to expiry (last 5 seconds of TTL)
	if (now - next.validatedAt > PREFETCH_STALE_MS - 5000) {
		const recheckResults = await batchCheckQueueCandidates(client, moderationQueue, [next.project])
		const recheck = recheckResults.get(next.project)
		if (!isEligibleQueueCandidate(recheck)) {
			prefetchQueue.value.shift()
			return navigateToNextUnlockedProject()
		}
	}

	prefetchQueue.value.shift()

	maintainPrefetchQueue()

	navigateToQueueProject(
		{ slug: next.slug, projectType: next.projectType, locked: false, isProcessing: true },
		next.project,
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
const {
	activatedStages,
	markStageVisited,
	message,
	persistedState,
	reviewedAnyway,
	visitedStages,
} = await loadChecklistPersistence(checklistPersistenceProjectId)
nodeStates.value = persistedState?.state ?? {}
const generatedActiveActions = ref<ActiveAction[] | null>(null)
const resolvedMessageAvailability = ref<Map<object, boolean>>(new Map())
const generatedMessage = computed(() => message.value !== null)
const loadingMessage = ref(false)
const moderationDecision = ref<ProjectStatus | null>(null)
const loadingModerationDecision = computed(() => moderationDecision.value !== null)
const approveSendStatus = computed<ProjectStatus>(() => {
	const requested = projectV2.value.requested_status
	return requested ?? 'approved'
})
const moderationSubmission = useModerationSubmission({
	project: projectV3,
	projectV2,
	versions,
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
	await persistImmediately(false, true)
	disablePersistence()
	emit('exit')
}

function openTakeOverModal() {
	takeOverModal.value?.show()
}

function reviewAnyway() {
	alreadyReviewed.value = false
	reviewedAnyway.value = true
	persist()
	maintainPrefetchQueue()
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

async function maintainPrefetchQueue() {
	if (isPrefetching.value) return
	if (!moderationQueue.isQueueMode) return

	const currentProjectId = projectV2.value?.id

	isPrefetching.value = true

	try {
		const now = Date.now()
		prefetchQueue.value = prefetchQueue.value.filter((p) => now - p.validatedAt < PREFETCH_STALE_MS)

		if (currentProjectId) {
			prefetchQueue.value = prefetchQueue.value.filter((p) => p.project !== currentProjectId)
		}

		if (prefetchQueue.value.length >= PREFETCH_TARGET_COUNT) {
			return
		}

		const prefetchedIds = new Set(prefetchQueue.value.map((p) => p.project))
		const queueItems = [...moderationQueue.currentQueue.items]
		const currentIndex = currentProjectId ? queueItems.indexOf(currentProjectId) : -1
		const remainingItems =
			currentIndex >= 0 ? queueItems.slice(currentIndex + 1) : queueItems.slice(1)

		const candidateIds = remainingItems.filter((id) => !prefetchedIds.has(id))

		if (candidateIds.length === 0) return

		let checkedCount = 0

		while (
			prefetchQueue.value.length < PREFETCH_TARGET_COUNT &&
			checkedCount < candidateIds.length
		) {
			const batch = candidateIds.slice(checkedCount, checkedCount + PREFETCH_BATCH_SIZE)
			checkedCount += batch.length

			const results = await batchCheckQueueCandidates(client, moderationQueue, batch)

			for (const id of batch) {
				const result = results.get(id)
				if (isEligibleQueueCandidate(result)) {
					prefetchQueue.value.push({
						project: id,
						slug: result?.slug ?? '',
						projectType: result?.projectType ?? '',
						validatedAt: Date.now(),
					})

					if (prefetchQueue.value.length >= PREFETCH_TARGET_COUNT) break
				} else {
					void moderationQueue.excludeProject(id)
				}
			}
		}
	} finally {
		isPrefetching.value = false
	}
}

const debouncedPrefetch = useDebounceFn(maintainPrefetchQueue, 300)

const {
	acquire: acquireLock,
	error: lockError,
	handleVisibilityChange,
	override: confirmTakeOverOverride,
	status: lockStatus,
	stop: stopLockMonitoring,
	timeRemaining: lockTimeRemaining,
} = useChecklistLock({
	projectId: projectV2.value.id,
	queue: moderationQueue,
	addNotification,
	refreshPrefetchQueue: () => void maintainPrefetchQueue(),
})

const lockDescription = computed(() => {
	const hasModerator = Boolean(lockStatus.value?.lockedBy?.username)
	if (lockStatus.value?.expired) {
		return `This project was being moderated${hasModerator ? ' by' : ''}`
	}
	return `This project is currently being moderated${hasModerator ? ' by' : ''}`
})

async function goToNextEligibleProject(candidateIds: string[]): Promise<boolean> {
	if (candidateIds.length === 0) return false

	const next = await findNextEligibleQueueProject(client, moderationQueue, candidateIds)

	if (!next) {
		await Promise.all(candidateIds.map((id) => moderationQueue.excludeProject(id)))
		return false
	}

	await Promise.all(next.excluded.map((id) => moderationQueue.excludeProject(id)))
	notifySkippedQueueProjects(next.excluded.length)
	navigateToQueueProject(next.result, next.project)
	return true
}

async function skipToNextProject() {
	const currentProjectId = projectV2.value?.id
	if (!currentProjectId) {
		console.warn('[skipToNextProject] No current project ID, aborting')
		return
	}
	debug('[skipToNextProject] Starting. Current project:', currentProjectId)
	debug('[skipToNextProject] Queue before complete:', [...moderationQueue.currentQueue.items])

	await moderationQueue.deferProject(currentProjectId)

	debug('[skipToNextProject] Queue after complete:', [...moderationQueue.currentQueue.items])
	debug('[skipToNextProject] hasItems:', moderationQueue.hasItems)

	if (await navigateToNextUnlockedProject()) {
		debug('[skipToNextProject] Used prefetch, returning')
		return
	}

	debug('[skipToNextProject] No prefetch, entering fallback with batch checking')

	const remainingIds = moderationQueue.currentQueue.items.filter((id) => id !== currentProjectId)

	if (remainingIds.length > 0) {
		if (await goToNextEligibleProject(remainingIds)) return

		debug('[skipToNextProject] No eligible projects in queue')
		addNotification({
			title: 'No projects available',
			text: 'All remaining projects are already moderated, deleted, or locked by others.',
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

const resetLabel = computed(() => {
	if (!isPseudoStage.value && currentStageHasState.value) {
		return 'Reset stage'
	}
	if (!isPseudoStage.value && !checklistHasState.value) {
		return 'Return to start'
	}
	return 'Reset checklist'
})

const checklistTitleText = computed(() => {
	if (alreadyReviewed.value || done.value) return 'Moderation'
	if (generatedMessage.value) return 'Generated message'

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
			const { proxy, changes } = createTrackedPatch(
				projectV3.value as Labrinth.Projects.v3.EditProjectRequest,
			)
			f._projectFn(proxy, state)
			return Object.keys(changes()).length > 0
		}
		if (f._versionFn) {
			const version = versions.value?.[0]
			if (!version) return true
			const { proxy, changes } = createTrackedPatch(
				version as Labrinth.Versions.v3.ModifyVersionRequest,
			)
			f._versionFn(proxy, state)
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
const hasSettledInitialStage = ref(!needsInitialStageSettle)

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
			hasSettledInitialStage.value = true
		},
		{ once: true },
	)
}

const router = useRouter()
const route = useRoute()

const projectUrlType = computed(() =>
	getProjectTypeForUrlShorthand(
		projectV2.value.project_type,
		projectV2.value.loaders ?? [],
		tags.value,
	),
)

let lastSyncedStageTarget: string | null = null
function syncStageUrl(stage: StageNode | undefined) {
	const navigate = stage?._navigate
	if (navigate === undefined) return
	const target = `/${projectUrlType.value}/${projectV2.value.slug}${navigate}`
	if (target === lastSyncedStageTarget) return
	lastSyncedStageTarget = target
	setTimeout(() => router.replace(target), 0)
}

watch(
	hasSettledInitialStage,
	(settled) => {
		if (settled) syncStageUrl(currentStageObj.value)
	},
	{ immediate: true },
)

const stageNavigateTarget = computed(() => {
	const navigate = currentStageObj.value?._navigate
	if (navigate === undefined || !projectV2.value) return null
	const base = `/${projectUrlType.value}/${projectV2.value.slug}`
	return navigate === '' ? base : `${base}${navigate}`
})

const stageNavigateLabel = computed(() => {
	const navigate = currentStageObj.value?._navigate
	if (navigate === '') return 'Project page'
	const segment = navigate?.split('/').filter(Boolean).pop()
	if (!segment) return ''
	return segment.charAt(0).toUpperCase() + segment.slice(1)
})

const stageNavigateButtonLabel = computed(() => `Navigate to ${stageNavigateLabel.value}`)

function navigateToStagePage() {
	if (stageNavigateTarget.value) router.push(stageNavigateTarget.value)
}

const {
	disable: disablePersistence,
	dispose: disposePersistence,
	persist,
	persistImmediately,
} = useChecklistPersistence({
	projectId: checklistPersistenceProjectId,
	nodeStates,
	activatedStages,
	reviewedAnyway,
	message,
	currentStage,
	currentStageNode: currentStageObj,
	firstVisibleStage: findFirstValidStage,
	markStageVisited,
	visitCurrentStageImmediately: !needsInitialStageSettle,
})

watch(
	nodeStates,
	async () => {
		const active = collectAllActiveActions()
		const newMap = new Map<object, boolean>()
		await Promise.all(
			active
				.filter((action) => getSegments(action.node).some((segment) => segment.type !== 'collect'))
				.map(async ({ node, state, statePath }) => {
					try {
						let hasContent = false
						for (const seg of getSegments(node)) {
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
	void persistImmediately(true)
	window.addEventListener('keydown', handleKeybinds)
	window.addEventListener('beforeunload', handleBeforeUnload)
	document.addEventListener('visibilitychange', handleVisibilityChange)
	if (settings.value.get(moderationSettings.General.ChecklistPosition) === 'right') {
		notifications.setNotificationLocation('left')
	}

	if (done.value || alreadyReviewed.value) return

	await acquireLock()
})

function handleBeforeUnload() {
	const projectId = projectV2.value?.id
	if (!projectId || !lockStatus.value?.isOwnLock) return

	const config = useRuntimeConfig()
	const base = config.public.apiBaseUrl.replace(/\/v\d\/?$/, '/_internal/')
	const token = (auth as unknown as { value?: { token?: string } }).value?.token
	if (!token) return

	// sendBeacon is POST-only and cannot set Authorization. The internal release endpoint
	// accepts the API bearer token as text/plain.
	void navigator.sendBeacon(
		`${base}moderation/lock/${projectId}/release`,
		new Blob([token], { type: 'text/plain' }),
	)
}

onUnmounted(() => {
	disposePersistence()
	window.removeEventListener('beforeunload', handleBeforeUnload)
	window.removeEventListener('keydown', handleKeybinds)
	document.removeEventListener('visibilitychange', handleVisibilityChange)
	notifications.setNotificationLocation('right')

	stopLockMonitoring()

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
		if (hasSettledInitialStage.value && oldIndex !== undefined && newIndex !== oldIndex) {
			syncStageUrl(resolvedStages.value[newIndex])
		}
	},
	{ immediate: true },
)

loadVersions()

function countStageActions(stage: StageNode): number {
	const actions = checklistLive.value.get(stage)?.activeActions ?? []
	const resolved = resolvedMessageAvailability.value
	return actions.filter((a) => {
		if (getSegments(a.node).every((segment) => segment.type === 'collect')) return false
		return resolved.get(a.node) ?? true
	}).length
}

function countStageFixes(stage: StageNode): number {
	return checklistLive.value.get(stage)?.fixCount ?? 0
}

function hasRequiredMissing(stage: StageNode): boolean {
	return checklistLive.value.get(stage)?.hasRequiredMissing ?? false
}

function collectAllActiveActions(): ActiveAction[] {
	return resolvedStages.value.flatMap((s) => checklistLive.value.get(s)?.activeActions ?? [])
}

function byPriority(a: ActiveAction, b: ActiveAction): number {
	return getPriority(a.node).compareTo(getPriority(b.node))
}

function getSegments(node: object): MessageSegment[] {
	return '_segments' in node && Array.isArray(node._segments)
		? (node._segments as MessageSegment[])
		: []
}

function getPriority(node: object): Priority {
	return (node as { _priority: Priority })._priority
}

async function assembleFullMessage() {
	const allEntries = collectAllActiveActions()
	generatedActiveActions.value = allEntries

	const consumed = new Set<object>()

	const parts: { entry: ActiveAction; content: string }[] = []
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

const {
	meta: stageMeta,
	nodes: stageNodes,
	state: stageState,
	write: stageWriter,
} = useNodeRendererState({
	currentStage: currentStageObj,
	nodeStates,
	project: projectV3,
	projectV2,
	isFixActionable,
})

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
	markStageVisited(currentStageObj.value.id)

	router.push(`/${projectUrlType.value}/${projectV2.value.slug}/moderation`)

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

const finishedId = import.meta.client
	? localStorage.getItem('moderation-checklist-finished')
	: null
if (finishedId === projectV2.value.id) {
	localStorage.removeItem('moderation-checklist-finished')
	hasNextProject.value = moderationQueue.queueLength > 0
	done.value = true
} else if (projectV2.value.status !== 'processing' && !reviewedAnyway.value) {
	alreadyReviewed.value = true
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
	moderationDecision.value = status
	try {
		const projectFixChanges = await moderationSubmission.mutateAsync({
			status,
			message: message.value,
			activeActions: active,
		})

		const willHaveNext = await moderationQueue.completeProject(projectId)
		// Set both states together - hasNextProject MUST be set before done
		// to avoid the race condition where done=true renders with hasNextProject=false
		hasNextProject.value = willHaveNext
		done.value = true
		clearGeneratedMessageState()
		await nextTick()

		await refreshModerationCaches(threadId)

		await Promise.race([
			moderationQueue.releaseLock(projectId),
			new Promise((r) => setTimeout(r, 2000)),
		])

		if (projectFixChanges?.slug) {
			const urlType = getProjectTypeForUrlShorthand(projectV2.value.project_type, [], tags.value)
			localStorage.setItem('moderation-checklist-finished', projectId)
			await navigateTo(`/${urlType}/${projectFixChanges.slug}/moderation`, { replace: true })
			return
		}
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
		const currentProjectId = projectV2.value?.id
		const isRealQueue =
			!!currentProjectId &&
			(moderationQueue.currentQueue.completed.includes(currentProjectId) ||
				moderationQueue.currentQueue.skipped.includes(currentProjectId))

		await navigateTo({
			name: 'moderation',
			state: {
				confetti: true,
				queueSummary: isRealQueue,
			},
		})

		await nextTick()

		if (!isRealQueue) {
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

			if (!(await goToNextEligibleProject(remainingIds))) {
				if (remainingIds.length > 0) {
					addNotification({
						title: 'No projects available',
						text: 'All remaining projects are already moderated, deleted, or locked by others.',
						type: 'warning',
					})
				}

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

	hasNextProject.value = await moderationQueue.deferProject(projectId)

	await endChecklist('skipped')
}

async function clearProjectLocalStorage() {
	disablePersistence()

	nodeStates.value = {}
	activatedStages.value = new Set()
	visitedStages.value = new Set()
	message.value = null
	await clearChecklistState(checklistPersistenceProjectId)
	clearSessionChecklistState(checklistPersistenceProjectId)
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
	label: string
	action: () => void
	text: string
	icon?: Component
	messages?: number
	fixes?: number
	requiredMissing?: boolean
	visited?: boolean
	tone?: 'green'
}

const stageOptions = computed<StageOption[]>(() => {
	const options = resolvedStages.value
		.map((stage, index) => {
			if (!shouldShowStage(stage)) return null

			const label = stage.label ?? kebabToTitleCase(stage.id)
			return {
				id: String(index),
				label,
				action: () => {
					clearGeneratedMessageState()
					currentStage.value = index
				},
				text: label,
				icon: stage._icon ?? undefined,
				messages: countStageActions(stage) || undefined,
				fixes: countStageFixes(stage) || undefined,
				requiredMissing: hasRequiredMissing(stage) || undefined,
				visited:
					((index !== currentStage.value || generatedMessage.value) &&
						stage.id &&
						visitedStages.value.has(stage.id)) ||
					undefined,
				tone: index === currentStage.value && !generatedMessage.value ? 'green' : undefined,
			}
		})
		.filter((opt): opt is StageOption => opt !== null)

	options.push({
		id: 'generate-message',
		label: 'Generate Message',
		action: () => generateMessage(),
		text: 'Generate Message',
		icon: CheckIcon,
		tone: generatedMessage.value ? 'green' : undefined,
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
