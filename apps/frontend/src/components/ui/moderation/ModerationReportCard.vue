<template>
	<div class="universal-card">
		<div
			class="flex w-full flex-col items-start justify-between gap-3 sm:flex-row sm:items-center sm:gap-0"
		>
			<span class="text-md flex flex-col gap-2 sm:flex-row sm:items-center">
				<span class="flex items-center gap-2">
					Reported for
					<span class="whitespace-nowrap rounded-full align-middle font-semibold text-contrast">
						{{ formattedReportType }}
					</span>
				</span>
				<span class="flex items-center gap-2">
					<span class="hidden sm:inline">By</span>
					<span class="sm:hidden">Reporter:</span>
					<nuxt-link
						:to="`/user/${report.reporter_user.username}`"
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
				<span class="text-md whitespace-nowrap text-secondary">{{
					formatRelativeTime(report.created)
				}}</span>
				<ButtonStyled v-if="visibleQuickReplies.length > 0" circular>
					<OverflowMenu :options="visibleQuickReplies">
						<span class="hidden sm:inline">Quick Reply</span>
						<span class="sr-only sm:hidden">Quick Reply</span>
						<ChevronDownIcon />
					</OverflowMenu>
				</ButtonStyled>
				<ButtonStyled circular>
					<OverflowMenu :options="quickActions">
						<template #default>
							<EllipsisVerticalIcon />
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

		<hr class="my-4 rounded-xl border-solid text-divider" />

		<div class="flex flex-col gap-4">
			<div class="flex flex-col gap-3 sm:flex-row sm:items-center">
				<div class="flex min-w-0 flex-1 items-center gap-3">
					<Avatar
						:src="reportItemAvatarUrl"
						:circle="report.item_type === 'user'"
						size="3rem"
						class="flex-shrink-0"
					/>
					<div class="min-w-0 flex-1">
						<span class="block truncate text-lg font-semibold">{{ reportItemTitle }}</span>
						<div class="flex flex-col gap-2 text-sm text-secondary sm:flex-row sm:items-center">
							<nuxt-link
								v-if="report.target && report.item_type != 'user'"
								:to="`/${report.target.type}/${report.target.slug}`"
								class="inline-flex flex-row items-center gap-1 truncate transition-colors duration-100 ease-in-out hover:text-brand"
							>
								<Avatar
									:src="report.target?.avatar_url"
									:circle="report.target.type === 'user'"
									size="1rem"
									class="flex-shrink-0"
								/>
								<span class="truncate">
									<OrganizationIcon
										v-if="report.target.type === 'organization'"
										class="align-middle"
									/>
									{{ report.target.name || 'Unknown User' }}
								</span>
							</nuxt-link>

							<div class="flex flex-wrap items-center gap-2">
								<span
									class="whitespace-nowrap rounded-full bg-button-bg p-0.5 px-2 text-xs font-semibold text-secondary"
								>
									{{ formattedItemType }}
								</span>
								<span
									v-if="report.item_type === 'version' && report.version"
									class="max-w-[200px] truncate font-mono text-xs sm:max-w-none"
								>
									{{
										report.version.files.find((file) => file.primary)?.filename || 'Unknown Version'
									}}
								</span>
							</div>
						</div>
					</div>
				</div>

				<div class="flex justify-end sm:justify-start">
					<ButtonStyled circular>
						<nuxt-link :to="reportItemUrl">
							<EyeIcon />
						</nuxt-link>
					</ButtonStyled>
				</div>
			</div>
		</div>

		<CollapsibleRegion ref="collapsibleRegion" class="my-4">
			<ReportThread
				v-if="report.thread"
				ref="reportThread"
				class="mb-16 sm:mb-0"
				:thread="report.thread"
				:report="report"
				:reporter="report.reporter_user"
				@update-thread="updateThread"
			/>
		</CollapsibleRegion>
	</div>
</template>
<script setup lang="ts">
import {
	ClipboardCopyIcon,
	EllipsisVerticalIcon,
	EyeIcon,
	LinkIcon,
	OrganizationIcon,
} from '@modrinth/assets'
import {
	type ExtendedReport,
	reportQuickReplies,
	type ReportQuickReply,
} from '@modrinth/moderation'
import {
	Avatar,
	ButtonStyled,
	CollapsibleRegion,
	injectNotificationManager,
	OverflowMenu,
	type OverflowMenuOption,
	useRelativeTime,
} from '@modrinth/ui'
import { computed } from 'vue'

import ChevronDownIcon from '../servers/icons/ChevronDownIcon.vue'
import ReportThread from '../thread/ReportThread.vue'

const { addNotification } = injectNotificationManager()

const props = defineProps<{
	report: ExtendedReport
}>()

const reportThread = ref<InstanceType<typeof ReportThread> | null>(null)
const collapsibleRegion = ref<InstanceType<typeof CollapsibleRegion> | null>(null)

const formatRelativeTime = useRelativeTime()

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

const visibleQuickReplies = computed<OverflowMenuOption[]>(() => {
	return reportQuickReplies
		.filter((reply) => {
			if (reply.shouldShow === undefined) return true
			if (typeof reply.shouldShow === 'function') {
				return reply.shouldShow(props.report)
			}

			return reply.shouldShow
		})
		.map(
			(reply) =>
				({
					id: reply.label,
					action: () => handleQuickReply(reply),
				}) as OverflowMenuOption,
		)
})

async function handleQuickReply(reply: ReportQuickReply) {
	const message =
		typeof reply.message === 'function' ? await reply.message(props.report) : reply.message

	collapsibleRegion.value?.setCollapsed(false)
	await nextTick()
	reportThread.value?.setReplyContent(message)
}

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

const formattedItemType = computed(() => {
	const itemType = props.report.item_type
	return itemType.charAt(0).toUpperCase() + itemType.slice(1)
})

const formattedReportType = computed(() => {
	const reportType = props.report.report_type

	// some are split by -, some are split by " "
	const words = reportType.includes('-') ? reportType.split('-') : reportType.split(' ')
	return words.map((word) => word.charAt(0).toUpperCase() + word.slice(1)).join(' ')
})
</script>

<style lang="scss" scoped></style>
