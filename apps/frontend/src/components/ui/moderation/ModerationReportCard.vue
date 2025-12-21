<template>
	<div class="overflow-hidden rounded-2xl">
		<div class="bg-bg-raised p-4">
			<div
				class="flex w-full flex-col items-start justify-between gap-3 sm:flex-row sm:items-center sm:gap-0"
			>
				<span class="text-md flex flex-col gap-2 sm:flex-row sm:items-center">
					<span class="flex items-center gap-2">
						<span class="text-secondary">Reported for</span>
						<span class="font-semibold text-contrast">
							{{ formattedReportType }}
						</span>
					</span>
					<span class="flex items-center gap-2">
						<span class="hidden text-secondary sm:inline">By</span>
						<span class="text-secondary sm:hidden">Reporter:</span>
						<nuxt-link
							:to="`/user/${report.reporter_user.username}`"
							target="_blank"
							class="inline-flex flex-row items-center gap-1 transition-colors duration-100 ease-in-out hover:text-brand"
						>
							<Avatar
								:src="report.reporter_user.avatar_url"
								circle
								size="1.75rem"
								class="flex-shrink-0"
							/>
							<span class="truncate">{{ report.reporter_user.username }}</span>
						</nuxt-link>
					</span>
				</span>

				<div class="flex flex-row items-center gap-2 self-end sm:self-auto">
					<span class="whitespace-nowrap text-sm text-secondary">{{
						formatRelativeTime(report.created)
					}}</span>
					<ButtonStyled circular>
						<OverflowMenu :options="quickActions">
							<template #default>
								<EllipsisVerticalIcon class="size-4" />
							</template>
							<template #copy-id>
								<ClipboardCopyIcon />
								<span class="hidden sm:inline">Copy ID</span>
							</template>
							<template #copy-link>
								<LinkIcon />
								<span class="hidden sm:inline">Copy link</span>
							</template>
						</OverflowMenu>
					</ButtonStyled>
				</div>
			</div>

			<div class="my-4 h-px bg-surface-5" />

			<div class="flex items-center justify-between">
				<div class="flex items-center gap-4">
					<Avatar
						:src="reportItemAvatarUrl"
						:circle="report.item_type === 'user'"
						size="4rem"
						:class="[
							'flex-shrink-0 border border-surface-5 bg-surface-4 !shadow-none',
							report.item_type !== 'user' && 'rounded-2xl',
						]"
					/>

					<div v-if="report.item_type === 'user'" class="flex flex-col gap-1.5">
						<NuxtLink
							:to="`/user/${report.user?.username}`"
							target="_blank"
							class="text-base font-semibold text-contrast hover:underline"
						>
							{{ report.user?.username || 'Unknown User' }}
						</NuxtLink>

						<span
							v-if="report.user?.created"
							v-tooltip="formatExactDate(report.user.created)"
							class="cursor-help text-sm text-secondary"
						>
							Joined {{ formatRelativeTime(report.user.created) }}
						</span>
					</div>

					<div v-else class="flex flex-col gap-1.5">
						<div class="flex items-center gap-2">
							<NuxtLink
								:to="reportItemUrl"
								target="_blank"
								class="text-base font-semibold text-contrast hover:underline"
							>
								{{ reportItemTitle }}
							</NuxtLink>

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

							<span
								v-if="report.item_type === 'version' && report.version"
								class="text-sm text-secondary"
							>
								{{ report.version.files.find((f) => f.primary)?.filename || 'Unknown Version' }}
							</span>
						</div>

						<div v-if="report.target" class="flex items-center gap-1">
							<Avatar
								:src="report.target.avatar_url"
								size="1.5rem"
								circle
								class="border border-surface-5 bg-surface-4 !shadow-none"
							/>
							<NuxtLink
								:to="`/${report.target.type}/${report.target.slug}`"
								target="_blank"
								class="text-sm font-medium text-secondary hover:underline"
							>
								{{ report.target.name }}
							</NuxtLink>
						</div>
					</div>
				</div>
			</div>
		</div>
		<CollapsibleRegion
			v-model:collapsed="isThreadCollapsed"
			:expand-text="expandText"
			collapse-text="Collapse thread"
		>
			<div class="bg-surface-2 p-4 pt-2">
				<ThreadView
					v-if="report.thread"
					ref="reportThread"
					:thread="report.thread"
					:quick-replies="reportQuickReplies"
					:quick-reply-context="report"
					:closed="reportClosed"
					@update-thread="updateThread"
				>
					<template #closedActions>
						<ButtonStyled v-if="isStaff(auth.user)" color="green" class="mt-2">
							<button class="w-full gap-2 sm:w-auto" @click="reopenReport()">
								<CheckCircleIcon class="size-4" />
								Reopen Thread
							</button>
						</ButtonStyled>
					</template>
					<template #additionalActions="{ hasReply }">
						<template v-if="isStaff(auth.user)">
							<ButtonStyled v-if="hasReply" color="red">
								<button class="w-full gap-2 sm:w-auto" @click="closeReport(true)">
									<CheckCircleIcon class="size-4" />
									Reply and close
								</button>
							</ButtonStyled>
							<ButtonStyled v-else color="red">
								<button class="w-full gap-2 sm:w-auto" @click="closeReport()">
									<CheckCircleIcon class="size-4" />
									Close report
								</button>
							</ButtonStyled>
						</template>
					</template>
				</ThreadView>
			</div>
		</CollapsibleRegion>
	</div>
