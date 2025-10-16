<template>
	<div>
		<div v-if="flags.developerMode" class="mb-4 font-bold text-heading">
			Thread ID:
			<CopyCode :text="thread.id" />
		</div>

		<div
			v-if="sortedMessages.length > 0"
			class="bg-raised flex flex-col space-y-4 rounded-xl p-3 sm:p-4"
		>
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

		<template v-if="reportClosed">
			<p class="text-secondary">This thread is closed and new messages cannot be sent to it.</p>
			<ButtonStyled v-if="isStaff(auth.user)" color="green" class="mt-2 w-full sm:w-auto">
				<button
					class="flex w-full items-center justify-center gap-2 sm:w-auto"
					@click="reopenReport()"
				>
					<CheckCircleIcon class="size-4" />
					Reopen Thread
				</button>
			</ButtonStyled>
		</template>

		<template v-else>
			<div class="mt-4">
				<MarkdownEditor
					v-model="replyBody"
					:placeholder="sortedMessages.length > 0 ? 'Reply to thread...' : 'Send a message...'"
					:on-image-upload="onUploadImage"
				/>
			</div>

			<div
				class="mt-4 flex flex-col items-stretch justify-between gap-3 sm:flex-row sm:items-center sm:gap-2"
			>
				<div class="flex flex-col items-stretch gap-2 sm:flex-row sm:items-center">
					<ButtonStyled v-if="sortedMessages.length > 0" color="brand" class="w-full sm:w-auto">
						<button
							:disabled="!replyBody"
							class="flex w-full items-center justify-center gap-2 sm:w-auto"
							@click="sendReply()"
						>
							<ReplyIcon class="size-4" />
							Reply
						</button>
					</ButtonStyled>
					<ButtonStyled v-else color="brand" class="w-full sm:w-auto">
						<button
							:disabled="!replyBody"
							class="flex w-full items-center justify-center gap-2 sm:w-auto"
							@click="sendReply()"
						>
							<SendIcon class="size-4" />
							Send
						</button>
					</ButtonStyled>
					<ButtonStyled v-if="isStaff(auth.user)" class="w-full sm:w-auto">
						<button
							:disabled="!replyBody"
							class="flex w-full items-center justify-center gap-2 sm:w-auto"
							@click="sendReply(true)"
						>
							<ScaleIcon class="size-4" />
							<span class="hidden sm:inline">Add private note</span>
							<span class="sm:hidden">Private note</span>
						</button>
					</ButtonStyled>
				</div>

				<div class="flex flex-col items-stretch gap-2 sm:flex-row sm:items-center">
					<template v-if="isStaff(auth.user)">
						<ButtonStyled v-if="replyBody" color="red" class="w-full sm:w-auto">
							<button
								class="flex w-full items-center justify-center gap-2 sm:w-auto"
								@click="closeReport(true)"
							>
								<CheckCircleIcon class="size-4" />
								<span class="hidden sm:inline">Close with reply</span>
								<span class="sm:hidden">Close & reply</span>
							</button>
						</ButtonStyled>
						<ButtonStyled v-else color="red" class="w-full sm:w-auto">
							<button
								class="flex w-full items-center justify-center gap-2 sm:w-auto"
								@click="closeReport()"
							>
								<CheckCircleIcon class="size-4" />
								Close report
							</button>
						</ButtonStyled>
					</template>
				</div>
			</div>
		</template>
	</div>
</template>

<script setup lang="ts">
import { CheckCircleIcon, ReplyIcon, ScaleIcon, SendIcon } from '@modrinth/assets'
import { ButtonStyled, CopyCode, injectNotificationManager, MarkdownEditor } from '@modrinth/ui'
import type { Report, Thread, ThreadMessage as TypeThreadMessage, User } from '@modrinth/utils'
import dayjs from 'dayjs'

import { useImageUpload } from '~/composables/image-upload.ts'
import { isStaff } from '~/helpers/users.js'

import ThreadMessage from './ThreadMessage.vue'

const { addNotification } = injectNotificationManager()

const props = defineProps<{
	thread: Thread
	reporter: User
	report: Report
}>()

defineExpose({
	setReplyContent,
})

const auth = await useAuth()

const emit = defineEmits<{
	updateThread: [thread: Thread]
}>()

const flags = useFeatureFlags()

const members = computed(() => {
	const membersMap: Record<string, User> = {
		[props.reporter.id]: props.reporter,
	}
	for (const member of props.thread.members) {
		membersMap[member.id] = member
	}
	return membersMap
})

const replyBody = ref('')
function setReplyContent(content: string) {
	replyBody.value = content
}

const sortedMessages = computed(() => {
	const messages: TypeThreadMessage[] = [
		{
			id: null,
			author_id: props.reporter.id,
			body: {
				type: 'text',
				body: props.report.body || 'Report opened.',
				private: false,
				replying_to: null,
				associated_images: [],
			},
			created: props.report.created,
			hide_identity: false,
		},
	]
	if (props.thread) {
		messages.push(
			...[...props.thread.messages].sort(
				(a, b) => dayjs(a.created).toDate().getTime() - dayjs(b.created).toDate().getTime(),
			),
		)
	}

	return messages
})

async function updateThreadLocal() {
	const threadId = props.report.thread_id
	if (threadId) {
		try {
			const thread = (await useBaseFetch(`thread/${threadId}`)) as Thread
			emit('updateThread', thread)
		} catch (error) {
			console.error('Failed to update thread:', error)
		}
	}
}

const imageIDs = ref<string[]>([])

async function onUploadImage(file: File) {
	const response = await useImageUpload(file, { context: 'thread_message' })

	imageIDs.value.push(response.id)
	imageIDs.value = imageIDs.value.slice(-10)

	return response.url
}

async function sendReply(privateMessage = false) {
	try {
		const body: any = {
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
	} catch (err: any) {
		addNotification({
			title: 'Error sending message',
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
}

const didCloseReport = ref(false)
const reportClosed = computed(() => {
	return didCloseReport.value || (props.report && props.report.closed)
})

async function closeReport(reply = false) {
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
		didCloseReport.value = true
	} catch (err: any) {
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
	} catch (err: any) {
		addNotification({
			title: 'Error reopening report',
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
}
</script>
