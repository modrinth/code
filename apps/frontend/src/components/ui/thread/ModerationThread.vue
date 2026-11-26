<template>
	<div :class="{ 'flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden': compact }">
		<DefineSendAction>
			<Button
				v-if="sortedMessages.length > 0 || (compact && isStaff(auth.user))"
				:type="!compact || useBrandSendButton ? 'colored' : 'base'"
				color="brand"
				:disabled="!replyBody.trim() || isLoading"
				@click="
					isApproved(project) && !isStaff(auth.user)
						? openReplyModal()
						: runBlockingAction('reply', () => sendReply())
				"
			>
				<SpinnerIcon v-if="loadingAction === 'reply'" class="animate-spin" aria-hidden="true" />
				<ReplyIcon v-else aria-hidden="true" />
				{{ formatMessage(messages.actionReply) }}
			</Button>
			<Button
				v-else
				:type="useBrandSendButton ? 'colored' : 'base'"
				color="brand"
				:disabled="!replyBody.trim() || isLoading"
				@click="
					isApproved(project) && !isStaff(auth.user)
						? openReplyModal()
						: runBlockingAction('send', () => sendReply())
				"
			>
				<SpinnerIcon v-if="loadingAction === 'send'" class="animate-spin" aria-hidden="true" />
				<SendIcon v-else aria-hidden="true" />
				{{ formatMessage(messages.actionSend) }}
			</Button>
		</DefineSendAction>
		<DefinePrivateNoteAction>
			<Button
				:disabled="!replyBody.trim() || isLoading"
				@click="runBlockingAction('private-note', () => sendReply({ privateMessage: true }))"
			>
				<SpinnerIcon
					v-if="loadingAction === 'private-note'"
					class="animate-spin"
					aria-hidden="true"
				/>
				<StickyNotePlusIcon v-else aria-hidden="true" />
				{{ formatMessage(compact ? messages.actionPrivateNote : messages.actionAddPrivateNote) }}
			</Button>
		</DefinePrivateNoteAction>
		<NewModal ref="modalReply" :header="formatMessage(messages.replyModalHeader)">
			<div class="flex max-w-[45rem] flex-col gap-3">
				<p class="m-0">{{ formatMessage(messages.replyModalDescription) }}</p>
				<p class="m-0">
					<IntlFormatted :message-id="messages.replyModalHelpCenterNote">
						<template #help-center-link="{ children }">
							<a class="text-link" href="https://support.modrinth.com" target="_blank">
								<component :is="() => children" />
							</a>
						</template>
					</IntlFormatted>
				</p>
				<Checkbox
					v-model="replyConfirmation"
					:description="formatMessage(messages.replyModalConfirmationDescription)"
				>
					{{ formatMessage(messages.replyModalConfirmationLabel) }}
				</Checkbox>
				<div class="flex flex-wrap items-center justify-end gap-2">
					<Button type="outlined" @click="modalReply.hide()">
						<XIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.cancelButton) }}
					</Button>
					<Button
						type="colored"
						color="brand"
						:disabled="!replyConfirmation || isLoading"
						@click="runBlockingAction('reply-modal', () => sendReplyFromModal())"
					>
						<SpinnerIcon
							v-if="loadingAction === 'reply-modal'"
							class="animate-spin"
							aria-hidden="true"
						/>
						<ReplyIcon v-else aria-hidden="true" />
						{{ formatMessage(messages.actionReplyToThread) }}
					</Button>
				</div>
			</div>
		</NewModal>
		<div v-if="flags.showThreadIds" class="mx-4 mb-3 font-semibold">
			Thread ID:
			<CopyCode :text="thread.id" />
		</div>
		<div
			v-bind="$attrs"
			class="flex flex-col"
			:class="{ 'min-h-0 min-w-0 flex-1 overflow-hidden': compact }"
		>
			<div
				v-if="hasTimelineEntries && $slots['thread-help']"
				class="shrink-0 border-0 border-b border-solid border-divider px-4 py-3 leading-tight text-secondary"
			>
				<slot name="thread-help" />
			</div>
			<div
				v-if="hasTimelineEntries"
				ref="messageViewport"
				class="flex flex-col pt-2"
				:class="{
					'min-h-0 min-w-0 flex-1 overflow-y-auto overflow-x-hidden overscroll-y-contain [overflow-wrap:anywhere]':
						compact,
				}"
			>
				<ThreadTimeline
					:project-owner-id="projectOwnerId"
					:messages="thread.messages"
					:issues="thread.issues"
					:members="members"
					:report="report"
					:auth="auth"
					raised
					image-previews
					@update-thread="() => updateThreadLocal()"
					@open-image="openImage"
				/>
			</div>
			<div
				v-else-if="compact"
				class="flex min-h-0 flex-1 flex-col items-center justify-center gap-2 overflow-y-auto overscroll-y-contain px-6 py-12 text-center"
			>
				<div
					class="mb-1 flex size-14 items-center justify-center rounded-2xl bg-surface-2 text-secondary"
				>
					<MessagesSquareIcon class="size-7" />
				</div>
				<p class="m-0 text-lg font-semibold leading-relaxed text-contrast">No messages yet.</p>
				<p v-if="$slots['thread-help']" class="m-0 max-w-80 leading-relaxed text-secondary">
					<slot name="thread-help" />
				</p>
			</div>
			<div v-if="report && report.closed" class="m-4 mt-2 flex flex-col gap-4">
				<p class="m-0">{{ formatMessage(messages.closedThreadDescription) }}</p>
				<Button
					v-if="isStaff(auth.user)"
					:disabled="isLoading"
					class="w-fit"
					@click="runBlockingAction('reopen', () => reopenReport())"
				>
					<SpinnerIcon v-if="loadingAction === 'reopen'" class="animate-spin" aria-hidden="true" />
					<CheckCircleIcon v-else aria-hidden="true" />
					{{ formatMessage(messages.actionReopenThread) }}
				</Button>
			</div>
			<template v-else-if="!report || !report.closed">
				<MarkdownEditor
					ref="replyEditor"
					v-model="replyBody"
					:class="
						compact
							? 'mt-auto shrink-0 !grow-0 border-0 border-t border-solid border-divider !px-5 !pt-5 [&_.cm-content]:!min-h-[6lh] [&_.cm-editor]:text-sm [&_.cm-scroller]:!max-h-[6lh] [&_.cm-scroller]:![font-family:var(--font-standard)]'
							: 'mx-4 mt-2 !w-auto'
					"
					:hide-markdown-hint="compact"
					:disabled="isLoading"
					:placeholder="
						formatMessage(
							sortedMessages.length > 0
								? messages.replyEditorPlaceholderReply
								: messages.replyEditorPlaceholderSend,
						)
					"
					:on-image-upload="onUploadImage"
				>
					<template v-if="compact" #toolbar />
					<template #preview="{ value }">
						<IssueMessage :message="value" highlighted />
					</template>
					<template #footer="{ openImageModal }">
						<div v-if="replyFacets.length" class="mx-4 mt-3 flex flex-col gap-2">
							<p class="m-0 text-sm text-secondary">
								{{ formatMessage(messages.replyAddresses) }}
							</p>
							<Checkbox
								v-for="facet in replyFacets"
								:key="facet.id"
								:model-value="selectedReplyFacetIds.includes(facet.id)"
								:disabled="isLoading"
								@update:model-value="selectReplyFacet(facet.id, $event)"
							>
								{{ facet.label }}
							</Checkbox>
						</div>
						<div
							v-if="compact"
							class="flex shrink-0 items-center justify-between gap-2 p-[18px] pb-4 pt-1.5"
						>
							<IconButton
								v-tooltip="formatMessage(messages.actionAttachImage)"
								:label="formatMessage(messages.actionAttachImage)"
								:disabled="isLoading"
								@click="openImageModal()"
							>
								<ImageIcon aria-hidden="true" />
							</IconButton>
							<div class="flex flex-wrap items-center justify-end gap-2">
								<TeleportOverflowMenu
									v-if="isStaff(auth.user) && moderationMenuOptions.length"
									:label="formatMessage(commonMessages.moreOptionsButton)"
									:options="moderationMenuOptions"
									:disabled="isLoading"
									:circular="false"
									placement="top-end"
								>
									<MoreHorizontalIcon aria-hidden="true" />
								</TeleportOverflowMenu>
								<PrivateNoteAction v-if="isStaff(auth.user)" />
								<SendAction />
							</div>
						</div>
					</template>
				</MarkdownEditor>
				<div
					v-if="!compact"
					class="flex shrink-0 flex-wrap items-center justify-between gap-4 p-4 pt-2"
				>
					<div class="flex flex-wrap items-center gap-2">
						<SendAction />
						<PrivateNoteAction v-if="isStaff(auth.user)" />
					</div>
					<div class="flex flex-wrap items-center gap-2">
						<template v-if="report">
							<Button
								v-if="isStaff(auth.user) && replyBody"
								type="colored"
								color="red"
								:disabled="isLoading"
								@click="runBlockingAction('close-with-reply', () => closeReport(true))"
							>
								<SpinnerIcon
									v-if="loadingAction === 'close-with-reply'"
									class="animate-spin"
									aria-hidden="true"
								/>
								<CheckCircleIcon v-else aria-hidden="true" />
								{{ formatMessage(messages.actionCloseWithReply) }}
							</Button>
							<Button
								v-else
								:disabled="isLoading"
								@click="runBlockingAction('close', () => closeReport())"
							>
								<SpinnerIcon
									v-if="loadingAction === 'close'"
									class="animate-spin"
									aria-hidden="true"
								/>
								<CheckCircleIcon v-else aria-hidden="true" />
								{{ formatMessage(messages.actionCloseThread) }}
							</Button>
						</template>
					</div>
				</div>
			</template>
		</div>
		<ImageViewerEditor
			ref="imageViewer"
			:items="imageItems"
			editor="disabled"
			@hide="restoreImageFocus"
		>
			<template #actions="{ item }">
				<ButtonLink
					v-tooltip="formatMessage(messages.openImageExternally)"
					type="quiet"
					class="!w-9 !rounded-full !p-0"
					:aria-label="formatMessage(messages.openImageExternally)"
					:href="item.src"
					target="_blank"
				>
					<ExternalIcon aria-hidden="true" />
				</ButtonLink>
			</template>
		</ImageViewerEditor>
	</div>
