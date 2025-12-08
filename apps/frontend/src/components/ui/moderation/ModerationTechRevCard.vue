<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	CheckCircleIcon,
	CheckIcon,
	ChevronDownIcon,
	ClipboardCopyIcon,
	CodeIcon,
	CopyIcon,
	DownloadIcon,
	EllipsisVerticalIcon,
	LinkIcon,
	LoaderCircleIcon,
	ShieldCheckIcon,
	TriangleAlertIcon,
} from '@modrinth/assets'
import { type TechReviewContext, techReviewQuickReplies } from '@modrinth/moderation'
import {
	Avatar,
	ButtonStyled,
	Collapsible,
	CollapsibleRegion,
	getProjectTypeIcon,
	injectModrinthClient,
	injectNotificationManager,
	MarkdownEditor,
	NewModal,
	OverflowMenu,
	type OverflowMenuOption,
} from '@modrinth/ui'
import { capitalizeString, formatProjectType, highlightCodeLines } from '@modrinth/utils'
import { computed, onUnmounted, ref, watch } from 'vue'

import NavTabs from '~/components/ui/NavTabs.vue'
import ThreadView from '~/components/ui/thread/ThreadView.vue'

const props = defineProps<{
	item: {
		project: Labrinth.Projects.v3.Project
		project_owner: Labrinth.TechReview.Internal.Ownership
		thread: Labrinth.TechReview.Internal.Thread
		reports: Labrinth.TechReview.Internal.FileReport[]
	}
	loadingIssues: Set<string>
}>()

const { addNotification } = injectNotificationManager()

const emit = defineEmits<{
	refetch: []
	loadFileSources: [reportId: string]
	markComplete: [projectId: string]
	markIssueSafe: [projectId: string, reportId: string, issueId: string]
}>()

const quickActions = computed<OverflowMenuOption[]>(() => {
	const actions: OverflowMenuOption[] = []

	const sourceUrl = props.item.project.link_urls?.['source']?.url
	if (sourceUrl) {
		actions.push({
			id: 'view-source',
			action: () => {
				window.open(sourceUrl, '_blank', 'noopener,noreferrer')
			},
		})
	}

	actions.push(
		{
			id: 'copy-link',
			action: () => {
				const base = window.location.origin
				const reportUrl = `${base}/moderation/technical-review/${props.item.project.id}`
				navigator.clipboard.writeText(reportUrl).then(() => {
					addNotification({
						type: 'success',
						title: 'Technical Report link copied',
						text: 'The link to this report has been copied to your clipboard.',
					})
				})
			},
		},
		{
			id: 'copy-id',
			action: () => {
				navigator.clipboard.writeText(props.item.project.id).then(() => {
					addNotification({
						type: 'success',
						title: 'Technical Report ID copied',
						text: 'The ID of this report has been copied to your clipboard.',
					})
				})
			},
		},
	)

	return actions
})

type Tab = 'Thread' | 'Files'
const tabs: readonly Tab[] = ['Thread', 'Files']
const currentTab = ref<Tab>('Thread')

const isThreadCollapsed = ref(true)

watch(
	() => props.item.thread?.messages?.length,
	(len) => {
		if (!len || len <= 1) {
			isThreadCollapsed.value = false
		}
	},
	{ immediate: true },
)

const remainingMessageCount = computed(() => {
	if (!props.item.thread?.messages) return 0
	return Math.max(0, props.item.thread.messages.length - 1)
})

const threadExpandText = computed(() => {
	if (remainingMessageCount.value === 0) return 'Expand'
	if (remainingMessageCount.value === 1) return 'Show 1 more message'
	return `Show ${remainingMessageCount.value} more messages`
})

const selectedFileId = ref<string | null>(null)

const selectedFile = computed(() => {
	if (!selectedFileId.value) return null
	return props.item.reports.find((r) => r.id === selectedFileId.value) ?? null
})

watch(selectedFile, (newFile) => {
	if (selectedFileId.value && (!newFile || newFile.issues.length === 0)) {
		backToFileList()
	}
})

