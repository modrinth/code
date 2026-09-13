<template>
	<div class="flex min-h-0 flex-col">
		<!-- History -->
		<div ref="historyEl" class="min-h-0 flex-1 overflow-y-auto px-1">
			<ConversationThread
				v-if="thread"
				:thread="thread"
				:project="projectV2"
				:current-member="currentMember ?? undefined"
				:auth="auth"
				hide-actions
				@update-thread="
					(thread: Labrinth.Threads.v3.Thread | null | undefined) => {
						updateThread(thread)
						invalidate()
					}
				"
			/>
			<div v-else class="flex items-center gap-2 py-6 text-secondary">
				<SpinnerIcon class="size-4 animate-spin" /> Loading thread…
			</div>
		</div>

		<!-- Merged composer -->
		<div class="shrink-0 border-0 border-t border-solid border-divider bg-surface-2">
			<button
				class="flex w-full items-center gap-2 px-2 py-1.5 text-xs font-semibold text-secondary hover:text-contrast"
				@click="composerOpen = !composerOpen"
			>
				<ChevronUpIcon v-if="!composerOpen" class="size-4" />
				<ChevronDownIcon v-else class="size-4" />
				{{ composerOpen ? 'Hide composer' : composerSummary }}
			</button>

			<div v-show="composerOpen" class="flex flex-col gap-2 px-2 pb-2">
				<div class="flex items-center gap-2">
					<div class="flex gap-1 rounded-md bg-bg p-0.5">
						<button
							v-for="m in MODES"
							:key="m.id"
							class="rounded px-2 py-0.5 text-xs font-semibold"
							:class="
								mode === m.id ? 'bg-surface-2 text-contrast' : 'text-secondary hover:text-contrast'
							"
							@click="mode = m.id"
						>
							{{ m.label }}
						</button>
					</div>
					<button
						v-if="mode !== 'decision' || engine.generatedMessage.value"
						v-tooltip="engine.useSimpleEditor.value ? 'Rich editor' : 'Plain text'"
						class="rounded p-1 text-secondary hover:bg-button-bg hover:text-contrast"
						aria-label="Toggle editor mode"
						@click="engine.useSimpleEditor.value = !engine.useSimpleEditor.value"
					>
						<ToggleLeftIcon v-if="engine.useSimpleEditor.value" class="size-4" />
						<ToggleRightIcon v-else class="size-4" />
					</button>
					<Button
						v-if="mode === 'decision' && engine.generatedMessage.value"
						v-tooltip="'Back to flagged issues'"
						size="xs"
						class="ml-auto"
						@click="engine.message.value = null"
					>
						<UndoIcon />
					</Button>
					<Button
						v-if="mode === 'decision'"
						size="xs"
						:class="{ 'ml-auto': !engine.generatedMessage.value }"
						:disabled="engine.loadingMessage.value"
						@click="engine.generateMessage()"
					>
						<SpinnerIcon v-if="engine.loadingMessage.value" class="animate-spin" />
						<RedoIcon v-else />
						{{ engine.generatedMessage.value ? 'Regenerate' : 'Generate from checklist' }}
					</Button>
				</div>

				<!-- Pre-generation: the flagged-issue chips double as the decision preview. Hover to
				     review every issue ever flagged this project (active + inactive); move the
				     cursor away and inactive ones — and any section left with nothing active — drop
				     out of view. Generating (manually, or automatically the moment a decision is
				     made) swaps this for the editor; "Undo" above brings it back. -->
				<div
					v-if="mode === 'decision' && !engine.generatedMessage.value"
					class="flex flex-col gap-1.5 rounded-md border border-solid border-divider bg-bg p-2"
					@pointerenter="issuesHover = true"
					@pointerleave="issuesHover = false"
				>
					<div class="flex items-center justify-between gap-2">
						<span class="text-[0.7rem] font-semibold uppercase tracking-wide text-secondary"
							>Flagged issues</span
						>
						<div class="flex items-center gap-2">
							<span
								v-if="!issuesHover && flaggedGroups.length > 0"
								class="text-[0.7rem] text-secondary"
								>Hover to review &amp; adjust</span
							>
							<button
								v-if="hasStaleIssues"
								v-tooltip="'Clear inactive flagged issues'"
								class="rounded p-0.5 text-secondary hover:bg-button-bg hover:text-contrast"
								aria-label="Clear inactive flagged issues"
								@click="engine.resetFlaggedIssues()"
							>
								<TrashIcon class="size-3.5" />
							</button>
						</div>
					</div>
					<p v-if="visibleGroups.length === 0" class="m-0 text-xs text-secondary">
						Nothing flagged yet — use the checklist buttons throughout the review, then generate the
						message here.
					</p>
					<div
						v-for="grp in visibleGroups"
						v-else
						:key="grp.stageId"
						class="flex flex-wrap items-center gap-1"
					>
						<button
							class="shrink-0 text-[0.7rem] font-semibold text-secondary hover:text-contrast"
							@click="goToStage(grp.stageId)"
						>
							{{ grp.label }}:
						</button>
						<button
							v-for="node in grp.nodes"
							:key="node.key"
							v-tooltip="chipTooltip(node)"
							class="rounded-full border border-solid px-2 py-0.5 text-xs transition-colors"
							:class="
								node.active
									? 'border-brand bg-brand-highlight font-medium text-contrast'
									: 'border-divider bg-transparent text-secondary hover:border-secondary hover:text-contrast'
							"
							@click="engine.setNodeActive(node.statePath, !node.active)"
						>
							{{ node.label }}
						</button>
					</div>
				</div>

				<template v-else>
					<MarkdownEditor
						v-if="!engine.useSimpleEditor.value"
						v-model="text"
						:max-height="160"
						:placeholder="composerPlaceholder"
						:disabled="false"
						:heading-buttons="false"
						:on-image-upload="engine.onUploadHandler"
					/>
					<Textarea
						v-else
						v-model="text"
						:placeholder="composerPlaceholder"
						autocomplete="off"
						input-class="h-28 font-mono text-sm"
					/>
				</template>

				<div class="flex items-center gap-1.5">
					<template v-if="engine.done.value">
						<Button
							type="colored"
							color="brand"
							size="sm"
							class="flex-1"
							@click="engine.endChecklist()"
						>
							<RightArrowIcon v-if="engine.hasNextProject.value" />
							<CheckIcon v-else />
							{{ engine.hasNextProject.value ? 'Next project' : 'All done' }}
						</Button>
					</template>

					<template v-else-if="mode === 'reply'">
						<Button
							type="colored"
							color="brand"
							size="sm"
							class="flex-1"
							:disabled="!text.trim() || sending"
							@click="sendReply"
						>
							<SpinnerIcon v-if="sending" class="animate-spin" />
							<component :is="replyAs === 'private' ? EyeOffIcon : ReplyIcon" v-else />
							{{ replyAs === 'private' ? 'Add private note' : 'Send reply' }}
						</Button>
						<div class="flex overflow-hidden rounded-md border border-solid border-divider text-xs">
							<button
								v-for="opt in REPLY_AS"
								:key="opt.id"
								class="px-2 py-1 font-semibold"
								:class="
									replyAs === opt.id
										? 'bg-button-bg text-contrast'
										: 'text-secondary hover:text-contrast'
								"
								@click="replyAs = opt.id"
							>
								{{ opt.label }}
							</button>
						</div>
					</template>

					<template v-else>
						<div
							v-if="lockBanner"
							class="flex w-full items-center gap-2 rounded-md bg-bg px-2 py-1.5 text-xs text-orange"
						>
							<LockIcon class="size-3.5 shrink-0" />
							{{ lockBanner }}
							<Button
								type="colored"
								color="orange"
								size="sm"
								class="ml-auto"
								:disabled="engine.loadingModerationDecision.value || engine.loadingMessage.value"
								@click="engine.requestTakeOver"
							>
								<LockOpenIcon />
								{{ engine.alreadyReviewed.value ? 'Review Anyways' : 'Take over' }}
							</Button>
						</div>
						<template v-else>
							<Button
								type="colored"
								color="red"
								size="sm"
								class="flex-1"
								:disabled="engine.loadingModerationDecision.value || engine.loadingMessage.value"
								@click="handleRejectOrWithhold('rejected')"
							>
								<SpinnerIcon
									v-if="
										engine.moderationDecision.value === 'rejected' || engine.loadingMessage.value
									"
									class="animate-spin"
								/>
								<XIcon v-else />
								Reject
							</Button>
							<Button
								type="colored"
								color="orange"
								size="sm"
								class="flex-1"
								:disabled="engine.loadingModerationDecision.value || engine.loadingMessage.value"
								@click="handleRejectOrWithhold('withheld')"
							>
								<SpinnerIcon
									v-if="
										engine.moderationDecision.value === 'withheld' || engine.loadingMessage.value
									"
									class="animate-spin"
								/>
								<EyeOffIcon v-else />
								Withhold
							</Button>
							<Button
								v-tooltip="
									isPostApprovalReview
										? 'Already approved — this posts the message as a thread reply instead of re-approving'
										: undefined
								"
								type="colored"
								:color="isPostApprovalReview ? 'blue' : 'green'"
								size="sm"
								class="flex-1"
								:disabled="
									engine.loadingModerationDecision.value || engine.loadingMessage.value || sending
								"
								@click="handleApproveOrWarn"
							>
								<SpinnerIcon
									v-if="
										sending ||
										engine.moderationDecision.value === engine.approveSendStatus.value ||
										engine.loadingMessage.value
									"
									class="animate-spin"
								/>
								<component :is="isPostApprovalReview ? ReplyIcon : CheckIcon" v-else />
								{{ isPostApprovalReview ? 'Send warning' : 'Approve' }}
							</Button>
						</template>
					</template>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	CheckIcon,
	ChevronDownIcon,
	ChevronUpIcon,
	EyeOffIcon,
	LockIcon,
	LockOpenIcon,
	RedoIcon,
	ReplyIcon,
	RightArrowIcon,
	SpinnerIcon,
	ToggleLeftIcon,
	ToggleRightIcon,
	TrashIcon,
	UndoIcon,
	XIcon,
} from '@modrinth/assets'
import { expandVariables } from '@modrinth/moderation'
import {
	collectMessageNodes,
	evalActiveAction,
	md,
	resolveChildren,
} from '@modrinth/moderation/src/types/node'
import { Button, injectProjectPageContext, MarkdownEditor, Textarea } from '@modrinth/ui'
import type { ProjectStatus } from '@modrinth/utils'
import { renderHighlightedString } from '@modrinth/utils'
import { useQueryClient } from '@tanstack/vue-query'
import { computed, nextTick, ref, watch, watchEffect } from 'vue'