</template>

<script setup>
import {
	CheckCircleIcon,
	ExternalIcon,
	ImageIcon,
	MessagesSquareIcon,
	MoreHorizontalIcon,
	ReplyIcon,
	SendIcon,
	SpinnerIcon,
	StickyNotePlusIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Button,
	ButtonLink,
	Checkbox,
	commonMessages,
	CopyCode,
	defineMessages,
	IconButton,
	ImageViewerEditor,
	injectModrinthClient,
	injectNotificationManager,
	IntlFormatted,
	MarkdownEditor,
	NewModal,
	TeleportOverflowMenu,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { createReusableTemplate } from '@vueuse/core'
import { computed, nextTick, onActivated, onMounted, ref, watch } from 'vue'

import IssueMessage from '~/components/ui/project-issue-card/issue-message.vue'
import ThreadTimeline from '~/components/ui/thread/ThreadTimeline.vue'
import { useImageUpload } from '~/composables/image-upload.ts'
import { isApproved } from '~/helpers/projects.js'
import { sendThreadReply } from '~/helpers/thread-issues'
import { isStaff } from '~/helpers/users.js'

const client = injectModrinthClient()
const queryClient = useQueryClient()
const route = useRoute()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const [DefineSendAction, SendAction] = createReusableTemplate()
const [DefinePrivateNoteAction, PrivateNoteAction] = createReusableTemplate()

const messages = defineMessages({
	actionAttachImage: {
		id: 'conversation-thread.action.attach-image',
		defaultMessage: 'Attach image',
	},
	replyAddresses: {
		id: 'conversation-thread.reply.addresses',
		defaultMessage: 'This reply addresses the following requested explanations:',
	},
	replyFacet: {
		id: 'conversation-thread.reply.facet',
		defaultMessage: 'Explanation for {issue}',
	},
	unknownIssue: {
		id: 'conversation-thread.reply.unknown-issue',
		defaultMessage: 'Moderation issue',
	},
	changedThread: {
		id: 'conversation-thread.reply.changed-thread',
		defaultMessage: 'The selected thread changed. Open it again before replying.',
	},
	openImageExternally: {
		id: 'conversation-thread.image.open-externally',
		defaultMessage: 'Open externally',
	},
	replyModalHeader: {
		id: 'conversation-thread.reply-modal.header',
		defaultMessage: 'Reply to thread',
	},
	replyModalDescription: {
		id: 'conversation-thread.reply-modal.description',
		defaultMessage:
			'Your project is already approved. As such, the moderation team does not actively monitor this thread. However, they may still see your message if there is a problem with your project.',
	},
	replyModalHelpCenterNote: {
		id: 'conversation-thread.reply-modal.help-center-note',
		defaultMessage:
			'If you need to get in contact with the moderation team, please use the <help-center-link>Modrinth Help Center</help-center-link> and click the blue bubble in the bottom right corner to contact support.',
	},
	replyModalConfirmationDescription: {
		id: 'conversation-thread.reply-modal.confirmation.description',
		defaultMessage: 'Confirm moderators do not actively monitor this',
	},
	replyModalConfirmationLabel: {
		id: 'conversation-thread.reply-modal.confirmation.label',
		defaultMessage: 'I acknowledge that the moderators do not actively monitor the thread.',
	},
	closedThreadDescription: {
		id: 'conversation-thread.closed-thread.description',
		defaultMessage: 'This thread is closed and new messages cannot be sent to it.',
	},
	replyEditorPlaceholderReply: {
		id: 'conversation-thread.reply-editor.placeholder.reply',
		defaultMessage: 'Reply to thread...',
	},
	replyEditorPlaceholderSend: {
		id: 'conversation-thread.reply-editor.placeholder.send',
		defaultMessage: 'Send a message...',
	},
	actionReplyToThread: {
		id: 'conversation-thread.action.reply-to-thread',
		defaultMessage: 'Reply to thread',
	},
	actionReopenThread: {
		id: 'conversation-thread.action.reopen-thread',
		defaultMessage: 'Reopen thread',
	},
	actionReply: {
		id: 'conversation-thread.action.reply',
		defaultMessage: 'Reply',
	},
	actionSend: {
		id: 'conversation-thread.action.send',
		defaultMessage: 'Send',
	},
	actionAddPrivateNote: {
		id: 'conversation-thread.action.add-private-note',
		defaultMessage: 'Add private note',
	},
	actionPrivateNote: {
		id: 'conversation-thread.action.private-note',
		defaultMessage: 'Private note',
	},
	actionCloseWithReply: {
		id: 'conversation-thread.action.close-with-reply',
		defaultMessage: 'Close with reply',
	},
	actionCloseThread: {
		id: 'conversation-thread.action.close-report',
		defaultMessage: 'Close report',
	},
	errorSendingMessage: {
		id: 'conversation-thread.error.sending-message',
		defaultMessage: 'Error sending message',
	},
	errorClosingReport: {
		id: 'conversation-thread.error.closing-report',
		defaultMessage: 'Error closing report',
	},
	errorReopeningReport: {
		id: 'conversation-thread.error.reopening-report',
		defaultMessage: 'Error reopening report',
	},
})

const props = defineProps({
	compact: { type: Boolean, default: false },
	projectOwnerId: {
		type: String,
		default: null,
	},
	thread: {
		type: Object,
		required: true,
	},
	report: {
		type: Object,
		required: false,
		default: null,
	},
	project: {
		type: Object,
		required: false,
		default: null,
	},
	moderationActions: {
		type: Array,
		default: () => [],
	},
	currentMember: {
		type: Object,
		default() {
			return null
		},
	},
	auth: {
		type: Object,
		required: true,
	},
})

const emit = defineEmits(['update-thread'])

const app = useNuxtApp()
const flags = useFeatureFlags()

const members = computed(() => {
	const members = {}
	for (const member of props.thread.members) {
		members[member.id] = member
	}
	return members
})

const replyBody = ref('')
const replyEditor = ref(null)
const useBrandSendButton = computed(() => props.compact && !isStaff(props.auth.user))
const selectedReplyFacetIds = ref([])
const replyFacets = computed(() =>
	props.project && (props.currentMember?.accepted || isStaff(props.auth.user))
		? (props.thread.issues ?? []).flatMap((issue) =>
				issue.facets
					.filter(
						(facet) =>
							facet.verdict === 'open' &&
							facet.what.type === 'acknowledge' &&
							facet.what.value.mode === 'reply',
					)
					.map((facet) => ({
						id: facet.id,
						label: formatMessage(messages.replyFacet, {
							issue: issue.why?.title ?? formatMessage(messages.unknownIssue),
						}),
					})),
			)
		: [],
)
function selectReplyFacet(id, selected) {
	selectedReplyFacetIds.value = selected
		? [...new Set([...selectedReplyFacetIds.value, id])]
		: selectedReplyFacetIds.value.filter((entry) => entry !== id)
}
watch(
	[() => route.query.reply_to_facet, replyFacets, replyEditor],
	async ([id, facets]) => {
		const ids = (Array.isArray(id) ? id : [id]).filter(
			(id) => typeof id === 'string' && facets.some((facet) => facet.id === id),
		)
		if (!ids.length) return
		for (const id of ids) selectReplyFacet(id, true)
		await nextTick()
		await replyEditor.value?.focus?.()
	},
	{ immediate: true, flush: 'post' },
)
watch(
	() => props.thread.id,
	() => {
		replyBody.value = ''
		selectedReplyFacetIds.value = []
		imageIDs.value = []
	},
)

const imageViewer = ref(null)
const imageItems = ref([])
let imageTrigger

async function openImage(image) {
	imageTrigger = image.element
	imageItems.value = [{ id: image.src, src: image.src, alt: image.alt }]
	await nextTick()
	imageViewer.value?.show(0)
}

function restoreImageFocus() {
	if (imageTrigger?.isConnected) imageTrigger.focus({ preventScroll: true })
	imageTrigger = undefined
}

watch(
	() => props.thread?.id,
	() => {
		imageTrigger = undefined
		imageViewer.value?.hide()
		imageItems.value = []
	},
)

const sortedMessages = computed(() => {
	if (props.thread !== null) {
		return props.thread.messages
			.slice()
			.sort((a, b) => app.$dayjs(a.created) - app.$dayjs(b.created))
	}
	return []
})
const hasTimelineEntries = computed(
	() =>
		sortedMessages.value.length > 0 || (isStaff(props.auth.user) && !!props.thread.issues?.length),
)

const messageViewport = ref(null)
async function scrollToLatestMessage() {
	if (!props.compact) return
	await nextTick()
	const viewport = messageViewport.value
	if (viewport) viewport.scrollTop = viewport.scrollHeight
}
onMounted(scrollToLatestMessage)
onActivated(scrollToLatestMessage)
watch(() => props.thread.messages.length, scrollToLatestMessage, { flush: 'post' })

const modalReply = ref(null)

const loadingAction = ref(null)
const isLoading = computed(() => loadingAction.value !== null)
const moderationMenuOptions = computed(() =>
	props.moderationActions.map(({ labelWithReply, action, ...option }) => {
		const withReply = Boolean(labelWithReply && replyBody.value.trim())
		return {
			...option,
			label: withReply ? labelWithReply : option.label,
			action: () => runBlockingAction(option.id, () => applyModerationAction(action, withReply)),
		}
	}),
)

async function applyModerationAction(action, withReply) {
	if (!withReply) return await action()

	const reviewReason = replyBody.value.trim()
	if (!reviewReason) return
	const threadId = props.thread.id
	const projectId = props.project?.id
	if ((await action(reviewReason)) === false) return
	if (props.thread.id === threadId && props.project?.id === projectId) await sendReply()
}

async function runBlockingAction(actionId, action) {
	if (loadingAction.value !== null) {
		return
	}
	loadingAction.value = actionId
	try {
		await action()
	} finally {
		loadingAction.value = null
	}
}

async function updateThreadLocal(threadId = props.thread.id) {
	const thread = await queryClient.fetchQuery({
		queryKey: ['thread', threadId],
		queryFn: () => client.labrinth.threads_v3.getThread(threadId),
		staleTime: 0,
	})
	if (props.thread.id === threadId) emit('update-thread', thread)
}

const imageIDs = ref([])

async function onUploadImage(file) {
	try {
		const response = await useImageUpload(file, { context: 'thread_message' })

		imageIDs.value.push(response.id)
		imageIDs.value = imageIDs.value.slice(-10)

		return response.url
	} catch (error) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: error instanceof Error ? error.message : String(error),
			type: 'error',
		})
		throw error
	}
}

