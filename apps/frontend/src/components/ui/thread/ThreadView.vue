<template>
	<div>
		<div v-if="flags.showThreadIds" class="m-4 font-bold text-heading">
			Thread ID:
			<CopyCode :text="thread.id" />
		</div>

		<div v-if="sortedMessages.length > 0" class="flex flex-col rounded-xl">
			<ThreadMessage
				v-for="message in sortedMessages"
				:key="'message-' + message.id"
				:thread="thread"
				:message="message"
				:members="members"
				:auth="auth"
				raised
				@update-thread="() => updateThreadLocal()"
			/>
			<slot name="afterMessages" />
		</div>
		<div v-else class="flex flex-col items-center justify-center space-y-3 py-12">
			<MessageIcon class="size-12 text-secondary" />
			<p class="text-lg text-secondary">No messages yet</p>
		</div>

		<div v-if="closed" class="flex flex-col gap-4 p-4 pt-2">
			<p class="m-0 text-secondary">This thread is closed and new messages cannot be sent to it.</p>
			<div>
				<slot name="closedActions" />
			</div>
		</div>

		<template v-else>
			<div class="px-4 py-2">
				<MarkdownEditor
					v-model="replyBody"
					:placeholder="sortedMessages.length > 0 ? 'Reply to thread...' : 'Send a message...'"
					:on-image-upload="onUploadImage"
				/>
			</div>

			<div
				class="mt-4 flex flex-col items-stretch justify-between gap-3 px-4 pb-4 sm:flex-row sm:items-center sm:gap-2"
			>
				<div class="flex flex-col items-stretch gap-2 sm:flex-row sm:items-center">
					<SplitButton
						v-if="primaryAction === 'note' && isStaff(auth.user)"
						type="colored"
						color="brand"
						menu-label="More send options"
						:disabled="!replyBody"
						:options="publicMessageOptions"
						class="w-full sm:w-auto"
						@click="sendReply(true)"
					>
						Add note
					</SplitButton>
					<template v-else>
						<Button
							v-if="sortedMessages.length > 0"
							type="colored"
							color="brand"
							:disabled="!replyBody"
							class="w-full gap-2 sm:w-auto"
							@click="sendReply()"
						>
							<ReplyIcon class="size-4" />
							Reply
						</Button>
						<Button
							v-else
							type="colored"
							color="brand"
							:disabled="!replyBody"
							class="w-full gap-2 sm:w-auto"
							@click="sendReply()"
						>
							<SendIcon class="size-4" />
							Send
						</Button>
						<Button
							v-if="isStaff(auth.user)"
							:disabled="!replyBody"
							class="w-full sm:w-auto"
							@click="sendReply(true)"
						>
							Add note
						</Button>
					</template>
					<TeleportOverflowMenu
						v-if="visibleQuickReplies.length > 0"
						label="More options"
						:options="visibleQuickReplies"
						class="!w-auto !rounded-xl !px-2.5"
					>
						<ArrowUpFromLineIcon />
						Load preset
						<ChevronDownIcon />
					</TeleportOverflowMenu>
				</div>

				<div class="flex flex-col items-stretch gap-2 sm:flex-row sm:items-center">
					<slot name="additionalActions" :has-reply="!!replyBody" />
				</div>
			</div>
		</template>
	</div>
</template>

<script setup lang="ts" generic="T">
import {
	ArrowUpFromLineIcon,
	ChevronDownIcon,
	MessageIcon,
	ReplyIcon,
	SendIcon,
} from '@modrinth/assets'
import type { QuickReply } from '@modrinth/moderation'
import { Button, SplitButton, TeleportOverflowMenu } from '@modrinth/ui'
import {
	type ButtonMenuOption,
	CopyCode,
	injectNotificationManager,
	MarkdownEditor,
} from '@modrinth/ui'
import type { Thread, User } from '@modrinth/utils'
import dayjs from 'dayjs'

import { useImageUpload } from '~/composables/image-upload.ts'
import { isStaff } from '~/helpers/users.js'

import ThreadMessage from './ThreadMessage.vue'

const { addNotification } = injectNotificationManager()

const props = withDefaults(
	defineProps<{
		thread: Thread
		quickReplies?: ReadonlyArray<QuickReply<T>>
		quickReplyContext?: T
		closed?: boolean
		primaryAction?: 'reply' | 'note'
	}>(),
	{
		primaryAction: 'reply',
	},
)

const visibleQuickReplies = computed<ButtonMenuOption[]>(() => {
	const replies = props.quickReplies
	const context = props.quickReplyContext

	if (!replies || !context) return []

	return replies
		.filter((reply) => {
			if (reply.shouldShow === undefined) return true
			return reply.shouldShow(context)
		})
		.map(
			(reply) =>
				({
					id: reply.label,
					label: reply.label,
					action: () => handleQuickReply(reply, context),
				}) as ButtonMenuOption,
		)
})

async function handleQuickReply(reply: QuickReply<T>, context: T) {
	const message = typeof reply.message === 'function' ? await reply.message(context) : reply.message

	await nextTick()
	setReplyContent(message)
}

defineExpose({
	setReplyContent,
	getReplyContent,
	sendReply,
})

const auth = useAuthState()

const emit = defineEmits<{
	updateThread: [thread: Thread]
}>()

const flags = useFeatureFlags()

const members = computed(() => {
	const membersMap: Record<string, User> = {}
	for (const member of props.thread.members) {
		membersMap[member.id] = member
	}
	return membersMap
})

const replyBody = ref('')

const publicMessageOptions = computed<ButtonMenuOption[]>(() => [
	{
		id: 'send-public',
		label: 'Send publicly',
		icon: SendIcon,
		action: () => sendReply(false),
		disabled: !replyBody.value,
	},
])

function setReplyContent(content: string) {
	replyBody.value = content
}

function getReplyContent(): string {
	return replyBody.value
}

const sortedMessages = computed(() => {
	if (!props.thread) return []

	return [...props.thread.messages].sort(
		(a, b) => dayjs(a.created).toDate().getTime() - dayjs(b.created).toDate().getTime(),
	)
})

async function updateThreadLocal() {
	const threadId = props.thread.id
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
</script>
