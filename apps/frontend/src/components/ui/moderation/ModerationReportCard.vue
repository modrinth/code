<template>
	<div
		ref="cardElement"
		class="relative overflow-hidden rounded-2xl border border-solid border-surface-4 transition-[transform,opacity] duration-[400ms] ease-in"
		:class="{ 'pointer-events-none translate-x-[120%] opacity-0': isSwipingAway }"
	>
		<div class="border-0 border-b border-solid border-surface-4 bg-bg-raised p-4">
			<div
				class="flex w-full flex-col items-start justify-between gap-3 sm:flex-row sm:items-center sm:gap-0"
			>
				<span class="text-md flex flex-col flex-wrap gap-1 sm:flex-row sm:items-center">
					<span class="flex items-center gap-1">
						<span class="text-secondary">Reported for</span>
						<span class="font-semibold text-contrast">
							{{ formattedReportType }}
						</span>
					</span>
					<span class="flex items-center gap-1">
						<span class="hidden text-secondary sm:inline">by</span>
						<span class="text-secondary sm:hidden">Reporter:</span>
						<ModerationOwnerLink :owner="reporter" />
					</span>
				</span>

				<div class="flex flex-row items-center gap-2 self-end sm:self-auto">
					<span
						v-tooltip="formatDateTime(report.created)"
						class="cursor-help whitespace-nowrap text-sm text-secondary"
					>
						{{ formatRelativeTime(report.created) }}
					</span>
					<div class="flex items-center gap-2">
						<CopyCode v-tooltip="'Copy report ID'" :text="report.id" />
						<CopyLinkButton
							copy-label="Copy report link"
							:url="`https://modrinth.com/moderation/reports/${props.report.id}`"
						/>
						<ButtonLink
							v-tooltip="'Open in new tab'"
							:href="`/moderation/reports/${props.report.id}`"
							target="_blank"
							circular
							icon-only
						>
							<ExternalIcon />
						</ButtonLink>
					</div>
				</div>
			</div>

			<div class="my-4 h-px bg-surface-5" />

			<ModerationItemHeader
				:avatar-url="reportItemAvatarUrl"
				:title="reportItemTitle"
				:title-to="reportItemTitleTo"
				:owner="itemOwner"
				:stats-owner="
					report.item_type === 'user' && report.user
						? {
								kind: 'user',
								id: report.user.id,
								name: report.user.username,
							}
						: null
				"
				:circle="report.item_type === 'user'"
				:pad-transparent-corners="report.item_type === 'shared-instance'"
				:title-button="report.item_type === 'shared-instance'"
				@title-click="openSharedInstanceInspector"
			>
				<template #badges>
					<div
						v-if="report.project?.project_type"
						class="flex items-center gap-1 rounded-full border border-solid border-surface-5 bg-surface-4 px-2.5 py-1"
					>
						<component
							:is="getProjectTypeIcon(report.project.project_type as any)"
							aria-hidden="true"
							class="h-4 w-4"
						/>
						<span class="text-sm font-medium text-secondary">
							{{ formatProjectType(report.project.project_type, true) }}
						</span>
					</div>

					<div
						v-if="report.project?.status"
						class="flex items-center gap-1 rounded-full border border-solid border-surface-5 bg-surface-4 px-2.5 py-1"
					>
						<Badge :type="report.project?.status" class="text-sm" />
					</div>

					<span
						v-if="report.item_type === 'version' && report.version"
						class="text-sm text-secondary"
					>
						{{ report.version.files.find((f) => f.primary)?.filename || 'Unknown Version' }}
					</span>
					<span
						v-if="
							report.item_type === 'shared-instance' &&
							report.shared_instance_version_id !== undefined
						"
						class="text-sm text-secondary"
					>
						Version {{ report.shared_instance_version_id }}
					</span>
					<CopyCode v-if="report.item_type === 'shared-instance'" :text="report.item_id" />
					<span
						v-if="report.item_type === 'shared-instance' && sharedInstanceQuarantined"
						class="bg-orange-highlight inline-flex items-center gap-1 rounded-full border border-solid border-orange px-2.5 py-1 text-sm font-semibold text-orange"
					>
						<LockIcon class="size-4" />
						Quarantined
					</span>
				</template>
				<template #subtitle>
					<span
						v-if="report.item_type === 'user' && report.user?.created"
						v-tooltip="formatDateTime(report.user.created)"
						class="cursor-help text-sm text-secondary"
					>
						Joined {{ formatRelativeTime(report.user.created) }}
					</span>
					<template v-else-if="report.item_type === 'shared-instance' && !itemOwner">
						<Button
							v-if="sharedInstanceError && !sharedInstanceLoading"
							type="outlined"
							size="sm"
							class="w-fit"
							@click="refetchSharedInstanceDetails()"
						>
							<ReceiptTextIcon />
							Retry loading shared instance details
						</Button>
						<span v-else class="flex items-center gap-1.5 text-sm text-secondary">
							<LoaderCircleIcon class="size-4 animate-spin" />
							Loading shared instance details…
						</span>
					</template>
				</template>
			</ModerationItemHeader>
		</div>
		<CollapsibleRegion
			v-model:collapsed="isThreadCollapsed"
			:expand-text="expandText"
			collapse-text="Collapse thread"
			:disabled="disableCollapsing"
		>
			<div class="bg-surface-2 pt-2">
				<ThreadView
					v-if="threadWithReportBody"
					ref="reportThread"
					:thread="threadWithReportBody"
					:quick-replies="reportQuickReplies"
					:quick-reply-context="report"
					:closed="reportClosed"
					@update-thread="updateThread"
				>
					<template #afterMessages>
						<template v-if="report.item_type === 'shared-instance'">
							<div
								v-if="sharedInstanceLoading"
								class="flex items-center gap-2 border-0 border-t border-solid border-surface-4 px-4 py-6 text-secondary"
							>
								<LoaderCircleIcon class="size-5 animate-spin" />
								Loading shared instance details…
							</div>
							<div
								v-else-if="sharedInstanceError"
								class="flex flex-col items-start gap-3 border-0 border-t border-solid border-surface-4 px-4 py-6"
							>
								<div class="flex flex-col gap-1">
									<span class="font-semibold text-contrast">
										Shared instance details could not be loaded
									</span>
									<span class="text-sm text-secondary">{{ sharedInstanceError }}</span>
								</div>
								<Button type="outlined" @click="refetchSharedInstanceDetails()">Try again</Button>
							</div>
							<SharedInstanceReportContext
								v-else-if="sharedInstanceDetails"
								:details="sharedInstanceDetails"
								:reported-version="report.shared_instance_version_id"
							/>
						</template>
					</template>
					<template #closedActions>
						<Button
							v-if="isStaff(auth.user)"
							type="colored"
							color="green"
							class="mt-2 w-full gap-2 sm:w-auto"
							@click="reopenReport()"
						>
							<CheckCircleIcon class="size-4" />
							Reopen report
						</Button>
					</template>
					<template #additionalActions="{ hasReply }">
						<template v-if="isStaff(auth.user)">
							<Button
								v-if="hasReply"
								type="colored"
								color="red"
								class="w-full gap-2 sm:w-auto"
								@click="closeReport(true)"
							>
								<CheckCircleIcon class="size-4" />
								Reply and close
							</Button>
							<Button
								v-else
								type="colored"
								color="red"
								class="w-full gap-2 sm:w-auto"
								@click="closeReport()"
							>
								<CheckCircleIcon class="size-4" />
								Close report
							</Button>
						</template>
					</template>
				</ThreadView>
			</div>
		</CollapsibleRegion>
		<div
			v-if="pendingDismiss"
			:key="dismissAnimationId"
			class="pointer-events-none absolute inset-x-0 bottom-0 h-1.5 overflow-hidden bg-highlight-red"
			aria-hidden="true"
		>
			<div
				class="report-dismiss-progress h-full w-full bg-red"
				:style="{ animationDuration: `${DISMISS_DELAY_MS}ms` }"
			/>
		</div>
		<SharedInstanceInspectorModal
			v-if="report.item_type === 'shared-instance'"
			ref="sharedInstanceInspector"
		/>
	</div>