const replyMutation = useMutation({
	mutationFn: async ({ threadId, projectId, privateMessage }) => {
		const assertCurrent = () => {
			if (props.thread.id !== threadId) throw new Error(formatMessage(messages.changedThread))
		}
		if (replyBody.value.trim()) {
			const reply = {
				threadId,
				body: replyBody.value,
				images: [...imageIDs.value],
				privateMessage,
			}
			const facetIds = privateMessage
				? []
				: (props.thread.issues ?? []).flatMap((issue) =>
						issue.facets.some((facet) => selectedReplyFacetIds.value.includes(facet.id))
							? issue.facets
									.filter(
										(facet) =>
											facet.verdict === 'open' &&
											facet.what.type === 'acknowledge' &&
											(facet.what.value.mode === 'checkbox' ||
												selectedReplyFacetIds.value.includes(facet.id)),
									)
									.map(({ id }) => id)
							: [],
					)
			await sendThreadReply(reply, client, assertCurrent)
			replyBody.value = ''
			imageIDs.value = []
			selectedReplyFacetIds.value = []
			for (const facetId of facetIds) {
				assertCurrent()
				await client.labrinth.threads_v3.user_addressed(facetId)
			}
		}
		assertCurrent()
		await updateThreadLocal(threadId)
		if (projectId)
			await queryClient.invalidateQueries({
				queryKey: ['project', projectId, 'validation'],
			})
		assertCurrent()
		return true
	},
	onError: (error) =>
		addNotification({
			title: formatMessage(messages.errorSendingMessage),
			text: error instanceof Error ? error.message : String(error),
			type: 'error',
		}),
	onSettled: async (_, __, { threadId, projectId }) => {
		await Promise.all([
			queryClient.invalidateQueries({ queryKey: ['thread', threadId] }),
			projectId
				? queryClient.invalidateQueries({
						queryKey: ['project', projectId, 'validation'],
					})
				: Promise.resolve(),
		])
	},
})

async function sendReplyFromModal() {
	if (await sendReply()) modalReply.value.hide()
}

async function sendReply({ privateMessage = false } = {}) {
	return await replyMutation
		.mutateAsync({
			threadId: props.thread.id,
			projectId: props.project?.id,
			privateMessage,
		})
		.catch(() => false)
}

async function closeReport(reply) {
	if (reply) {
		if (!(await sendReply())) return
	}

	try {
		await useBaseFetch(`report/${props.report.id}`, {
			method: 'PATCH',
			body: {
				closed: true,
			},
		})
		await updateThreadLocal()
	} catch (err) {
		addNotification({
			title: formatMessage(messages.errorClosingReport),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
}

async function reopenReport() {
	try {
		await useBaseFetch(`report/${props.report.id}`, {
			method: 'PATCH',
			body: {
				closed: false,
			},
		})
		await updateThreadLocal()
	} catch (err) {
		addNotification({
			title: formatMessage(messages.errorReopeningReport),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
}

const replyConfirmation = ref(false)

function openReplyModal() {
	replyConfirmation.value = false
	modalReply.value.show()
}

defineOptions({
	inheritAttrs: false,
})
</script>
