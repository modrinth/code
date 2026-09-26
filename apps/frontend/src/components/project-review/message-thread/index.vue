<template>
	<div class="flex min-h-0 min-w-0 flex-1 flex-col">
		<div
			ref="scrollContainer"
			class="min-h-0 flex-1 overflow-y-auto overscroll-contain"
			@scroll="updateScrollPosition"
		>
			<div ref="content" class="flex min-h-full flex-col justify-end">
				<template v-if="thread">
					<ThreadMessage
						v-for="(message, index) in sortedMessages"
						:key="message.id ?? `${message.created}:${index}`"
						:message="message"
						:members="members"
						:auth="auth"
						raised
						class="shrink-0 text-xs"
						@update-thread="() => refetch()"
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
		<Composer v-if="thread" ref="composer" />
	</div>
</template>

<script setup lang="ts">
import { Button, useVIntl } from '@modrinth/ui'
import { useResizeObserver } from '@vueuse/core'
import { computed, nextTick, ref, watch } from 'vue'

import ThreadMessage from '~/components/ui/thread/ThreadMessage.vue'
import { injectProjectReviewPageContext } from '~/providers/project-review'

import { injectProjectReviewContext } from '../layout/context'
import { projectReviewMessages as messages } from '../messages'
import Composer from './composer.vue'

const { threadQuery } = injectProjectReviewPageContext()
const { rightVisible, toggleSidebar } = injectProjectReviewContext()
const { data: thread, isError, refetch } = threadQuery
const auth = useAuthState()
const { formatMessage } = useVIntl()
const sortedMessages = computed(() =>
	[...(thread.value?.messages ?? [])].sort((a, b) => Date.parse(a.created) - Date.parse(b.created)),
)
const members = computed(() =>
	Object.fromEntries((thread.value?.members ?? []).map((member) => [member.id, member])),
)
const composer = ref<InstanceType<typeof Composer>>()
const scrollContainer = ref<HTMLElement>()
const content = ref<HTMLElement>()
const isAtBottom = ref(true)
function updateScrollPosition() {
	const container = scrollContainer.value
	if (!container || !container.clientHeight) return
	isAtBottom.value = container.scrollHeight - container.scrollTop - container.clientHeight <= 1
}
function scrollToBottom() {
	const container = scrollContainer.value
	if (container?.clientHeight) container.scrollTop = container.scrollHeight
}
useResizeObserver([scrollContainer, content], () => {
	if (isAtBottom.value) scrollToBottom()
})
watch(
	() => thread.value?.id,
	() => {
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
	await composer.value?.openEditor()
}
defineExpose({ openEditor })
</script>