import { injectModerationChecklist } from '~/components/ui/moderation/checklist/checklist-context'
import ConversationThread from '~/components/ui/thread/ConversationThread.vue'

const engine = injectModerationChecklist()
const {
	projectV3: project,
	projectV2,
	thread,
	currentMember,
	invalidate,
} = injectProjectPageContext()
const auth = await useAuth()

const MODES = [
	{ id: 'reply' as const, label: 'Reply' },
	{ id: 'decision' as const, label: 'Decision' },
]
const REPLY_AS = [
	{ id: 'public' as const, label: 'Public' },
	{ id: 'private' as const, label: 'Private' },
]

const mode = ref<'reply' | 'decision'>('decision')
const replyAs = ref<'public' | 'private'>('public')
const sending = ref(false)
const composerOpen = ref(true)
const historyEl = ref<HTMLElement | null>(null)
/** While hovering the flagged-issues block, reveal inactive (previously touched) chips too. */
const issuesHover = ref(false)

/** Every issue ever flagged this project, grouped by stage, with its live active state. */
const flaggedGroups = computed(() => {
	const stageLabel = (id: string) =>
		engine.resolvedStages.value.find((s) => s.id === id)?.label ?? id
	const byStage = new Map<
		string,
		{ key: string; label: string; statePath: string[]; active: boolean; tooltip?: string }[]
	>()
	for (const [key, n] of Object.entries(engine.touchedNodes.value)) {
		if (!byStage.has(n.stageId)) byStage.set(n.stageId, [])
		byStage.get(n.stageId)!.push({
			key,
			label: n.label,
			statePath: n.statePath,
			active: engine.activeNodePaths.value.has(key),
			tooltip: n.tooltip,
		})
	}
	return [...byStage.entries()].map(([stageId, nodes]) => ({
		stageId,
		label: stageLabel(stageId),
		nodes,
	}))
})