</template>
<script setup lang="ts">
import {
	CheckCircleIcon,
	ExternalIcon,
	LoaderCircleIcon,
	LockIcon,
	ReceiptTextIcon,
} from '@modrinth/assets'
import { type ExtendedReport, reportQuickReplies } from '@modrinth/moderation'
import {
	Badge,
	Button,
	ButtonLink,
	CollapsibleRegion,
	CopyCode,
	CopyLinkButton,
	formatReportType,
	getProjectTypeIcon,
	injectModrinthClient,
	injectNotificationManager,
	useFormatDateTime,
	useRelativeTime,
	useVIntl,
} from '@modrinth/ui'
import { formatProjectType } from '@modrinth/utils'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { useIntersectionObserver } from '@vueuse/core'
import { computed, onUnmounted, ref, watch } from 'vue'

import { sharedInstanceQueryOptions } from '~/composables/queries/shared-instance'
import { getRequestErrorMessage } from '~/helpers/shared-instance-moderation'
import { isStaff } from '~/helpers/users.js'

import ThreadView from '../thread/ThreadView.vue'
import ModerationItemHeader from './ModerationItemHeader.vue'
import ModerationOwnerLink, { type ModerationOwner } from './ModerationOwnerLink.vue'
import SharedInstanceInspectorModal from './shared-instance-inspector/index.vue'
import SharedInstanceReportContext from './SharedInstanceReportContext.vue'

