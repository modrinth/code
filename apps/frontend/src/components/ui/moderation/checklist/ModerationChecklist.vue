<template>
	<KeybindsModal ref="keybindsModal" />
	<div
		tabindex="0"
		class="moderation-checklist flex w-[600px] max-w-full flex-col rounded-2xl border-[1px] border-solid border-orange bg-bg-raised p-4 transition-all delay-200 duration-200 ease-in-out"
		:class="{ '!w-fit': collapsed, locked: lockStatus?.locked && !lockStatus?.isOwnLock }"
	>
		<div class="flex grow-0 items-center gap-2">
			<h1 class="m-0 mr-auto flex items-center gap-2 text-2xl font-extrabold text-contrast">
				<ScaleIcon class="text-orange" /> Moderation
			</h1>
			<ButtonStyled circular>
				<button v-tooltip="`Keyboard shortcuts`" @click="keybindsModal?.show($event)">
					<KeyboardIcon />
				</button>
			</ButtonStyled>
			<ButtonStyled circular>
				<a v-tooltip="`Stage guidance`" target="_blank" :href="currentStageObj.guidance_url">
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

		<Collapsible base-class="grow" class="flex grow flex-col" :collapsed="collapsed">
			<div class="my-4 h-[1px] w-full bg-divider" />

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
						class="mt-4 flex grow justify-between gap-2 border-0 border-t-[1px] border-solid border-divider pt-4"
					>
						<div class="flex items-center gap-2">
							<ButtonStyled v-if="lockStatus.expired" @click="retryAcquireLock">
								<button>
									<LockIcon aria-hidden="true" />
									Take over
								</button>
							</ButtonStyled>
						</div>
						<div class="flex items-center gap-2">
							<ButtonStyled
								v-if="moderationStore.isQueueMode && moderationStore.queueLength > 1"
								color="brand"
								@click="skipToNextProject"
							>
								<button>
									<RightArrowIcon aria-hidden="true" />
									Next project ({{ moderationStore.queueLength }} left)
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
						class="mt-4 flex grow justify-between gap-2 border-0 border-t-[1px] border-solid border-divider pt-4"
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
								v-if="moderationStore.isQueueMode && moderationStore.queueLength > 1"
								color="brand"
								@click="skipToNextProject"
							>
								<button>
									<RightArrowIcon aria-hidden="true" />
									Next project ({{ moderationStore.queueLength }} left)
								</button>
							</ButtonStyled>
						</div>
					</div>
				</div>
			</div>

			<template v-else>
				<div class="flex-1">
					<div v-if="done">
						<p>
							You are done moderating this project!
							<template v-if="moderationStore.hasItems">
								There are
								{{ moderationStore.queueLength }} left.
							</template>
						</p>
					</div>
					<div v-else-if="generatedMessage">
						<div>
							<ButtonStyled>
								<button class="mb-2" @click="useSimpleEditor = !useSimpleEditor">
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
							<MarkdownEditor
								v-if="!useSimpleEditor"
								v-model="message"
								:max-height="400"
								placeholder="No message generated."
								:disabled="false"
								:heading-buttons="false"
							/>
							<textarea
								v-else
								v-model="message"
								type="text"
								class="bg-bg-input h-[400px] w-full rounded-lg border border-solid border-divider px-3 py-2 font-mono text-base"
								placeholder="No message generated."
								autocomplete="off"
								@input="persistState"
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
					<div v-else>
						<h2 class="m-0 mb-2 text-lg font-extrabold">
							{{ currentStageObj.title }}
						</h2>

						<div v-if="currentStageObj.text" class="mb-4">
							<div v-if="stageTextExpanded" class="markdown-body" v-html="stageTextExpanded"></div>
							<div v-else class="markdown-body">Loading stage content...</div>
						</div>

						<!-- Action components grouped by type -->
						<div class="space-y-4">
							<!-- Button actions group -->
							<div v-if="buttonActions.length > 0" class="button-actions-group">
								<div class="flex flex-wrap gap-2">
									<template v-for="action in buttonActions" :key="getActionKey(action)">
										<ButtonStyled
											:color="isActionSelected(action) ? 'brand' : 'standard'"
											@click="toggleAction(action)"
										>
											<button>
												{{ action.label }}
											</button>
										</ButtonStyled>
									</template>
								</div>
							</div>

							<!-- Toggle actions group -->
							<div v-if="toggleActions.length > 0" class="toggle-actions-group space-y-3">
								<template v-for="action in toggleActions" :key="getActionKey(action)">
									<Checkbox
										:model-value="isActionSelected(action)"
										:label="action.label"
										:description="action.description"
										:disabled="false"
										@update:model-value="toggleAction(action)"
									/>
								</template>
							</div>

							<!-- Dropdown actions group -->
							<div v-if="dropdownActions.length > 0" class="dropdown-actions-group space-y-3">
								<template v-for="action in dropdownActions" :key="getActionKey(action)">
									<div class="inputs universal-labels">
										<div>
											<label :for="`dropdown-${getActionId(action)}`">
												<span class="label__title">{{ action.label }}</span>
											</label>
											<DropdownSelect
												:max-visible-options="3"
												render-up
												:name="`dropdown-${getActionId(action)}`"
												:options="getVisibleDropdownOptions(action)"
												:model-value="getDropdownValue(action)"
												:placeholder="'Select an option'"
												:disabled="false"
												:display-name="(opt: any) => opt?.label || 'Unknown option'"
												@update:model-value="
													(selected: any) => selectDropdownOption(action, selected)
												"
											/>
										</div>
									</div>
								</template>
							</div>

							<!-- Multi-select chips actions group -->
							<div
								v-if="multiSelectActions.length > 0"
								class="multi-select-actions-group space-y-3"
							>
								<template v-for="action in multiSelectActions" :key="getActionKey(action)">
									<div>
										<div class="mb-2 font-semibold">{{ action.label }}</div>
										<div class="flex flex-wrap gap-2">
											<ButtonStyled
												v-for="(option, optIndex) in getVisibleMultiSelectOptions(action)"
												:key="`${getActionId(action)}-chip-${optIndex}`"
												:color="isChipSelected(action, optIndex) ? 'brand' : 'standard'"
												@click="toggleChip(action, optIndex)"
											>
												<button>
													{{ option.label }}
												</button>
											</ButtonStyled>
										</div>
									</div>
								</template>
							</div>
						</div>

						<div v-if="isAnyVisibleInputs" class="my-4 h-[1px] w-full bg-divider" />

						<!-- Additional text inputs -->
						<div class="space-y-4">
							<template v-for="action in visibleActions" :key="`inputs-${getActionKey(action)}`">
								<div
									v-if="action.relevantExtraInput && isActionSelected(action)"
									class="inputs universal-labels"
								>
									<div
										v-for="(input, inputIndex) in getVisibleInputs(action, actionStates)"
										:key="`input-${getActionId(action)}-${inputIndex}`"
										class="mt-2"
									>
										<template v-if="input.large">
											<label :for="`input-${getActionId(action)}-${inputIndex}`">
												<span class="label__title">
													{{ input.label }}
													<span v-if="input.required" class="required">*</span>
												</span>
											</label>
											<MarkdownEditor
												:id="`input-${getActionId(action)}-${inputIndex}`"
												v-model="textInputValues[`${getActionId(action)}-${inputIndex}`]"
												:placeholder="input.placeholder"
												:max-height="300"
												:disabled="false"
												:heading-buttons="false"
												@input="persistState"
											/>
										</template>
										<template v-else>
											<label :for="`input-${getActionId(action)}-${inputIndex}`">
												<span class="label__title">
													{{ input.label }}
													<span v-if="input.required" class="required">*</span>
												</span>
											</label>
											<input
												:id="`input-${getActionId(action)}-${inputIndex}`"
												v-model="textInputValues[`${getActionId(action)}-${inputIndex}`]"
												type="text"
												:placeholder="input.placeholder"
												autocomplete="off"
												@input="persistState"
											/>
										</template>
									</div>
								</div>
							</template>
						</div>
					</div>
				</div>

				<!-- Stage control buttons -->
				<div class="mt-auto">
					<div
						class="mt-4 flex grow justify-between gap-2 border-0 border-t-[1px] border-solid border-divider pt-4"
					>
						<div class="flex items-center gap-2">
							<ButtonStyled v-if="!done && !generatedMessage && moderationStore.hasItems">
								<button @click="skipCurrentProject">
									<XIcon aria-hidden="true" />
									Skip ({{ moderationStore.queueLength }} left)
								</button>
							</ButtonStyled>
						</div>

						<div class="flex items-center gap-2">
							<div v-if="done">
								<ButtonStyled color="brand">
									<button @click="endChecklist(undefined)">
										<template v-if="hasNextProject">
											<RightArrowIcon aria-hidden="true" />
											Next project ({{ moderationStore.queueLength }} left)
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
									<button @click="goBackToStages">
										<LeftArrowIcon aria-hidden="true" />
										Edit
									</button>
								</ButtonStyled>
								<ButtonStyled color="red">
									<button @click="sendMessage('rejected')">
										<XIcon aria-hidden="true" />
										Reject
									</button>
								</ButtonStyled>
								<ButtonStyled color="orange">
									<button @click="sendMessage('withheld')">
										<EyeOffIcon aria-hidden="true" />
										Withhold
									</button>
								</ButtonStyled>
								<ButtonStyled color="green">
									<button @click="sendMessage(projectV2.requested_status ?? 'approved')">
										<CheckIcon aria-hidden="true" />
										Approve
									</button>
								</ButtonStyled>
							</div>

							<div v-else class="flex items-center gap-2">
								<OverflowMenu
									v-if="!generatedMessage"
									:options="stageOptions"
									class="bg-transparent p-0"
								>
									<ButtonStyled circular>
										<button v-tooltip="`Stages`">
											<ListBulletedIcon />
										</button>
									</ButtonStyled>

									<template
										v-for="opt in stageOptions.filter(
											(opt) => 'id' in opt && 'text' in opt && 'icon' in opt,
										)"
										#[opt.id]
										:key="opt.id"
									>
										<component :is="opt.icon" v-if="opt.icon" class="mr-2" />
										{{ opt.text }}
									</template>
								</OverflowMenu>
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
	EyeOffIcon,
	FileTextIcon,
	KeyboardIcon,
	LeftArrowIcon,
	ListBulletedIcon,
	LockIcon,
	RightArrowIcon,
	ScaleIcon,
	ToggleLeftIcon,
	ToggleRightIcon,
	XIcon,
} from '@modrinth/assets'
import {
	type Action,
	type ButtonAction,
	checklist,
	type ConditionalButtonAction,
	deserializeActionStates,
	type DropdownAction,
	expandVariables,
	finalPermissionMessages,
	findMatchingVariant,
	flattenProjectV3Variables,
	flattenProjectVariables,
	flattenStaticVariables,
	getActionIdForStage,
	getActionMessage,
	getVisibleInputs,
	handleKeybind,
	initializeActionState,
	kebabToTitleCase,
	keybinds,
	type MultiSelectChipsAction,
	processMessage,
	serializeActionStates,
	type Stage,
	type ToggleAction,
} from '@modrinth/moderation'
import {
	Avatar,
	ButtonStyled,
	Checkbox,
	Collapsible,
	DropdownSelect,
	injectNotificationManager,
	injectProjectPageContext,
	MarkdownEditor,
	OverflowMenu,
	type OverflowMenuOption,
	useDebugLogger,
} from '@modrinth/ui'
import {
	type ModerationJudgements,
	type ModerationModpackItem,
	type ProjectStatus,
	renderHighlightedString,
} from '@modrinth/utils'
import { computedAsync, useDebounceFn, useLocalStorage } from '@vueuse/core'