/**
 * The message each touched issue would actually contribute — including any sub-inputs typed
 * into it (a reupload's "original project"/"original author" fields, say) — keyed by the same
 * `statePath.join('/')` string `flaggedGroups` uses. Mirrors `ChecklistStageButtons.vue`'s
 * per-stage tooltip computation, just across every stage that has a touched issue instead of one.
 */
const messagePreviewMap = ref(new Map<string, string>())

watchEffect(async () => {
	const stageIds = new Set(flaggedGroups.value.map((g) => g.stageId))
	const newMap = new Map<string, string>()
	await Promise.all(
		[...stageIds].map(async (stageId) => {
			const stage = engine.resolvedStages.value.find((s) => s.id === stageId)
			if (!stage) return
			const stageState = engine.nodeStates.value[stageId] ?? {}
			const nodes = resolveChildren(stage, stageState)
			const actions = collectMessageNodes(nodes, stageState, [stageId])
			await Promise.all(
				actions.map(async (entry) => {
					try {
						const raw = await evalActiveAction(entry, actions, new Set())
						const expanded = expandVariables(raw, projectV2.value, project.value).trim()
						if (expanded) {
							newMap.set(
								entry.statePath.join('/'),
								`<div class="markdown-body moderation-tooltip-markdown">${renderHighlightedString(expanded)}</div>`,
							)
						}
					} catch {
						// Leave unset — falls back to the static tooltip / generic label below.
					}
				}),
			)
		}),
	)
	messagePreviewMap.value = newMap
})

