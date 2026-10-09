<template>
	<div>
		<div v-if="flags.showThreadIds" class="m-4 font-bold text-heading">
			Thread ID:
			<CopyCode :text="thread.id" />
		</div>

		<div
			v-if="sortedMessages.length > 0 || (isStaff(auth.user) && thread.issues?.length)"
			class="flex flex-col rounded-xl"
		>
			<ThreadTimeline
				:project-owner-id="projectOwner?.id"
				:messages="thread.messages"
				:issues="thread.issues"
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
					<template v-if="primaryAction === 'note' && isStaff(auth.user)">
						<Button
							type="colored"
							color="brand"
							:disabled="!replyBody"
							class="w-full sm:w-auto"
							@click="sendReply(false)"
						>
							<SendIcon />
							Send publicly
						</Button>
						<Button :disabled="!replyBody" class="w-full sm:w-auto" @click="sendReply(true)">
							<StickyNotePlusIcon />
							Add private note
						</Button>
					</template>
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
							Send publicly
						</Button>
						<Button
							v-if="isStaff(auth.user)"
							:disabled="!replyBody"
							class="w-full sm:w-auto"
							@click="sendReply(true)"
						>
							<StickyNotePlusIcon />
							Add private note
						</Button>
					</template>
					<TeleportOverflowMenu
						v-if="visibleQuickReplies.length > 0"
						type="outlined"
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
import type { Labrinth } from '@modrinth/api-client'
import {
	ArrowUpFromLineIcon,
	ChevronDownIcon,
	MessageIcon,
	ReplyIcon,
	SendIcon,
	StickyNotePlusIcon,
} from '@modrinth/assets'
import type { QuickReply } from '@modrinth/moderation'
import { Button, TeleportOverflowMenu } from '@modrinth/ui'
import {
	type ButtonMenuOption,
	CopyCode,
	injectNotificationManager,
	MarkdownEditor,
} from '@modrinth/ui'
import type { Thread } from '@modrinth/utils'
import dayjs from 'dayjs'

import { useImageUpload } from '~/composables/image-upload.ts'
import { useThreadProjectOwner } from '~/composables/thread-project-owner'
import { isStaff } from '~/helpers/users.js'

import ThreadTimeline from './ThreadTimeline.vue'

const { addNotification } = injectNotificationManager()

const props = withDefaults(
	defineProps<{
		thread: Thread & { issues?: Labrinth.Threads.v3.ThreadIssue[] }
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

const projectOwner = useThreadProjectOwner(computed(() => props.thread))

const members = computed(() => {
	const membersMap: Record<
		string,
		Pick<Labrinth.Threads.v3.ThreadMember, 'id' | 'username' | 'role'> &
			Partial<Pick<Labrinth.Threads.v3.ThreadMember, 'avatar_url'>>
	> = {}
	for (const member of props.thread.members) {
		membersMap[member.id] = member
	}
	if (projectOwner.value) membersMap[projectOwner.value.id] = projectOwner.value
	return membersMap
})

const replyBody = ref('')

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