import { useGeneratedState } from '~/composables/generated'
import { getProjectTypeForUrlShorthand } from '~/helpers/projects.js'
import { useModerationStore } from '~/store/moderation.ts'

import KeybindsModal from './ChecklistKeybindsModal.vue'
import ModpackPermissionsFlow from './ModpackPermissionsFlow.vue'

const notifications = injectNotificationManager()
const { addNotification } = notifications
const debug = useDebugLogger('ModerationChecklist')

const keybindsModal = ref<InstanceType<typeof KeybindsModal>>()

const props = defineProps<{
	collapsed: boolean
}>()

const { projectV2, projectV3 } = injectProjectPageContext()

const moderationStore = useModerationStore()
const tags = useGeneratedState()
const auth = await useAuth()

const lockStatus = ref<{
	locked: boolean
	lockedBy?: { id: string; username: string; avatar_url?: string }
	lockedAt?: Date
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

const LOCK_EXPIRY_MINUTES = 15

function handleVisibilityChange() {
	if (document.visibilityState === 'visible' && lockStatus.value?.isOwnLock) {
		// Immediately refresh the lock when returning to the tab
		// This handles cases where the heartbeat was throttled while backgrounded
		moderationStore.refreshLock()
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
	const expiresAt = new Date(lockedAt.getTime() + LOCK_EXPIRY_MINUTES * 60 * 1000)
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
			await moderationStore.refreshLock()
		},
		5 * 60 * 1000,
	)
}