/**
 * What's actually rendered: while hovering, every touched stage/issue shows (active or not) so
 * they can be picked back up. Once the cursor leaves, anything not active drops out — inactive
 * entries disappear, and a section left with nothing active in it disappears entirely.
 */
const visibleGroups = computed(() => {
	if (issuesHover.value) return flaggedGroups.value
	return flaggedGroups.value
		.map((grp) => ({ ...grp, nodes: grp.nodes.filter((n) => n.active) }))
		.filter((grp) => grp.nodes.length > 0)
})

/** Whether there's anything for the "clear inactive flagged issues" button to actually clear. */
const hasStaleIssues = computed(() =>
	flaggedGroups.value.some((grp) => grp.nodes.some((n) => !n.active)),
)

function chipTooltip(node: { key: string; active: boolean; tooltip?: string }) {
	const preview = messagePreviewMap.value.get(node.key)
	if (preview) return { content: preview, html: true }
	return node.tooltip ?? (node.active ? 'Active — click to clear' : 'Click to flag')
}

function goToStage(stageId: string) {
	engine.setStage(stageId)
	engine.focusStage(engine.resolvedStages.value.find((s) => s.id === stageId))
}

/** Generating is a step the moderator can trigger manually, or that happens automatically the
 *  moment they take a decision without having generated (or written) a message yet. */
async function handleDecision(status: Parameters<typeof engine.sendMessage>[0]) {
	if (!engine.generatedMessage.value) await engine.generateMessage()
	await engine.sendMessage(status)
}

/** A project that's *already* approved has nothing left for "Approve" to do — re-approving would
 *  just be a meaningless (possibly erroring) status change — so flagged issues there are instead
 *  issued as a warning, and Reject/Withhold become "missed the deadline" once a warning exists. */
const isPostApprovalReview = computed(() => project.value?.status === 'approved')