const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const auth = useAuthState()

const DISMISS_DELAY_MS = 3000
const SWIPE_DURATION_MS = 400
const SHARED_INSTANCE_PRELOAD_MARGIN = '400px'

const props = defineProps<{
	report: ExtendedReport
	collapsed: boolean
	disableCollapsing?: boolean
	dismissAfterClose?: boolean
}>()

const emit = defineEmits<{
	dismiss: []
}>()

const reportThread = ref<{
	setReplyContent: (content: string) => void
	sendReply: (privateMessage?: boolean) => Promise<void>
} | null>(null)
const isThreadCollapsed = ref(props.collapsed)
const sharedInstanceInspector = ref<InstanceType<typeof SharedInstanceInspectorModal> | null>(null)
const cardElement = ref<HTMLElement | null>(null)
const sharedInstanceRequested = ref(false)

const { stop: stopObservingCard } = useIntersectionObserver(
	() => (props.report.item_type === 'shared-instance' ? cardElement.value : null),
	([entry]) => {
		if (!entry?.isIntersecting) return
		sharedInstanceRequested.value = true
		stopObservingCard()
	},
	{ rootMargin: SHARED_INSTANCE_PRELOAD_MARGIN },
)

const {
	data: sharedInstanceDetails,
	error: sharedInstanceQueryError,
	isFetching: sharedInstanceFetching,
	refetch: refetchSharedInstanceDetails,
} = useQuery(
	computed(() => ({
		...sharedInstanceQueryOptions.moderationContext(props.report.item_id, client, queryClient),
		enabled: props.report.item_type === 'shared-instance' && sharedInstanceRequested.value,
	})),
)
const sharedInstanceLoading = computed(
	() => sharedInstanceFetching.value && !sharedInstanceDetails.value,
)
const sharedInstanceError = computed(() =>
	sharedInstanceQueryError.value
		? getRequestErrorMessage(
				sharedInstanceQueryError.value,
				'Failed to load shared instance details.',
			)
		: null,
)

function openSharedInstanceInspector() {
	if (props.report.item_type !== 'shared-instance') return
	sharedInstanceInspector.value?.show(props.report.item_id, {
		version: props.report.shared_instance_version_id,
		reportedVersions:
			props.report.shared_instance_version_id !== undefined
				? [props.report.shared_instance_version_id]
				: [],
	})
}

const closedOverride = ref<boolean | null>(null)
const pendingDismiss = ref(false)
const isSwipingAway = ref(false)
const dismissAnimationId = ref(0)
const thread = ref(props.report.thread)
let dismissTimeout: ReturnType<typeof setTimeout> | null = null
let swipeTimeout: ReturnType<typeof setTimeout> | null = null

const reportClosed = computed(() => closedOverride.value ?? props.report.closed)

watch(
	() => props.report.thread,
	(value) => {
		thread.value = value
	},
)

watch(
	() => props.report.closed,
	(closed) => {
		if (closedOverride.value === closed) {
			closedOverride.value = null
		}
	},
)
const sharedInstanceQuarantined = computed(
	() =>
		sharedInstanceDetails.value?.quarantine ?? props.report.shared_instance?.quarantine ?? false,
)

const threadWithReportBody = computed(() => {
	if (!thread.value) return null

	const reportBodyMessage = {
		id: `report-body-${props.report.id}`,
		author_id: props.report.reporter_user.id,
		body: {
			type: 'text' as const,
			body: props.report.body || 'Report opened.',
			private: false,
			replying_to: null,
			associated_images: [],
		},
		created: props.report.created,
		hide_identity: false,
	}

	return {
		...thread.value,
		messages: [reportBodyMessage, ...thread.value.messages],
		members: [props.report.reporter_user, ...thread.value.members],
	}
})

const remainingMessageCount = computed(() => {
	if (!thread.value?.messages) return 0
	return thread.value.messages.length
})

const expandText = computed(() => {
	if (remainingMessageCount.value === 0) return 'Expand'
	if (remainingMessageCount.value === 1) return 'Show 1 more message'
	return `Show ${remainingMessageCount.value} more messages`
})

const reporter = computed<ModerationOwner>(() => ({
	kind: 'user',
	id: props.report.reporter_user.id,
	name: props.report.reporter_user.username,
	icon_url: props.report.reporter_user.avatar_url,
}))