function handleLockAcquired() {
	lockStatus.value = { locked: false, isOwnLock: true }
	lockError.value = false
	initializeAllStages()
	clearLockCountdown()
	startLockHeartbeat()
	maintainPrefetchQueue() // Start prefetching immediately (not debounced)
}

function handleLockUnavailable() {
	lockError.value = true
	lockStatus.value = { locked: false, isOwnLock: false }
	initializeAllStages()
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
		const recheck = await moderationStore.checkLock(next.projectId)
		if (recheck.locked && !recheck.expired) {
			// Project got locked, remove from queue and try next
			prefetchQueue.value.shift()
			return navigateToNextUnlockedProject() // Recurse to try next
		}
	}

	// Remove from queue after validation
	prefetchQueue.value.shift()

	// Mark skipped projects as completed
	next.skippedIds.forEach((id) => moderationStore.completeCurrentProject(id, 'skipped'))

	if (next.skippedIds.length > 0) {
		addNotification({
			title: 'Skipped locked projects',
			text: `Skipped ${next.skippedIds.length} project(s) being moderated by others.`,
			type: 'info',
		})
	}

	// Trigger prefetch replenishment in background (don't await)
	maintainPrefetchQueue()

	// Navigate to canonical URL if we have metadata (avoids middleware redirect)
	if (next.slug && next.projectType) {
		const urlType = getProjectTypeForUrlShorthand(next.projectType, [], tags.value)

		navigateTo({
			path: `/${urlType}/${next.slug}`,
			state: { showChecklist: true },
		})
	} else {
		// Fallback: use project ID (will trigger middleware redirect)
		navigateTo({
			name: 'type-id',
			params: { type: 'project', id: next.projectId },
			state: { showChecklist: true },
		})
	}
	return true
}

const variables = computed(() => {
	return {
		...flattenStaticVariables(),
		...flattenProjectVariables(projectV2.value),
		...flattenProjectV3Variables(projectV3.value),
	}
})

const modpackPermissionsComplete = ref(false)
const modpackJudgements = ref<ModerationJudgements>({})
const isModpackPermissionsStage = computed(() => {
	return currentStageObj.value.id === 'modpack-permissions'
})

const useSimpleEditor = ref(false)
const message = ref('')
const generatedMessage = ref(false)
const loadingMessage = ref(false)
const done = ref(false)

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
		const released = await moderationStore.releaseLock(projectId)
		if (!released && lockStatus.value?.isOwnLock) {
			console.warn('Failed to release moderation lock for project:', projectId)
		}
	}
	emit('exit')
}

