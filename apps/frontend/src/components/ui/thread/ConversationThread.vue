<template>
	<div>
		<Modal
			ref="modalSubmit"
			:header="isRejected(project) ? 'Resubmit for review' : 'Submit for review'"
		>
			<div class="modal-submit universal-body">
				<span>
					You're submitting <span class="project-title">{{ project.title }}</span> to be reviewed
					again by the moderators.
				</span>
				<span>
					Make sure you have addressed the comments from the moderation team.
					<span class="known-errors">
						Repeated submissions without addressing the moderators' comments may result in an
						account suspension.
					</span>
				</span>
				<Checkbox
					v-model="submissionConfirmation"
					description="Confirm I have addressed the messages from the moderators"
				>
					I confirm that I have properly addressed the moderators' comments.
				</Checkbox>
				<div class="input-group push-right">
					<ButtonStyled color="orange">
						<button
							:disabled="!submissionConfirmation || isLoading"
							@click="runBlockingAction('resubmit-modal', resubmit)"
						>
							<SpinnerIcon
								v-if="loadingAction === 'resubmit-modal'"
								class="animate-spin"
								aria-hidden="true"
							/>
							<ScaleIcon v-else aria-hidden="true" />
							Resubmit for review
						</button>
					</ButtonStyled>
				</div>
			</div>
		</Modal>
		<Modal ref="modalReply" header="Reply to thread">
			<div class="modal-submit universal-body">
				<span>
					Your project is already approved. As such, the moderation team does not actively monitor
					this thread. However, they may still see your message if there is a problem with your
					project.
				</span>
				<span>
					If you need to get in contact with the moderation team, please use the
					<a class="text-link" href="https://support.modrinth.com" target="_blank">
						Modrinth Help Center
					</a>
					and click the green bubble to contact support.
				</span>
				<Checkbox
					v-model="replyConfirmation"
					description="Confirm moderators do not actively monitor this"
				>
					I acknowledge that the moderators do not actively monitor the thread.
				</Checkbox>
				<div class="input-group push-right">
					<ButtonStyled color="brand">
						<button
							:disabled="!replyConfirmation || isLoading"
							@click="runBlockingAction('reply-modal', () => sendReplyFromModal())"
						>
							<SpinnerIcon
								v-if="loadingAction === 'reply-modal'"
								class="animate-spin"
								aria-hidden="true"
							/>
							<ReplyIcon v-else aria-hidden="true" />
							Reply to thread
						</button>
					</ButtonStyled>
				</div>
			</div>
		</Modal>
		<div v-if="flags.developerMode" class="thread-id">
			Thread ID:
			<CopyCode :text="thread.id" />
		</div>
		<div v-if="sortedMessages.length > 0" class="messages universal-card recessed">
			<ThreadMessage
				v-for="message in sortedMessages"
				:key="'message-' + message.id"
				:thread="thread"
				:message="message"
				:members="members"
				:report="report"
				:auth="auth"
				raised
				@update-thread="() => updateThreadLocal()"
			/>
		</div>
		<template v-if="report && report.closed">
			<p>This thread is closed and new messages cannot be sent to it.</p>
			<ButtonStyled v-if="isStaff(auth.user)">
				<button :disabled="isLoading" @click="runBlockingAction('reopen', () => reopenReport())">
					<SpinnerIcon v-if="loadingAction === 'reopen'" class="animate-spin" aria-hidden="true" />
					<CheckCircleIcon v-else aria-hidden="true" />
					Reopen thread
				</button>
			</ButtonStyled>
		</template>
		<template v-else-if="!report || !report.closed">
			<div class="markdown-editor-spacing">
				<MarkdownEditor
					v-model="replyBody"
					:placeholder="sortedMessages.length > 0 ? 'Reply to thread...' : 'Send a message...'"
					:on-image-upload="onUploadImage"
				/>
			</div>
			<div class="input-group">
				<ButtonStyled color="brand">
					<button
						v-if="sortedMessages.length > 0"
						:disabled="!replyBody || isLoading"
						@click="
							isApproved(project) && !isStaff(auth.user)
								? openReplyModal()
								: runBlockingAction('reply', () => sendReply())
						"
					>
						<SpinnerIcon v-if="loadingAction === 'reply'" class="animate-spin" aria-hidden="true" />
						<ReplyIcon v-else aria-hidden="true" />
						Reply
					</button>
					<button
						v-else
						:disabled="!replyBody || isLoading"
						@click="
							isApproved(project) && !isStaff(auth.user)
								? openReplyModal()
								: runBlockingAction('send', () => sendReply())
						"
					>
						<SpinnerIcon v-if="loadingAction === 'send'" class="animate-spin" aria-hidden="true" />
						<SendIcon v-else aria-hidden="true" />
						Send
					</button>
				</ButtonStyled>
				<ButtonStyled v-if="isStaff(auth.user)">
					<button
						:disabled="!replyBody || isLoading"
						@click="runBlockingAction('private-note', () => sendReply(null, true))"
					>
						<SpinnerIcon
							v-if="loadingAction === 'private-note'"
							class="animate-spin"
							aria-hidden="true"
						/>
						<ScaleIcon v-else aria-hidden="true" />
						Add private note
					</button>
				</ButtonStyled>
				<template v-if="currentMember && !isStaff(auth.user)">
					<template v-if="isRejected(project)">
						<ButtonStyled color="orange">
							<button v-if="replyBody" :disabled="isLoading" @click="openResubmitModal(true)">
								<ScaleIcon aria-hidden="true" />
								Resubmit for review with reply
							</button>
							<button v-else :disabled="isLoading" @click="openResubmitModal(false)">
								<ScaleIcon aria-hidden="true" />
								Resubmit for review
							</button>
						</ButtonStyled>
					</template>
				</template>
				<div class="spacer"></div>
				<div class="input-group extra-options">
					<template v-if="report">
						<template v-if="isStaff(auth.user)">
							<ButtonStyled color="red">
								<button
									v-if="replyBody"
									:disabled="isLoading"
									@click="runBlockingAction('close-with-reply', () => closeReport(true))"
								>
									<SpinnerIcon
										v-if="loadingAction === 'close-with-reply'"
										class="animate-spin"
										aria-hidden="true"
									/>
									<CheckCircleIcon v-else aria-hidden="true" />
									Close with reply
								</button>
								<button
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
									Close thread
								</button>
							</ButtonStyled>
						</template>
					</template>
					<template v-if="project">
						<template v-if="isStaff(auth.user)">
							<ButtonStyled v-if="replyBody" color="green">
								<button
									:disabled="isApproved(project) || isLoading"
									@click="runBlockingAction('approve-with-reply', () => sendReply(requestedStatus))"
								>
									<SpinnerIcon
										v-if="loadingAction === 'approve-with-reply'"
										class="animate-spin"
										aria-hidden="true"
									/>
									<CheckIcon v-else aria-hidden="true" />
									Approve with reply
								</button>
							</ButtonStyled>
							<ButtonStyled v-else color="green">
								<button
									:disabled="isApproved(project) || isLoading"
									@click="runBlockingAction('approve', () => setStatus(requestedStatus))"
								>
									<SpinnerIcon
										v-if="loadingAction === 'approve'"
										class="animate-spin"
										aria-hidden="true"
									/>
									<CheckIcon v-else aria-hidden="true" />
									Approve
								</button>
							</ButtonStyled>
							<div class="joined-buttons">
								<ButtonStyled v-if="replyBody" color="red">
									<button
										:disabled="project.status === 'rejected' || isLoading"
										@click="runBlockingAction('reject-with-reply', () => sendReply('rejected'))"
									>
										<SpinnerIcon
											v-if="loadingAction === 'reject-with-reply'"
											class="animate-spin"
											aria-hidden="true"
										/>
										<XIcon v-else aria-hidden="true" />
										Reject with reply
									</button>
								</ButtonStyled>
								<ButtonStyled v-else color="red">
									<button
										:disabled="project.status === 'rejected' || isLoading"
										@click="runBlockingAction('reject', () => setStatus('rejected'))"
									>
										<SpinnerIcon
											v-if="loadingAction === 'reject'"
											class="animate-spin"
											aria-hidden="true"
										/>
										<XIcon v-else aria-hidden="true" />
										Reject
									</button>
								</ButtonStyled>
								<ButtonStyled color="red">
									<OverflowMenu
										class="btn-dropdown-animation"
										:disabled="isLoading"
										:options="
											replyBody
												? [
														{
															id: 'withhold-reply',
															color: 'danger',
															action: () =>
																runBlockingAction('withhold-reply', () => sendReply('withheld')),
															hoverFilled: true,
															disabled: project.status === 'withheld' || isLoading,
														},
														{
															id: 'set-to-draft-reply',
															action: () =>
																runBlockingAction('set-to-draft-reply', () => sendReply('draft')),
															hoverFilled: true,
															disabled: project.status === 'draft' || isLoading,
														},
														{
															id: 'send-to-review-reply',
															action: () =>
																runBlockingAction('send-to-review-reply', () =>
																	sendReply('processing', true),
																),
															hoverFilled: true,
															disabled: project.status === 'processing' || isLoading,
														},
													]
												: [
														{
															id: 'withhold',
															color: 'danger',
															action: () =>
																runBlockingAction('withhold', () => setStatus('withheld')),
															hoverFilled: true,
															disabled: project.status === 'withheld' || isLoading,
														},
														{
															id: 'set-to-draft',
															action: () =>
																runBlockingAction('set-to-draft', () => setStatus('draft')),
															hoverFilled: true,
															disabled: project.status === 'draft' || isLoading,
														},
														{
															id: 'send-to-review',
															action: () =>
																runBlockingAction('send-to-review', () => setStatus('processing')),
															hoverFilled: true,
															disabled: project.status === 'processing' || isLoading,
														},
													]
										"
									>
										<SpinnerIcon v-if="isDropdownLoading" class="animate-spin" aria-hidden="true" />
										<DropdownIcon v-else aria-hidden="true" />
										<template #withhold-reply>
											<EyeOffIcon aria-hidden="true" />
											Withhold with reply
										</template>
										<template #withhold>
											<EyeOffIcon aria-hidden="true" />
											Withhold
										</template>
										<template #set-to-draft-reply>
											<FileTextIcon aria-hidden="true" />
											Set to draft with reply
										</template>
										<template #set-to-draft>
											<FileTextIcon aria-hidden="true" />
											Set to draft
										</template>
										<template #send-to-review-reply>
											<ScaleIcon aria-hidden="true" />
											Send to review with reply
										</template>
										<template #send-to-review>
											<ScaleIcon aria-hidden="true" />
											Send to review
										</template>
									</OverflowMenu>
								</ButtonStyled>
							</div>
						</template>
					</template>
				</div>
			</div>
		</template>
	</div>
