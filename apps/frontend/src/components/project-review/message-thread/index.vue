<template>
	<div class="flex min-h-0 min-w-0 flex-1 flex-col">
		<div
			ref="scrollContainer"
			class="min-h-0 flex-1 overflow-y-auto overscroll-contain [overflow-anchor:none]"
			@scroll="updateScrollPosition"
		>
			<div ref="content" class="flex min-h-full flex-col justify-end">
				<template v-if="thread">
					<ThreadTimeline
						:messages="thread.messages"
						:issues="thread.issues"
						:members="members"
						:auth="auth"
						raised
						image-previews
						message-class="shrink-0 !px-2 text-xs"
						@update-thread="() => refetch()"
						@open-image="openImage"
					/>
				</template>
				<div v-else-if="isError" class="flex flex-col gap-3 p-4">
					<p class="m-0 text-red" role="alert">
						{{ formatMessage(messages.loadError) }}
					</p>
					<Button class="w-fit" @click="() => refetch()">{{
						formatMessage(messages.retry)
					}}</Button>
				</div>
				<p v-else class="m-0 p-4 text-secondary" role="status">
					{{ formatMessage(messages.loading) }}
				</p>
			</div>
		</div>
		<MessageBox v-if="thread" ref="messageBox" />
		<ImageViewerEditor
			ref="viewer"
			:items="imageItems"
			editor="disabled"
			:pixelated="pixelated"
			@hide="restoreImageFocus"
		>
			<template #actions="{ item }">
				<ImageViewerActions v-model:pixelated="pixelated" :src="item.src" />
			</template>
		</ImageViewerEditor>
	</div>
</template>

<script setup lang="ts">
import { Button, ImageViewerEditor, useVIntl } from '@modrinth/ui'
import { useResizeObserver } from '@vueuse/core'
import { computed, nextTick, ref, watch } from 'vue'

import ThreadTimeline from '~/components/ui/thread/ThreadTimeline.vue'
import { injectProjectReviewPageContext } from '~/providers/project-review'

import ImageViewerActions from '../image-viewer-actions.vue'
import { injectProjectReviewContext } from '../layout/context'
import { projectReviewMessages as messages } from '../messages'
import MessageBox from './message-box.vue'

const { threadQuery, pixelated } = injectProjectReviewPageContext()
const { rightVisible, toggleSidebar } = injectProjectReviewContext()
const { data: thread, isError, refetch } = threadQuery
const auth = useAuthState()
const { formatMessage } = useVIntl()
const viewer = ref<InstanceType<typeof ImageViewerEditor>>()
const imageItems = ref<{ id: string; src: string; alt: string }[]>([])
let imageTrigger: HTMLImageElement | undefined

async function openImage(image: { src: string; alt: string; element: HTMLImageElement }) {
	imageTrigger = image.element
	imageItems.value = [
		{
			id: image.src,
			src: image.src,
			alt: image.alt || formatMessage(messages.imageNumber, { number: 1 }),
		},
	]
	await nextTick()
	viewer.value?.show(0)
}

function restoreImageFocus() {
	if (imageTrigger?.isConnected) imageTrigger.focus({ preventScroll: true })
	imageTrigger = undefined
}
const members = computed(() =>
	Object.fromEntries((thread.value?.members ?? []).map((member) => [member.id, member])),
)
const messageBox = ref<InstanceType<typeof MessageBox>>()
const scrollContainer = ref<HTMLElement>()
const content = ref<HTMLElement>()
const isAtBottom = ref(true)
let bottomOffset = 0
let viewportHeight = 0
let contentHeight = 0
function updateScrollPosition() {
	const container = scrollContainer.value
	if (!container || !container.clientHeight) return
	if (container.clientHeight !== viewportHeight || container.scrollHeight !== contentHeight) return
	bottomOffset = Math.max(0, container.scrollHeight - container.scrollTop - container.clientHeight)
	isAtBottom.value = bottomOffset <= 1
}
function scrollToBottom() {
	const container = scrollContainer.value
	bottomOffset = 0
	if (container?.clientHeight) container.scrollTop = container.scrollHeight
}
useResizeObserver([scrollContainer, content], () => {
	const container = scrollContainer.value
	if (!container?.clientHeight) return
	if (isAtBottom.value) {
		scrollToBottom()
	} else if (container.clientHeight !== viewportHeight) {
		container.scrollTop = container.scrollHeight - container.clientHeight - bottomOffset
	}
	viewportHeight = container.clientHeight
	contentHeight = container.scrollHeight
	updateScrollPosition()
})
watch(
	() => thread.value?.id,
	() => {
		imageTrigger = undefined
		imageItems.value = []
		isAtBottom.value = true
		scrollToBottom()
	},
	{ flush: 'post' },
)
async function openEditor() {
	if (!rightVisible.value) toggleSidebar('right')
	isAtBottom.value = true
	await nextTick()
	scrollToBottom()
	await messageBox.value?.openEditor()
}
defineExpose({ openEditor })
</script>