async function retryAcquireLock() {
	const projectId = projectV2.value?.id
	if (!projectId) {
		console.warn('[retryAcquireLock] No project ID available')
		return
	}
	const result = await moderationStore.acquireLock(projectId)

	if (result.success) {
		handleLockAcquired()
	} else if (result.locked_by) {
		// Still locked by another moderator, update status
		lockStatus.value = {
			locked: true,
			lockedBy: result.locked_by,
			lockedAt: result.locked_at ? new Date(result.locked_at) : undefined,
			expired: result.expired,
			isOwnLock: false,
		}
		lockError.value = false

		// Restart countdown timer
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
	initializeAllStages()
	// Start prefetching the next project in the background
	maintainPrefetchQueue()
}

// Batch check locks and fetch project metadata in parallel
interface LockCheckResult {
	locked: boolean
	expired?: boolean
	isOwnLock?: boolean
	slug?: string
	projectType?: string
}

async function batchCheckLocksWithMetadata(
	projectIds: string[],
): Promise<Map<string, LockCheckResult>> {
	const results = new Map<string, LockCheckResult>()
	const currentUserId = (auth.value?.user as { id?: string } | null)?.id

	// Check locks and fetch minimal project data in parallel
	const checks = await Promise.allSettled(
		projectIds.map(async (id) => {
			// Parallel: check lock AND fetch project metadata
			const [lockStatus, projectData] = await Promise.all([
				moderationStore.checkLock(id),
				useBaseFetch(`project/${id}`, { method: 'GET' }).catch(() => null),
			])

			// Check if lock is by the current user (own lock = can acquire)
			const isOwnLock = lockStatus.locked_by?.id === currentUserId

			return {
				id,
				locked: lockStatus.locked,
				expired: lockStatus.expired,
				isOwnLock,
				slug: (projectData as { slug?: string })?.slug,
				projectType: (projectData as { project_type?: string })?.project_type,
			}
		}),
	)

	// Use forEach with index to avoid indexOf bug on PromiseSettledResult
	checks.forEach((result, index) => {
		if (result.status === 'fulfilled') {
			results.set(result.value.id, result.value)
		} else {
			// On error, mark as needing fallback (no metadata)
			results.set(projectIds[index], { locked: false })
		}
	})

	return results
}

// Maintain a queue of prefetched unlocked projects for instant navigation
async function maintainPrefetchQueue() {
	if (isPrefetching.value) return
	if (!moderationStore.isQueueMode) return

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
		const queueItems = [...moderationStore.currentQueue.items]
		const currentIndex = currentProjectId ? queueItems.indexOf(currentProjectId) : -1
		const remainingItems =
			currentIndex >= 0 ? queueItems.slice(currentIndex + 1) : queueItems.slice(1)

		const candidateIds = remainingItems
			.filter((id) => !prefetchedIds.has(id))
			.slice(0, PREFETCH_BATCH_SIZE * 2) // Check up to 10 candidates

		if (candidateIds.length === 0) return

		// 5. Batch check locks AND fetch metadata in parallel
		const skippedIds: string[] = []
		let checkedCount = 0

		while (
			prefetchQueue.value.length < PREFETCH_TARGET_COUNT &&
			checkedCount < candidateIds.length
		) {
			const batch = candidateIds.slice(checkedCount, checkedCount + PREFETCH_BATCH_SIZE)
			checkedCount += batch.length

			const results = await batchCheckLocksWithMetadata(batch)

			for (const id of batch) {
				const result = results.get(id)
				// Treat as unlocked if: not locked, OR expired, OR it's our own lock
				if (!result?.locked || result?.expired || result?.isOwnLock) {
					// Found unlocked project with metadata
					if (result?.slug && result?.projectType) {
						prefetchQueue.value.push({
							projectId: id,
							slug: result.slug,
							projectType: result.projectType,
							validatedAt: Date.now(),
							skippedIds: [...skippedIds],
						})
					} else {
						// No metadata - still add but will need fallback navigation
						prefetchQueue.value.push({
							projectId: id,
							slug: '', // Empty = use fallback
							projectType: '',
							validatedAt: Date.now(),
							skippedIds: [...skippedIds],
						})
					}

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

const MAX_SKIP_ATTEMPTS = 10

async function skipToNextProject() {
	// Skip the current project
	const currentProjectId = projectV2.value?.id
	if (!currentProjectId) {
		console.warn('[skipToNextProject] No current project ID, aborting')
		return
	}
	debug('[skipToNextProject] Starting. Current project:', currentProjectId)
	debug('[skipToNextProject] Queue before complete:', [...moderationStore.currentQueue.items])

	moderationStore.completeCurrentProject(currentProjectId, 'skipped')

	debug('[skipToNextProject] Queue after complete:', [...moderationStore.currentQueue.items])
	debug('[skipToNextProject] hasItems:', moderationStore.hasItems)

	// Use prefetched data if available
	if (await navigateToNextUnlockedProject()) {
		debug('[skipToNextProject] Used prefetch, returning')
		return
	}

	debug('[skipToNextProject] No prefetch, entering fallback with batch checking')

	// Fallback: batch check remaining projects with metadata (excluding current)
	const remainingIds: string[] = []
	const queueItems = moderationStore.currentQueue.items

	// Build list of remaining projects, excluding current
	for (const id of queueItems) {
		if (id === currentProjectId) continue
		if (remainingIds.length >= MAX_SKIP_ATTEMPTS) break
		remainingIds.push(id)
	}

	if (remainingIds.length > 0) {
		const results = await batchCheckLocksWithMetadata(remainingIds)

		let skippedCount = 0
		for (const id of remainingIds) {
			const result = results.get(id)
			// Treat as unlocked if: not locked, OR expired, OR it's our own lock
			if (!result?.locked || result?.expired || result?.isOwnLock) {
				// Found unlocked - skip the locked ones before it
				if (skippedCount > 0) {
					addNotification({
						title: 'Skipped locked projects',
						text: `Skipped ${skippedCount} project(s) being moderated by others.`,
						type: 'info',
					})
				}

				// Navigate to canonical URL if we have metadata
				if (result?.slug && result?.projectType) {
					const urlType = getProjectTypeForUrlShorthand(result.projectType, [], tags.value)
					navigateTo({
						path: `/${urlType}/${result.slug}`,
						state: { showChecklist: true },
					})
				} else {
					// Fallback: use project ID
					navigateTo({
						name: 'type-id',
						params: { type: 'project', id },
						state: { showChecklist: true },
					})
				}
				return
			}
			moderationStore.completeCurrentProject(id, 'skipped')
			skippedCount++
		}

		// All checked were locked
		debug('[skipToNextProject] All projects were locked, skippedCount:', skippedCount)
		addNotification({
			title: 'All projects locked',
			text: 'All remaining projects are currently being moderated by others.',
			type: 'warning',
		})
	}

	debug('[skipToNextProject] Emitting exit')
	emit('exit')
}

function resetProgress() {
	currentStage.value = findFirstValidStage()
	actionStates.value = {}
	textInputValues.value = {}

	done.value = false
	generatedMessage.value = false
	message.value = ''
	loadingMessage.value = false

	localStorage.removeItem(`modpack-permissions-${projectV2.value.id}`)
	localStorage.removeItem(`modpack-permissions-index-${projectV2.value.id}`)

	sessionStorage.removeItem(`modpack-permissions-data-${projectV2.value.id}`)
	sessionStorage.removeItem(`modpack-permissions-permanent-no-${projectV2.value.id}`)
	sessionStorage.removeItem(`modpack-permissions-updated-${projectV2.value.id}`)

	modpackPermissionsComplete.value = false
	modpackJudgements.value = {}

	initializeAllStages()
}

function findFirstValidStage(): number {
	for (let i = 0; i < checklist.length; i++) {
		if (shouldShowStageIndex(i)) {
			return i
		}
	}
	return 0
}

const currentStageObj = computed(() => checklist[currentStage.value])
const currentStage = useLocalStorage(`moderation-stage-${projectV2.value.slug}`, () =>
	findFirstValidStage(),
)

const stageTextExpanded = computedAsync(async () => {
	const stageIndex = currentStage.value
	const stage = checklist[stageIndex]
	if (stage.text) {
		return renderHighlightedString(
			expandVariables(
				await stage.text(projectV2.value, projectV3.value),
				projectV2.value,
				projectV3.value,
				variables.value,
			),
		)
	}
	return null
}, null)

interface ActionState {
	selected: boolean
	value?: any
}

const persistedActionStates = useLocalStorage(
	`moderation-actions-${projectV2.value.slug}`,
	{},
	{
		serializer: {
			read: (v: any) => (v ? deserializeActionStates(v) : {}),
			write: (v: any) => serializeActionStates(v),
		},
	},
)

const router = useRouter()

const persistedTextInputs = useLocalStorage(
	`moderation-inputs-${projectV2.value.slug}`,
	{} as Record<string, string>,
)

const actionStates = ref<Record<string, ActionState>>(persistedActionStates.value)
const textInputValues = ref<Record<string, string>>(persistedTextInputs.value)

const persistState = () => {
	persistedActionStates.value = actionStates.value
	persistedTextInputs.value = textInputValues.value
}

watch(actionStates, persistState, { deep: true })
watch(textInputValues, persistState, { deep: true })

interface MessagePart {
	weight: number
	content: string
	actionId: string
	stageIndex: number
}

function handleKeybinds(event: KeyboardEvent) {
	const focusedActionIndex = ref<number | null>(null)

	handleKeybind(
		event,
		{
			project: projectV2.value,
			state: {
				currentStage: currentStage.value,
				totalStages: checklist.length,
				currentStageId: currentStageObj.value.id,
				currentStageTitle: currentStageObj.value.title,

				isCollapsed: props.collapsed,
				isDone: done.value,
				hasGeneratedMessage: generatedMessage.value,
				isLoadingMessage: loadingMessage.value,
				isModpackPermissionsStage: isModpackPermissionsStage.value,

				futureProjectCount: moderationStore.queueLength,
				visibleActionsCount: visibleActions.value.length,

				focusedActionIndex: focusedActionIndex.value,
				focusedActionType:
					focusedActionIndex.value !== null
						? (visibleActions.value[focusedActionIndex.value]?.type as any)
						: null,
			},
			actions: {
				tryGoNext: nextStage,
				tryGoBack: previousStage,
				tryGenerateMessage: generateMessage,
				trySkipProject: skipCurrentProject,

				tryToggleCollapse: () => emit('toggleCollapsed'),
				tryResetProgress: resetProgress,
				tryExitModeration: handleExit,

				tryApprove: () => sendMessage(projectV2.value.requested_status ?? 'approved'),
				tryReject: () => sendMessage('rejected'),
				tryWithhold: () => sendMessage('withheld'),
				tryEditMessage: goBackToStages,

				tryToggleAction: (actionIndex: number) => {
					const action = visibleActions.value[actionIndex]
					if (action) {
						toggleAction(action)
					}
				},
				trySelectDropdownOption: (actionIndex: number, optionIndex: number) => {
					const action = visibleActions.value[actionIndex] as DropdownAction
					if (action && action.type === 'dropdown') {
						const visibleOptions = getVisibleDropdownOptions(action)
						if (optionIndex < visibleOptions.length) {
							selectDropdownOption(action, visibleOptions[optionIndex])
						}
					}
				},
				tryToggleChip: (actionIndex: number, chipIndex: number) => {
					const action = visibleActions.value[actionIndex] as MultiSelectChipsAction
					if (action && action.type === 'multi-select-chips') {
						const visibleOptions = getVisibleMultiSelectOptions(action)
						if (chipIndex < visibleOptions.length) {
							toggleChip(action, chipIndex)
						}
					}
				},

				tryFocusNextAction: () => {
					if (visibleActions.value.length === 0) return
					if (focusedActionIndex.value === null) {
						focusedActionIndex.value = 0
					} else {
						focusedActionIndex.value = (focusedActionIndex.value + 1) % visibleActions.value.length
					}
				},
				tryFocusPreviousAction: () => {
					if (visibleActions.value.length === 0) return
					if (focusedActionIndex.value === null) {
						focusedActionIndex.value = visibleActions.value.length - 1
					} else {
						focusedActionIndex.value =
							focusedActionIndex.value === 0
								? visibleActions.value.length - 1
								: focusedActionIndex.value - 1
					}
				},
				tryActivateFocusedAction: () => {
					if (focusedActionIndex.value === null) return
					const action = visibleActions.value[focusedActionIndex.value]
					if (!action) return

					if (
						action.type === 'button' ||
						action.type === 'conditional-button' ||
						action.type === 'toggle'
					) {
						toggleAction(action)
					}
				},
			},
		},
		keybinds,
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
	window.addEventListener('keydown', handleKeybinds)
	document.addEventListener('visibilitychange', handleVisibilityChange)
	notifications.setNotificationLocation('left')

	// Check if project has already been reviewed (not in processing status)
	if (projectV2.value.status !== 'processing') {
		alreadyReviewed.value = true
		return
	}

	// Try to acquire lock
	const result = await moderationStore.acquireLock(projectV2.value.id)

	if (result.success) {
		handleLockAcquired()
	} else if (result.locked_by) {
		// Actually locked by another moderator
		// In queue mode with more projects - auto-skip to next project
		if (moderationStore.isQueueMode && moderationStore.queueLength > 1) {
			addNotification({
				title: 'Project locked',
				text: `Skipped project locked by @${result.locked_by.username}.`,
				type: 'info',
			})
			// skipToNextProject already calls completeCurrentProject
			await skipToNextProject()
			return
		}

		// Single project mode or last in queue - show locked UI
		lockStatus.value = {
			locked: true,
			lockedBy: result.locked_by,
			lockedAt: result.locked_at ? new Date(result.locked_at) : undefined,
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

onUnmounted(() => {
	window.removeEventListener('keydown', handleKeybinds)
	document.removeEventListener('visibilitychange', handleVisibilityChange)
	notifications.setNotificationLocation('right')

	if (lockCheckInterval.value) {
		clearInterval(lockCheckInterval.value)
	}
	clearLockCountdown()

	// Clear prefetch state to prevent memory leaks
	prefetchQueue.value = []
	isPrefetching.value = false
})

function initializeAllStages() {
	checklist.forEach((stage, stageIndex) => {
		initializeStageActions(stage, stageIndex)
	})
}

function initializeCurrentStage() {
	initializeStageActions(currentStageObj.value, currentStage.value)
}

watch(
	currentStage,
	(newIndex, oldIndex) => {
		const stage = checklist[newIndex]
		// only navigate when the stage actually changes (not on initial mount/remount)
		if (oldIndex !== undefined && newIndex !== oldIndex && stage?.navigate) {
			router.push(`/${projectV2.value.project_type}/${projectV2.value.slug}${stage.navigate}`)
		}

		initializeCurrentStage()
	},
	{ immediate: true },
)

function initializeStageActions(stage: Stage, stageIndex: number) {
	stage.actions.forEach((action, index) => {
		const actionId = getActionIdForStage(action, stageIndex, index)
		if (!actionStates.value[actionId]) {
			actionStates.value[actionId] = initializeActionState(action)
		}
	})

	stage.actions.forEach((action) => {
		if (action.enablesActions) {
			action.enablesActions.forEach((enabledAction, index) => {
				const actionId = getActionIdForStage(enabledAction, currentStage.value, index)
				if (!actionStates.value[actionId]) {
					actionStates.value[actionId] = initializeActionState(enabledAction)
				}
			})
		}
	})
}

function getActionId(action: Action, index?: number): string {
	// If index is not provided, find it in the current stage's actions
	if (index === undefined) {
		index = currentStageObj.value.actions.indexOf(action)
	}
	return getActionIdForStage(action, currentStage.value, index)
}

function getActionKey(action: Action): string {
	// Find the actual index of this action in the current stage's actions array
	const index = currentStageObj.value.actions.indexOf(action)
	return `${currentStage.value}-${index}-${getActionId(action, index)}`
}

const visibleActions = computed(() => {
	const selectedActionIds = Object.entries(actionStates.value)
		.filter(([_, state]) => state.selected)
		.map(([id]) => id)

	const allActions: Action[] = []
	const actionSources = new Map<Action, { enabledBy?: Action; actionIndex?: number }>()

	currentStageObj.value.actions.forEach((action, actionIndex) => {
		if (shouldShowAction(action)) {
			allActions.push(action)
			actionSources.set(action, { actionIndex })

			if (action.enablesActions) {
				action.enablesActions.forEach((enabledAction) => {
					if (shouldShowAction(enabledAction)) {
						allActions.push(enabledAction)
						actionSources.set(enabledAction, { enabledBy: action, actionIndex })
					}
				})
			}
		}
	})

	return allActions.filter((action) => {
		const source = actionSources.get(action)

		if (source?.enabledBy) {
			const enablerId = getActionId(source.enabledBy, source.actionIndex)
			if (!selectedActionIds.includes(enablerId)) {
				return false
			}
		}

		const disabledByOthers = currentStageObj.value.actions.some((otherAction, otherIndex) => {
			const otherId = getActionId(otherAction, otherIndex)
			return (
				selectedActionIds.includes(otherId) &&
				otherAction.disablesActions?.includes(
					action.id || `action-${currentStage.value}-${source?.actionIndex}`,
				)
			)
		})

		return !disabledByOthers
	})
})

const buttonActions = computed(() =>
	visibleActions.value.filter(
		(action) => action.type === 'button' || action.type === 'conditional-button',
	),
)

const toggleActions = computed(() =>
	visibleActions.value.filter((action) => action.type === 'toggle'),
)

const dropdownActions = computed(() =>
	visibleActions.value.filter((action) => action.type === 'dropdown'),
)

const multiSelectActions = computed(() =>
	visibleActions.value.filter((action) => action.type === 'multi-select-chips'),
)

function getDropdownValue(action: DropdownAction) {
	const actionIndex = currentStageObj.value.actions.indexOf(action)
	const actionId = getActionId(action, actionIndex)
	const visibleOptions = getVisibleDropdownOptions(action)
	const currentValue = actionStates.value[actionId]?.value ?? action.defaultOption ?? 0

	const allOptions = action.options
	const storedOption = allOptions[currentValue]

	if (storedOption && visibleOptions.includes(storedOption)) {
		return storedOption
	}

	return visibleOptions[0] || null
}

function isActionSelected(action: Action): boolean {
	const actionIndex = currentStageObj.value.actions.indexOf(action)
	const actionId = getActionId(action, actionIndex)
	return actionStates.value[actionId]?.selected || false
}

function toggleAction(action: Action) {
	const actionIndex = currentStageObj.value.actions.indexOf(action)
	const actionId = getActionId(action, actionIndex)
	const state = actionStates.value[actionId]
	if (state) {
		state.selected = !state.selected
		persistState()
	}
}

function selectDropdownOption(action: DropdownAction, selected: any) {
	const actionIndex = currentStageObj.value.actions.indexOf(action)
	const actionId = getActionId(action, actionIndex)
	const state = actionStates.value[actionId]
	if (state && selected !== undefined && selected !== null) {
		const optionIndex = action.options.findIndex(
			(opt) => opt === selected || (opt?.label && selected?.label && opt.label === selected.label),
		)

		if (optionIndex !== -1) {
			state.value = optionIndex
			state.selected = true
			persistState()
		}
	}
}

function isChipSelected(action: MultiSelectChipsAction, optionIndex: number): boolean {
	const actionIndex = currentStageObj.value.actions.indexOf(action)
	const actionId = getActionId(action, actionIndex)
	const selectedSet = actionStates.value[actionId]?.value as Set<number> | undefined

	const visibleOptions = getVisibleMultiSelectOptions(action)
	const visibleOption = visibleOptions[optionIndex]
	const originalIndex = action.options.findIndex((opt) => opt === visibleOption)

	return selectedSet?.has(originalIndex) || false
}

function toggleChip(action: MultiSelectChipsAction, optionIndex: number) {
	const actionIndex = currentStageObj.value.actions.indexOf(action)
	const actionId = getActionId(action, actionIndex)
	const state = actionStates.value[actionId]
	if (state && state.value instanceof Set) {
		const visibleOptions = getVisibleMultiSelectOptions(action)
		const visibleOption = visibleOptions[optionIndex]
		const originalIndex = action.options.findIndex((opt) => opt === visibleOption)

		if (originalIndex !== -1) {
			if (state.value.has(originalIndex)) {
				state.value.delete(originalIndex)
			} else {
				state.value.add(originalIndex)
			}
			state.selected = state.value.size > 0
			persistState()
		}
	}
}

const isAnyVisibleInputs = computed(() => {
	return visibleActions.value.some((action) => {
		const visibleInputs = getVisibleInputs(action, actionStates.value)
		return visibleInputs.length > 0 && isActionSelected(action)
	})
})

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

async function assembleFullMessage() {
	const messageParts: MessagePart[] = []

	for (let stageIndex = 0; stageIndex < checklist.length; stageIndex++) {
		const stage = checklist[stageIndex]

		await processStageActions(stage, stageIndex, messageParts)
	}

	messageParts.sort((a, b) => a.weight - b.weight)

	const finalMessage = expandVariables(
		messageParts
			.map((part) => part.content)
			.filter((content) => content.trim().length > 0)
			.join('\n\n'),
		projectV2.value,
		projectV3.value,
	)

	return finalMessage
}

async function processStageActions(stage: Stage, stageIndex: number, messageParts: MessagePart[]) {
	const selectedActionIds = Object.entries(actionStates.value)
		.filter(([_, state]) => state.selected)
		.map(([id]) => id)

	for (let actionIndex = 0; actionIndex < stage.actions.length; actionIndex++) {
		const action = stage.actions[actionIndex]
		const actionId = getActionIdForStage(action, stageIndex, actionIndex)
		const state = actionStates.value[actionId]

		if (!state?.selected) continue

		await processAction(action, actionId, state, selectedActionIds, stageIndex, messageParts)

		if (action.enablesActions) {
			for (let enabledIndex = 0; enabledIndex < action.enablesActions.length; enabledIndex++) {
				const enabledAction = action.enablesActions[enabledIndex]
				const enabledActionId = getActionIdForStage(
					enabledAction,
					stageIndex,
					actionIndex,
					enabledIndex,
				)
				const enabledState = actionStates.value[enabledActionId]

				if (enabledState?.selected) {
					await processAction(
						enabledAction,
						enabledActionId,
						enabledState,
						selectedActionIds,
						stageIndex,
						messageParts,
					)
				}
			}
		}
	}
}

async function processAction(
	action: Action,
	actionId: string,
	state: ActionState,
	selectedActionIds: string[],
	stageIndex: number,
	messageParts: MessagePart[],
) {
	const allValidActionIds: string[] = []
	checklist.forEach((stage, stageIdx) => {
		stage.actions.forEach((stageAction, actionIdx) => {
			allValidActionIds.push(getActionIdForStage(stageAction, stageIdx, actionIdx))
			if (stageAction.enablesActions) {
				stageAction.enablesActions.forEach((enabledAction, enabledIdx) => {
					allValidActionIds.push(
						getActionIdForStage(enabledAction, stageIdx, actionIdx, enabledIdx),
					)
				})
			}
		})
	})

	if (action.type === 'button' || action.type === 'toggle') {
		const buttonAction = action as ButtonAction | ToggleAction
		const message = await getActionMessage(buttonAction, selectedActionIds, allValidActionIds)
		if (message) {
			messageParts.push({
				weight: buttonAction.weight,
				content: processMessage(message, action, stageIndex, textInputValues.value),
				actionId,
				stageIndex,
			})
		}
	} else if (action.type === 'conditional-button') {
		const conditionalAction = action as ConditionalButtonAction
		const matchingVariant = findMatchingVariant(
			conditionalAction.messageVariants,
			selectedActionIds,
			allValidActionIds,
			stageIndex,
		)

		let message: string
		let weight: number

		if (matchingVariant) {
			message = (await matchingVariant.message()) as string
			weight = matchingVariant.weight
		} else if (conditionalAction.fallbackMessage) {
			message = (await conditionalAction.fallbackMessage()) as string
			weight = conditionalAction.fallbackWeight ?? 0
		} else {
			return
		}

		messageParts.push({
			weight,
			content: processMessage(message, action, stageIndex, textInputValues.value),
			actionId,
			stageIndex,
		})
	} else if (action.type === 'dropdown') {
		const dropdownAction = action as DropdownAction
		const selectedIndex = state.value ?? 0
		const selectedOption = dropdownAction.options[selectedIndex]

		if (selectedOption && 'message' in selectedOption && 'weight' in selectedOption) {
			const message = (await selectedOption.message()) as string
			messageParts.push({
				weight: selectedOption.weight,
				content: processMessage(message, action, stageIndex, textInputValues.value),
				actionId,
				stageIndex,
			})
		}
	} else if (action.type === 'multi-select-chips') {
		const multiSelectAction = action as MultiSelectChipsAction
		const selectedIndices = state.value as Set<number>

		for (const index of selectedIndices) {
			const option = multiSelectAction.options[index]
			if (option && 'message' in option && 'weight' in option) {
				const message = (await option.message()) as string
				messageParts.push({
					weight: option.weight,
					content: processMessage(message, action, stageIndex, textInputValues.value),
					actionId: `${actionId}-option-${index}`,
					stageIndex,
				})
			}
		}
	}
}

function shouldShowStage(stage: Stage): boolean {
	let hasVisibleActions = false

	for (const a of stage.actions) {
		if (shouldShowAction(a)) {
			hasVisibleActions = true
		}
	}

	if (!hasVisibleActions) {
		return false
	}

	if (typeof stage.shouldShow === 'function') {
		return stage.shouldShow(projectV2.value, projectV3.value)
	}

	return true
}

function shouldShowAction(action: Action): boolean {
	if (typeof action.shouldShow === 'function') {
		return action.shouldShow(projectV2.value)
	}

	return true
}

function getVisibleDropdownOptions(action: DropdownAction) {
	return action.options.filter((option) => {
		if (typeof option.shouldShow === 'function') {
			return option.shouldShow(projectV2.value)
		}
		return true
	})
}

function getVisibleMultiSelectOptions(action: MultiSelectChipsAction) {
	return action.options.filter((option) => {
		if (typeof option.shouldShow === 'function') {
			return option.shouldShow(projectV2.value)
		}
		return true
	})
}

function shouldShowStageIndex(stageIndex: number): boolean {
	return shouldShowStage(checklist[stageIndex])
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

	while (targetStage < checklist.length) {
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
	generatedMessage.value = false
	message.value = ''

	let targetStage = checklist.length - 1
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

		generatedMessage.value = true
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

	try {
		await useBaseFetch(`project/${projectId}`, {
			method: 'PATCH',
			body: {
				status,
			},
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

		if (projectType === 'modpack' && Object.keys(modpackJudgements.value).length > 0) {
			await useBaseFetch(`moderation/project`, {
				internal: true,
				method: 'POST',
				body: modpackJudgements.value,
			})
		}

		const willHaveNext = moderationStore.completeCurrentProject(projectId, 'completed')

		moderationStore.releaseLock(projectId).catch((err) => {
			console.warn('Failed to release lock:', err)
		})

		// Set both states together - hasNextProject MUST be set before done
		// to avoid the race condition where done=true renders with hasNextProject=false
		hasNextProject.value = willHaveNext
		done.value = true
	} catch (error) {
		console.error('Error submitting moderation:', error)
		addNotification({
			title: 'Error submitting moderation',
			text: 'Failed to submit moderation decision. Please try again.',
			type: 'error',
		})
	}
}

async function endChecklist(status?: string) {
	clearProjectLocalStorage()

	if (!hasNextProject.value) {
		await navigateTo({
			name: 'moderation',
			state: {
				confetti: true,
			},
		})

		await nextTick()

		if (moderationStore.currentQueue.total > 1) {
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
		// Use prefetched data if available for instant navigation
		if (!(await navigateToNextUnlockedProject())) {
			// Fallback: batch check remaining projects with metadata
			const remainingIds: string[] = []
			const currentProjectId = projectV2.value?.id
			const queueItems = moderationStore.currentQueue.items

			// Build list of remaining projects, excluding current
			for (const id of queueItems) {
				if (id === currentProjectId) continue
				if (remainingIds.length >= MAX_SKIP_ATTEMPTS) break
				remainingIds.push(id)
			}

			let foundUnlocked = false
			if (remainingIds.length > 0) {
				const results = await batchCheckLocksWithMetadata(remainingIds)

				let skippedCount = 0
				for (const id of remainingIds) {
					const result = results.get(id)
					// Treat as unlocked if: not locked, OR expired, OR it's our own lock
					if (!result?.locked || result?.expired || result?.isOwnLock) {
						// Found unlocked - skip the locked ones before it
						if (skippedCount > 0) {
							addNotification({
								title: 'Skipped locked projects',
								text: `Skipped ${skippedCount} project(s) being moderated by others.`,
								type: 'info',
							})
						}

						// Navigate to canonical URL if we have metadata
						if (result?.slug && result?.projectType) {
							const urlType = getProjectTypeForUrlShorthand(result.projectType, [], tags.value)
							navigateTo({
								path: `/${urlType}/${result.slug}`,
								state: { showChecklist: true },
							})
						} else {
							// Fallback: use project ID
							navigateTo({
								name: 'type-id',
								params: { type: 'project', id },
								state: { showChecklist: true },
							})
						}
						foundUnlocked = true
						break
					}
					moderationStore.completeCurrentProject(id, 'skipped')
					skippedCount++
				}

				// If no unlocked projects found, show notification
				if (!foundUnlocked && skippedCount > 0) {
					addNotification({
						title: 'All projects locked',
						text: 'All remaining projects are currently being moderated by others.',
						type: 'warning',
					})
				}
			}

			// If no unlocked projects found, go back to moderation queue
			if (!foundUnlocked) {
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

	moderationStore.releaseLock(projectId).catch((err) => {
		console.warn('Failed to release lock:', err)
	})

	hasNextProject.value = moderationStore.completeCurrentProject(projectId, 'skipped')

	await endChecklist('skipped')
}

function clearProjectLocalStorage() {
	localStorage.removeItem(`modpack-permissions-${projectV2.value.id}`)
	localStorage.removeItem(`modpack-permissions-index-${projectV2.value.id}`)
	localStorage.removeItem(`moderation-actions-${projectV2.value.slug}`)
	localStorage.removeItem(`moderation-inputs-${projectV2.value.slug}`)
	localStorage.removeItem(`moderation-stage-${projectV2.value.slug}`)

	sessionStorage.removeItem(`modpack-permissions-data-${projectV2.value.id}`)
	sessionStorage.removeItem(`modpack-permissions-permanent-no-${projectV2.value.id}`)
	sessionStorage.removeItem(`modpack-permissions-updated-${projectV2.value.id}`)

	actionStates.value = {}
}

const isLastVisibleStage = computed(() => {
	for (let i = currentStage.value + 1; i < checklist.length; i++) {
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

const stageOptions = computed<OverflowMenuOption[]>(() => {
	const options = checklist
		.map((stage, index) => {
			if (!shouldShowStage(stage)) return null

			return {
				id: String(index),
				action: () => (currentStage.value = index),
				text: stage.id ? kebabToTitleCase(stage.id) : stage.title,
				color: index === currentStage.value && !generatedMessage.value ? 'green' : undefined,
				hoverFilled: true,
				icon: stage.icon ? stage.icon : undefined,
			} as OverflowMenuOption
		})
		.filter((opt): opt is OverflowMenuOption => opt !== null)

	options.push({
		id: 'generate-message',
		action: () => generateMessage(),
		text: 'Generate Message',
		color: generatedMessage.value ? 'green' : undefined,
		hoverFilled: true,
		icon: CheckIcon,
	} as OverflowMenuOption)

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
</style>