const client = injectModrinthClient()

const allFiles = computed(() => {
	return props.item.reports
})

const highestSeverity = computed(() => {
	const severities = props.item.reports
		.flatMap((r) => r.issues)
		.flatMap((i) => i.details)
		.map((d) => d.severity)

	const order = { severe: 3, high: 2, medium: 1, low: 0 } as Record<string, number>
	return severities.sort((a, b) => (order[b] ?? 0) - (order[a] ?? 0))[0] || 'low'
})

const navTabsLinks = computed(() => {
	const links = tabs.map((tab) => ({
		label: tab as string,
		href: tab.toLowerCase(),
	}))

	if (selectedFile.value) {
		links.push({
			label: selectedFile.value.file_name,
			href: 'file',
		})
	}

	return links
})

const activeTabIndex = computed(() => {
	if (selectedFile.value) {
		return navTabsLinks.value.length - 1
	}
	return tabs.indexOf(currentTab.value)
})

// Handle tab clicks from NavTabs
function handleTabClick(index: number) {
	if (index < tabs.length) {
		currentTab.value = tabs[index]
		backToFileList()
	}
}

function getSeverityBadgeColor(severity: Labrinth.TechReview.Internal.DelphiSeverity): string {
	switch (severity) {
		case 'severe':
			return 'border-red/60 border bg-highlight-red text-red'
		case 'high':
		case 'medium':
			return 'border-orange/60 border bg-highlight-orange text-orange'
		case 'low':
		default:
			return 'border-green/60 border bg-highlight-green text-green'
	}
}

const severityColor = computed(() => {
	switch (highestSeverity.value) {
		case 'severe':
			return 'text-red bg-highlight-red border-solid border-[1px] border-red'
		case 'high':
			return 'text-orange bg-highlight-orange border-solid border-[1px] border-orange'
		case 'medium':
			return 'text-blue bg-highlight-blue border-solid border-[1px] border-blue'
		case 'low':
		default:
			return 'text-green bg-highlight-green border-solid border-[1px] border-green'
	}
})

const formattedDate = computed(() => {
	const dates = props.item.reports.map((r) => new Date(r.created))
	const earliest = new Date(Math.min(...dates.map((d) => d.getTime())))
	const now = new Date()
	const diffDays = Math.floor((now.getTime() - earliest.getTime()) / (1000 * 60 * 60 * 24))
	if (diffDays === 0) return 'Today'
	if (diffDays === 1) return '1 day ago'
	return `${diffDays} days ago`
})