</template>
<script setup lang="ts">
import {
	CheckCircleIcon,
	ClipboardCopyIcon,
	EllipsisVerticalIcon,
	LinkIcon,
} from '@modrinth/assets'
import { type ExtendedReport, reportQuickReplies } from '@modrinth/moderation'
import type { OverflowMenuOption } from '@modrinth/ui'
import {
	Avatar,
	ButtonStyled,
	CollapsibleRegion,
	getProjectTypeIcon,
	injectNotificationManager,
	OverflowMenu,
	useRelativeTime,
} from '@modrinth/ui'
import { formatProjectType } from '@modrinth/utils'
import dayjs from 'dayjs'
import { computed } from 'vue'

import { isStaff } from '~/helpers/users.js'

import ThreadView from '../thread/ThreadView.vue'

const { addNotification } = injectNotificationManager()
const auth = await useAuth()

const props = defineProps<{
	report: ExtendedReport
}>()

const reportThread = ref<{
	setReplyContent: (content: string) => void
	sendReply: (privateMessage?: boolean) => Promise<void>
} | null>(null)
const isThreadCollapsed = ref(true)

const didCloseReport = ref(false)
const reportClosed = computed(() => {
	return didCloseReport.value || props.report.closed
})

const remainingMessageCount = computed(() => {
	if (!props.report.thread?.messages) return 0
	return Math.max(0, props.report.thread.messages.length - 1)
})

const expandText = computed(() => {
	if (remainingMessageCount.value === 0) return 'Expand'
	if (remainingMessageCount.value === 1) return 'Show 1 more message'
	return `Show ${remainingMessageCount.value} more messages`
})

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
		updateThread(props.report.thread)
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
		updateThread(props.report.thread)
		didCloseReport.value = false
	} catch (err: any) {
		addNotification({
			title: 'Error reopening report',
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
}

const formatRelativeTime = useRelativeTime()

function formatExactDate(date: string): string {
	return dayjs(date).format('MMMM D, YYYY [at] h:mm A')
}

function updateThread(newThread: any) {
	if (props.report.thread) {
		Object.assign(props.report.thread, newThread)
	}
}

const quickActions: OverflowMenuOption[] = [
	{
		id: 'copy-link',
		action: () => {
			const base = window.location.origin
			const reportUrl = `${base}/moderation/reports/${props.report.id}`
			navigator.clipboard.writeText(reportUrl).then(() => {
				addNotification({
					type: 'success',
					title: 'Report link copied',
					text: 'The link to this report has been copied to your clipboard.',
				})
			})
		},
	},
	{
		id: 'copy-id',
		action: () => {
			navigator.clipboard.writeText(props.report.id).then(() => {
				addNotification({
					type: 'success',
					title: 'Report ID copied',
					text: 'The ID of this report has been copied to your clipboard.',
				})
			})
		},
	},
]

const reportItemAvatarUrl = computed(() => {
	switch (props.report.item_type) {
		case 'project':
		case 'version':
			return props.report.project?.icon_url || ''
		case 'user':
			return props.report.user?.avatar_url || ''
		default:
			return undefined
	}
})

const reportItemTitle = computed(() => {
	if (props.report.item_type === 'user') return props.report.user?.username || 'Unknown User'

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
		default:
			return `/${props.report.item_type}/${props.report.id}`
	}
})

const formattedReportType = computed(() => {
	const reportType = props.report.report_type

	// some are split by -, some are split by " "
	const words = reportType.includes('-') ? reportType.split('-') : reportType.split(' ')
	return words.map((word) => word.charAt(0).toUpperCase() + word.slice(1)).join(' ')
})
</script>