async function closeReport(reply = false) {
	if (reply && reportThread.value) {
		await reportThread.value.sendReply()
	}

	try {
		await useBaseFetch(`report/${props.report.id}`, {
			method: 'PATCH',
			body: {
				closed: true,
			},
		})
		await refreshThread()
		closedOverride.value = true
		startDismissCountdown()
		void refreshReportQuery()
	} catch (err: any) {
		addNotification({
			title: 'Error closing report',
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
}

async function reopenReport() {
	cancelDismissCountdown()

	try {
		await useBaseFetch(`report/${props.report.id}`, {
			method: 'PATCH',
			body: {
				closed: false,
			},
		})
		await refreshThread()
		closedOverride.value = false
		void refreshReportQuery()
	} catch (err: any) {
		addNotification({
			title: 'Error reopening report',
			text: err.data ? err.data.description : err,
			type: 'error',
		})
		if (reportClosed.value) {
			startDismissCountdown()
		}
	}
}

function cancelDismissCountdown() {
	pendingDismiss.value = false
	if (dismissTimeout !== null) {
		clearTimeout(dismissTimeout)
		dismissTimeout = null
	}
}

function startDismissCountdown() {
	if (!props.dismissAfterClose || isSwipingAway.value) return

	cancelDismissCountdown()
	dismissAnimationId.value += 1
	pendingDismiss.value = true
	dismissTimeout = setTimeout(() => {
		dismissTimeout = null
		swipeAway()
	}, DISMISS_DELAY_MS)
}

function swipeAway() {
	if (isSwipingAway.value) return

	isSwipingAway.value = true
	swipeTimeout = setTimeout(() => {
		swipeTimeout = null
		emit('dismiss')
	}, SWIPE_DURATION_MS)
}

onUnmounted(() => {
	cancelDismissCountdown()
	if (swipeTimeout !== null) {
		clearTimeout(swipeTimeout)
		swipeTimeout = null
	}
})

const formatRelativeTime = useRelativeTime()
const formatDateTime = useFormatDateTime({
	timeStyle: 'short',
	dateStyle: 'long',
})

async function refreshThread() {
	const threadId = thread.value?.id ?? props.report.thread?.id ?? props.report.thread_id
	if (!threadId) return

	const nextThread = await useBaseFetch(`thread/${threadId}`)
	updateThread(nextThread)
}

function updateThread(newThread: any) {
	thread.value = newThread
	if (props.report.thread) {
		Object.assign(props.report.thread, newThread)
	}
}

function refreshReportQuery() {
	return queryClient.invalidateQueries({ queryKey: ['report', props.report.id] })
}

const reportItemAvatarUrl = computed(() => {
	switch (props.report.item_type) {
		case 'project':
		case 'version':
			return props.report.project?.icon_url || ''
		case 'user':
			return props.report.user?.avatar_url || ''
		case 'shared-instance':
			return sharedInstanceDetails.value?.icon_url || props.report.shared_instance?.icon || ''
		default:
			return undefined
	}
})

const reportItemTitle = computed(() => {
	if (props.report.item_type === 'user') return props.report.user?.username || 'Unknown User'
	if (props.report.item_type === 'shared-instance') {
		return (
			sharedInstanceDetails.value?.name || props.report.shared_instance?.name || 'Shared instance'
		)
	}

	return props.report.project?.title || 'Unknown Project'
})

const reportItemUrl = computed(() => {
	switch (props.report.item_type) {
		case 'user':
			return `/user/${props.report.user?.username}`
		case 'project':
			return `/${props.report.project?.project_type}/${props.report.project?.slug}`
		case 'version':
			return `/${props.report.project?.project_type}/${props.report.project?.slug}/version/${props.report.version?.id}`
		case 'shared-instance':
			return ''
		default:
			return `/${props.report.item_type}/${props.report.id}`
	}
})

const reportItemTitleTo = computed(() =>
	props.report.item_type === 'shared-instance' ? undefined : reportItemUrl.value,
)

const itemOwner = computed((): ModerationOwner | null => {
	if (props.report.item_type === 'user') return null

	if (props.report.item_type === 'shared-instance') {
		const owner = sharedInstanceDetails.value?.owner
		if (!owner) return null

		return {
			kind: 'user',
			id: owner.id,
			name: owner.username,
			icon_url: owner.avatar_url ?? null,
		}
	}

	const target = props.report.target
	if (!target) return null

	return {
		kind: target.type,
		id: target.id,
		name: target.name,
		icon_url: target.avatar_url ?? null,
	}
})

const formattedReportType = computed(() =>
	formatReportType(formatMessage, props.report.report_type),
)
</script>

<style scoped>
.report-dismiss-progress {
	transform-origin: left center;
	animation-name: report-dismiss-fill;
	animation-timing-function: linear;
	animation-fill-mode: forwards;
}

@keyframes report-dismiss-fill {
	from {
		transform: scaleX(0);
	}
	to {
		transform: scaleX(1);
	}
}
</style>