function formatFileSize(bytes: number): string {
	if (bytes < 1024) return `${bytes} B`
	if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KiB`
	return `${(bytes / (1024 * 1024)).toFixed(2)} MiB`
}

function viewFileFlags(file: Labrinth.TechReview.Internal.FileReport) {
	selectedFileId.value = file.id
	emit('loadFileSources', file.id)
}

function backToFileList() {
	selectedFileId.value = null
}

async function copyToClipboard(code: string, detailId: string) {
	try {
		await navigator.clipboard.writeText(code)
		showCopyFeedback.value.set(detailId, true)
		setTimeout(() => {
			showCopyFeedback.value.delete(detailId)
		}, 2000)
	} catch (error) {
		console.error('Failed to copy code:', error)
	}
}

async function updateIssueStatus(
	issueId: string,
	reportId: string,
	status: Labrinth.TechReview.Internal.DelphiReportIssueStatus,
) {
	try {
		await client.labrinth.tech_review_internal.updateIssue(issueId, { status })

		if (status === 'safe') {
			emit('markIssueSafe', props.item.project.id, reportId, issueId)
			addNotification({
				type: 'success',
				title: 'Issue marked as safe',
				text: 'This issue has been marked as a false positive.',
			})
		} else if (status === 'unsafe') {
			emit('markComplete', props.item.project.id)
			addNotification({
				type: 'success',
				title: 'Marked as malware',
				text: 'This issue has been confirmed as malicious. The project has been rejected.',
			})
		}
	} catch (error) {
		console.error('Failed to update issue status:', error)
		addNotification({
			type: 'error',
			title: 'Failed to update issue',
			text: 'An error occurred while updating the issue status.',
		})
	}
}

const expandedIssues = ref<Set<string>>(new Set())
const showCopyFeedback = ref<Map<string, boolean>>(new Map())

const malwareModal = ref<InstanceType<typeof NewModal> | null>(null)
const malwareReason = ref('')
const pendingMalwareIssue = ref<{ issueId: string } | null>(null)

type ActionType = 'safe' | 'malware'
type ButtonState = 'idle' | number | 'completed'

const buttonStates = ref<Map<ActionType, ButtonState>>(new Map())
const buttonIntervals = ref<Map<ActionType, ReturnType<typeof setInterval>>>(new Map())

function getButtonState(action: ActionType): ButtonState {
	return buttonStates.value.get(action) ?? 'idle'
}

// TODO: move this into new buttonstyled refactored component at a later date
async function handleTopLevelAction(action: ActionType) {
	const currentState = getButtonState(action)

	if (typeof currentState === 'number') {
		const intervalId = buttonIntervals.value.get(action)
		if (intervalId) clearInterval(intervalId)
		buttonIntervals.value.delete(action)
		buttonStates.value.delete(action)
		return
	}

	if (currentState === 'completed') return

	buttonStates.value.set(action, 5)

	const intervalId = setInterval(async () => {
		const state = buttonStates.value.get(action)
		if (typeof state === 'number' && state > 1) {
			buttonStates.value.set(action, state - 1)
		} else {
			clearInterval(intervalId)
			buttonIntervals.value.delete(action)

			try {
				const status = action === 'safe' ? 'safe' : 'unsafe'
				await Promise.all(
					props.item.reports.map((report) =>
						client.labrinth.tech_review_internal.updateReport(report.id, { status }),
					),
				)

				buttonStates.value.set(action, 'completed')
				emit('markComplete', props.item.project.id)

				addNotification({
					type: 'success',
					title: action === 'safe' ? 'Marked as safe' : 'Marked as malware',
					text:
						action === 'safe'
							? 'All reports for this project have been marked as safe.'
							: 'All reports for this project have been marked as malware. The project has been rejected.',
				})
			} catch (error) {
				console.error('Failed to update reports:', error)
				buttonStates.value.delete(action)
				addNotification({
					type: 'error',
					title: 'Failed to update reports',
					text: 'An error occurred while updating the report status.',
				})
			}
		}
	}, 1000)

	buttonIntervals.value.set(action, intervalId)
}

onUnmounted(() => {
	buttonIntervals.value.forEach((intervalId) => clearInterval(intervalId))
})

function openMalwareModal(issueId?: string) {
	if (issueId) {
		pendingMalwareIssue.value = { issueId }
	} else {
		pendingMalwareIssue.value = null
	}
	malwareModal.value?.show()
}

async function confirmMalwareAction() {
	try {
		if (pendingMalwareIssue.value) {
			const { issueId } = pendingMalwareIssue.value
			await client.labrinth.tech_review_internal.updateIssue(issueId, {
				status: 'unsafe',
				message: malwareReason.value || undefined,
			})

			emit('markComplete', props.item.project.id)
			malwareModal.value?.hide()
			malwareReason.value = ''
			pendingMalwareIssue.value = null

			addNotification({
				type: 'success',
				title: 'Marked as malware',
				text: 'This issue has been confirmed as malicious. The project has been rejected.',
			})
		} else {
			// top-level
			await Promise.all(
				props.item.reports.map((report) =>
					client.labrinth.tech_review_internal.updateReport(report.id, {
						status: 'unsafe',
						message: malwareReason.value || undefined,
					}),
				),
			)

			buttonStates.value.set('malware', 'completed')
			emit('markComplete', props.item.project.id)
			malwareModal.value?.hide()
			malwareReason.value = ''

			addNotification({
				type: 'success',
				title: 'Marked as malware',
				text: 'All reports for this project have been marked as malware. The project has been rejected.',
			})
		}
	} catch (error) {
		console.error('Failed to update reports:', error)
		addNotification({
			type: 'error',
			title: 'Failed to update reports',
			text: 'An error occurred while updating the report status.',
		})
	}
}

function toggleIssue(issueId: string) {
	if (expandedIssues.value.has(issueId)) {
		expandedIssues.value.delete(issueId)
	} else {
		expandedIssues.value.add(issueId)
	}
}

function handleThreadUpdate() {
	emit('refetch')
}

const techReviewContext = computed<TechReviewContext>(() => ({
	project: props.item.project,
	project_owner: props.item.project_owner,
	reports: props.item.reports,
}))
</script>

<template>
	<div class="shadow-card overflow-hidden rounded-2xl border border-surface-5 bg-surface-3">
		<div class="flex flex-col gap-4 bg-surface-3 p-4">
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-4">
					<Avatar
						:src="item.project.icon_url"
						class="rounded-2xl border border-surface-5 bg-surface-4 !shadow-none"
						size="4rem"
					/>

					<div class="flex flex-col gap-1.5">
						<div class="flex items-center gap-2">
							<NuxtLink
								:to="`/${item.project.project_types[0]}/${item.project.slug ?? item.project.id}`"
								target="_blank"
								class="text-lg font-semibold text-contrast hover:underline"
							>
								{{ item.project.name }}
							</NuxtLink>

							<div
								class="flex items-center gap-1 rounded-full border border-solid border-surface-5 bg-surface-4 px-2.5 py-1"
							>
								<component
									:is="getProjectTypeIcon(item.project.project_types[0] as any)"
									aria-hidden="true"
									class="h-4 w-4"
								/>
								<span
									v-for="project_type in item.project.project_types"
									:key="project_type + item.project.id"
									class="text-sm font-medium text-secondary"
									>{{ formatProjectType(project_type, true) }}</span
								>
							</div>

							<div
								class="rounded-full border border-solid border-surface-5 bg-surface-4 px-2.5 py-1"
							>
								<span class="text-sm font-medium text-secondary">Auto-Flagged</span>
							</div>

							<div class="rounded-full px-2.5 py-1" :class="severityColor">
								<span class="text-sm font-medium">{{
									capitalizeString(highestSeverity.toLowerCase())
								}}</span>
							</div>
						</div>

						<div class="flex items-center gap-1">
							<Avatar
								:src="item.project_owner.icon_url"
								class="rounded-full border border-surface-5 bg-surface-4 !shadow-none"
								size="1.5rem"
								circle
							/>
							<NuxtLink
								:to="`/${item.project_owner.kind}/${item.project_owner.id}`"
								target="_blank"
								class="text-sm font-medium text-secondary hover:underline"
							>
								{{ item.project_owner.name }}
							</NuxtLink>
						</div>
					</div>
				</div>

				<div class="flex items-center gap-3">
					<span class="text-base text-secondary">{{ formattedDate }}</span>
					<div class="flex items-center gap-2">
						<ButtonStyled color="brand">
							<button
								class="!w-[85px] !shadow-none"
								:disabled="getButtonState('malware') !== 'idle'"
								@click="handleTopLevelAction('safe')"
							>
								<LoaderCircleIcon
									v-if="typeof getButtonState('safe') === 'number'"
									class="animate-spin"
								/>
								<CheckCircleIcon v-else-if="getButtonState('safe') === 'completed'" />
								<ShieldCheckIcon v-else />
								{{ typeof getButtonState('safe') === 'number' ? getButtonState('safe') : 'Safe' }}
							</button>
						</ButtonStyled>
						<ButtonStyled color="red">
							<button
								class="!w-[116px] !shadow-none"
								:disabled="getButtonState('safe') !== 'idle'"
								@click="openMalwareModal()"
							>
								<CheckCircleIcon v-if="getButtonState('malware') === 'completed'" />
								<TriangleAlertIcon v-else />
								Malware
							</button>
						</ButtonStyled>
						<ButtonStyled circular>
							<OverflowMenu :options="quickActions" class="!shadow-none">
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
								<template #view-source>
									<CodeIcon />
									<span class="hidden sm:inline">View source</span>
								</template>
							</OverflowMenu>
						</ButtonStyled>
					</div>
				</div>
			</div>

			<div class="h-px w-full bg-surface-5"></div>

			<NavTabs
				mode="local"
				:links="navTabsLinks"
				:active-index="activeTabIndex"
				@tab-click="handleTabClick"
			/>
		</div>

		<div class="border-t border-surface-3 bg-surface-2">
			<template v-if="currentTab === 'Thread'">
				<CollapsibleRegion
					v-model:collapsed="isThreadCollapsed"
					:expand-text="threadExpandText"
					collapse-text="Collapse thread"
					class="border-x border-b border-solid border-surface-3"
				>
					<div class="bg-surface-2 p-4">
						<!-- DEV-531 -->
						<!-- @vue-expect-error TODO: will convert ThreadView to use api-client types at a later date -->
						<ThreadView
							:thread="item.thread"
							:quick-replies="techReviewQuickReplies"
							:quick-reply-context="techReviewContext"
							@update-thread="handleThreadUpdate"
						/>
					</div>
				</CollapsibleRegion>
			</template>

			<template v-else-if="currentTab === 'Files' && !selectedFile">
				<div
					v-for="(file, idx) in allFiles"
					:key="idx"
					class="flex items-center justify-between border-0 border-x border-b border-solid border-surface-3 bg-surface-2 px-4 py-3"
					:class="{
						'rounded-bl-2xl rounded-br-2xl': idx === allFiles.length - 1,
						'bg-[#E8E8E8] dark:bg-[#1A1C20]': idx % 2 === 1,
					}"
				>
					<div class="flex items-center gap-3">
						<span class="font-medium text-contrast">{{ file.file_name }}</span>
						<div class="rounded-full border border-solid border-surface-5 bg-surface-3 px-2.5 py-1">
							<span class="text-sm font-medium text-secondary">{{
								formatFileSize(file.file_size)
							}}</span>
						</div>
						<div
							class="border-red/60 flex items-center gap-1 rounded-full border border-solid bg-highlight-red px-2.5 py-1 text-sm text-red"
						>
							{{ file.issues.length }} flags
						</div>
					</div>

					<div class="flex items-center gap-2">
						<ButtonStyled>
							<button @click="viewFileFlags(file)">Flags</button>
						</ButtonStyled>
						<ButtonStyled type="outlined">
							<a
								:href="file.download_url"
								:title="`Download ${file.file_name}`"
								:download="file.file_name"
								target="_blank"
								rel="noopener noreferrer"
								class="!border-px !border-surface-4"
								tabindex="0"
							>
								<DownloadIcon /> Download
							</a>
						</ButtonStyled>
					</div>
				</div>
			</template>

			<template v-else-if="currentTab === 'Files' && selectedFile">
				<div
					v-for="(issue, idx) in selectedFile.issues"
					:key="issue.id"
					class="border-x border-b border-t-0 border-solid border-surface-3 bg-surface-2 transition-colors duration-200 hover:bg-surface-4"
					:class="{ 'rounded-bl-2xl rounded-br-2xl': idx === selectedFile.issues.length - 1 }"
				>
					<div
						class="flex cursor-pointer items-center justify-between p-4"
						@click="toggleIssue(issue.id)"
					>
						<div class="my-auto flex items-center gap-2">
							<ButtonStyled type="transparent" circular>
								<button
									class="transition-transform"
									:class="{ 'rotate-180': expandedIssues.has(issue.id) }"
								>
									<ChevronDownIcon class="h-5 w-5 text-contrast" />
								</button>
							</ButtonStyled>

							<span class="text-base font-semibold text-contrast">{{
								issue.issue_type.replace(/_/g, ' ')
							}}</span>

							<div
								v-if="issue.details.length > 0"
								class="rounded-full border-solid px-2.5 py-1"
								:class="getSeverityBadgeColor(issue.details[0].severity)"
							>
								<span class="text-sm font-medium">{{
									capitalizeString(issue.details[0].severity)
								}}</span>
							</div>

							<Transition name="fade">
								<div
									v-if="loadingIssues.has(issue.id)"
									class="rounded-full border border-solid border-surface-5 bg-surface-3 px-2.5 py-1"
								>
									<span class="flex items-center gap-1.5 text-sm font-medium text-secondary">
										<LoaderCircleIcon class="size-4 animate-spin" />
										Loading source...
									</span>
								</div>
							</Transition>
						</div>

						<div class="flex items-center gap-2" @click.stop>
							<ButtonStyled color="brand" type="outlined">
								<button
									class="!border-[1px]"
									@click="updateIssueStatus(issue.id, selectedFile.id, 'safe')"
								>
									Safe
								</button>
							</ButtonStyled>

							<ButtonStyled color="red" type="outlined">
								<button class="!border-[1px]" @click="openMalwareModal(issue.id)">Malware</button>
							</ButtonStyled>
						</div>
					</div>

					<Collapsible :collapsed="!expandedIssues.has(issue.id)">
						<div class="flex flex-col gap-4 px-4 pb-4">
							<div
								v-for="(detail, detailIdx) in issue.details"
								:key="detailIdx"
								class="flex flex-col"
							>
								<p class="mt-0 pt-0 font-mono text-sm text-secondary">{{ detail.file_path }}</p>

								<div
									v-if="detail.decompiled_source"
									class="relative overflow-hidden rounded-lg border border-solid border-surface-5 bg-surface-4"
								>
									<ButtonStyled circular type="transparent">
										<button
											v-tooltip="`Copy code`"
											class="absolute right-2 top-2 border-[1px]"
											@click="copyToClipboard(detail.decompiled_source, `${issue.id}-${detailIdx}`)"
										>
											<CopyIcon v-if="!showCopyFeedback.get(`${issue.id}-${detailIdx}`)" />
											<CheckIcon v-else />
										</button>
									</ButtonStyled>

									<div class="overflow-x-auto bg-surface-3 py-3">
										<div
											v-for="(line, n) in highlightCodeLines(detail.decompiled_source, 'java')"
											:key="n"
											class="flex font-mono text-[13px] leading-[1.6]"
										>
											<div
												class="select-none border-0 border-r border-solid border-surface-5 px-4 py-0 text-right text-primary"
												style="min-width: 3.5rem"
											>
												{{ n + 1 }}
											</div>
											<div class="flex-1 px-4 py-0 text-primary">
												<pre v-html="line || ' '"></pre>
											</div>
										</div>
									</div>
								</div>
								<div
									v-else
									class="rounded-lg border border-solid border-surface-5 bg-surface-3 p-4"
								>
									<p class="text-sm text-secondary">
										Source code not available or failed to decompile for this flag.
									</p>
								</div>
							</div>
						</div>
					</Collapsible>
				</div>
			</template>
		</div>
	</div>

	<NewModal ref="malwareModal" header="Confirm Malware" fade="danger">
		<div class="flex flex-col gap-4">
			<div class="flex flex-col gap-1">
				<label class="text-md font-semibold text-contrast">Rejection reason (optional)</label>
				<MarkdownEditor
					v-model="malwareReason"
					placeholder="Explain why this is malware/being rejected..."
					:heading-buttons="false"
				/>
			</div>
		</div>
		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled type="outlined">
					<button @click="malwareModal?.hide()">Cancel</button>
				</ButtonStyled>
				<ButtonStyled color="red">
					<button @click="confirmMalwareAction">
						<TriangleAlertIcon />
						Confirm Malware
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<style scoped>
pre {
	all: unset;
	display: inline;
	white-space: pre;
}

.fade-enter-active {
	transition: opacity 0.3s ease-in;
	transition-delay: 0.2s;
}

.fade-leave-active {
	transition: opacity 0.15s ease-out;
}

.fade-enter-from,
.fade-leave-to {
	opacity: 0;
}
</style>