</template>

<script setup>
import {
	CheckCircleIcon,
	CheckIcon,
	DropdownIcon,
	EyeOffIcon,
	FileTextIcon,
	ReplyIcon,
	ScaleIcon,
	SendIcon,
	SpinnerIcon,
	XIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	Checkbox,
	CopyCode,
	injectNotificationManager,
	MarkdownEditor,
	OverflowMenu,
} from '@modrinth/ui'

import Modal from '~/components/ui/Modal.vue'
import ThreadMessage from '~/components/ui/thread/ThreadMessage.vue'
import { useImageUpload } from '~/composables/image-upload.ts'
import { isApproved, isRejected } from '~/helpers/projects.js'
import { isStaff } from '~/helpers/users.js'

const { addNotification } = injectNotificationManager()

const props = defineProps({
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
	setStatus: {
		type: Function,
		required: false,
		default: () => {},
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

const sortedMessages = computed(() => {
	if (props.thread !== null) {
		return props.thread.messages
			.slice()
			.sort((a, b) => app.$dayjs(a.created) - app.$dayjs(b.created))
	}
	return []
})

const modalSubmit = ref(null)
const modalReply = ref(null)

const loadingAction = ref(null)
const isLoading = computed(() => loadingAction.value !== null)
const dropdownActionIds = [
	'withhold',
	'withhold-reply',
	'set-to-draft',
	'set-to-draft-reply',
	'send-to-review',
	'send-to-review-reply',
]
const isDropdownLoading = computed(() => dropdownActionIds.includes(loadingAction.value))

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

async function updateThreadLocal() {
	let threadId = null
	if (props.project) {
		threadId = props.project.thread_id
	} else if (props.report) {
		threadId = props.report.thread_id
	}
	let thread = null
	if (threadId) {
		thread = await useBaseFetch(`thread/${threadId}`)
	}
	emit('update-thread', thread)
}

const imageIDs = ref([])

async function onUploadImage(file) {
	const response = await useImageUpload(file, { context: 'thread_message' })

	imageIDs.value.push(response.id)
	// Keep the last 10 entries of image IDs
	imageIDs.value = imageIDs.value.slice(-10)

	return response.url
}

async function sendReplyFromModal(status = null, privateMessage = false) {
	await sendReply(status, privateMessage)
	modalReply.value.hide()
}

async function sendReply(status = null, privateMessage = false) {
	try {
		const body = {
			body: {
				type: 'text',
				body: replyBody.value,
				private: privateMessage,
			},
		}

		if (imageIDs.value.length > 0) {
			body.body = {
				...body.body,
				uploaded_images: imageIDs.value,
			}
		}

		await useBaseFetch(`thread/${props.thread.id}`, {
			method: 'POST',
			body,
		})

		replyBody.value = ''

		await updateThreadLocal()
		if (status !== null) {
			await props.setStatus(status)
		}
	} catch (err) {
		addNotification({
			title: 'Error sending message',
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
}

async function closeReport(reply) {
	if (reply) {
		await sendReply()
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
			title: 'Error closing report',
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
			title: 'Error reopening report',
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
}

const replyWithSubmission = ref(false)
const submissionConfirmation = ref(false)
const replyConfirmation = ref(false)

function openResubmitModal(reply) {
	submissionConfirmation.value = false
	replyWithSubmission.value = reply
	modalSubmit.value.show()
}

function openReplyModal() {
	replyConfirmation.value = false
	modalReply.value.show()
}

async function resubmit() {
	if (replyWithSubmission.value) {
		await sendReply('processing')
	} else {
		await props.setStatus('processing')
	}
	modalSubmit.value.hide()
}

const requestedStatus = computed(() => props.project.requested_status ?? 'approved')
</script>

<style lang="scss" scoped>
.markdown-editor-spacing {
	margin-bottom: var(--gap-md);
}

.messages {
	display: flex;
	flex-direction: column;
	padding: var(--spacing-card-md);
}

.thread-id {
	margin-bottom: var(--spacing-card-md);
	font-weight: bold;
	color: var(--color-heading);
}

.input-group {
	.spacer {
		flex-grow: 1;
		flex-shrink: 1;
	}

	.extra-options {
		flex-basis: fit-content;
	}
}

.modal-submit {
	padding: var(--spacing-card-bg);
	display: flex;
	flex-direction: column;
	gap: var(--spacing-card-lg);

	.project-title {
		font-weight: bold;
	}
}
</style>