/** Distinctive sentence from `post-approval/issue-warning.md` — its presence in a public thread
 *  reply is how a prior warning is detected, since the warning is only ever posted, never stored
 *  as separate state. */
const WARNING_MARKER = 'failure to address these concerns in a timely manner'

const wasPreviouslyWarned = computed(() =>
	(thread.value?.messages ?? []).some(
		(m) =>
			m.body.type === 'text' &&
			!m.body.private &&
			typeof m.body.body === 'string' &&
			m.body.body.includes(WARNING_MARKER),
	),
)

async function handleApproveOrWarn() {
	if (!isPostApprovalReview.value) {
		await handleDecision(engine.approveSendStatus.value)
		return
	}
	if (!engine.generatedMessage.value) await engine.generateMessage()
	if (!engine.message.value?.trim()) return
	sending.value = true
	const ok = await engine.postThreadReply(
		`${engine.message.value}\n\n${await md('checklist/messages/post-approval/issue-warning')({})}`,
	)
	sending.value = false
	if (ok) engine.message.value = null
}

/** Reject/Withhold on an already-approved project: if it was already warned via
 *  `handleApproveOrWarn` above, this is treated as a missed deadline — the (possibly updated)
 *  flagged-issue message is wrapped with the missed-deadline notice and the status change goes
 *  through as normal. Otherwise it's an ordinary decision. */
async function handleRejectOrWithhold(status: ProjectStatus) {
	if (!isPostApprovalReview.value || !wasPreviouslyWarned.value) {
		await handleDecision(status)
		return
	}
	if (!engine.generatedMessage.value) await engine.generateMessage()
	const base = engine.message.value?.trim() ?? ''
	const notice = await md('checklist/messages/post-approval/missed-deadline', () => ({
		STATUS: status,
	}))({})
	engine.message.value = base ? `${base}\n\n${notice}` : notice
	await handleDecision(status)
}

const text = computed({
	get: () => engine.message.value ?? '',
	set: (v: string) => {
		engine.message.value = v === '' ? null : v
	},
})

watch(
	() => engine.generatedMessage.value,
	(has) => {
		if (has) {
			//mode.value = 'decision'
			composerOpen.value = true
		}
	},
)

// Keep the thread scrolled to the newest message as it loads / updates.
watch(
	() => thread.value?.messages?.length,
	async () => {
		await nextTick()
		if (historyEl.value) historyEl.value.scrollTop = historyEl.value.scrollHeight
	},
	{ immediate: true },
)

const composerPlaceholder = computed(() =>
	mode.value === 'reply'
		? replyAs.value === 'private'
			? 'Private note for other moderators…'
			: 'Message the project owner…'
		: 'Generate the decision message from the checklist, or write one.',
)

const composerSummary = computed(() => {
	const t = text.value.trim()
	if (t) return `Draft: ${t.slice(0, 40)}${t.length > 40 ? '…' : ''}`
	return mode.value === 'decision' ? 'Compose decision' : 'Reply to thread'
})

async function sendReply() {
	if (!text.value.trim() || sending.value) return
	sending.value = true
	const ok = await engine.postThreadReply(text.value, replyAs.value === 'private')
	sending.value = false
	if (ok) engine.message.value = null
}

const lockBanner = computed(() => {
	const s = engine.lockStatus.value
	if (engine.isLockedByOther.value && s?.lockedBy?.username) {
		return `Locked by @${s.lockedBy.username}${engine.lockTimeRemaining.value ? ` — ${engine.lockTimeRemaining.value}` : ''}`
	}
	if (engine.alreadyReviewed.value && !engine.reviewedAnyway.value) {
		return 'This project was already moderated.'
	}
	return null
})

const queryClient = useQueryClient()

function updateThread(newThread: Labrinth.Threads.v3.Thread | null | undefined) {
	const threadId = newThread?.id ?? project.value?.thread_id
	if (!threadId) return

	queryClient.setQueryData<Labrinth.Threads.v3.Thread | null | undefined>(
		['thread', threadId],
		newThread,
	)
}
</script>
