<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	BugIcon,
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
	OverflowMenu,
	type OverflowMenuOption,
} from '@modrinth/ui'
import {
	capitalizeString,
	formatProjectType,
	highlightCodeLines,
	type ThreadMessage,
	type User,
} from '@modrinth/utils'
import dayjs from 'dayjs'
import { computed, ref, watch } from 'vue'

import type { UnsafeFile } from '~/components/ui/moderation/MaliciousSummaryModal.vue'
import NavTabs from '~/components/ui/NavTabs.vue'
import ThreadView from '~/components/ui/thread/ThreadView.vue'

const auth = await useAuth()
const featureFlags = useFeatureFlags()

type FlattenedFileReport = Labrinth.TechReview.Internal.FileReport & {
	id: string
	version_id: string
}

interface FileDecisions {
	fileName: string
	fileSize: number
	decisions: Array<{
		filePath: string
		issueType: string
		severity: string
		decision: 'safe' | 'malware'
	}>
	maxSeverity: string
}

const props = defineProps<{
	item: {
		project: Labrinth.Projects.v3.Project
		project_owner: Labrinth.TechReview.Internal.Ownership
		thread: Labrinth.TechReview.Internal.Thread
		reports: FlattenedFileReport[]
	}
	loadingIssues: Set<string>
	decompiledSources: Map<string, string>
}>()

const { addNotification } = injectNotificationManager()

const emit = defineEmits<{
	refetch: []
	loadFileSources: [reportId: string]
	markComplete: [projectId: string]
	showMaliciousSummary: [unsafeFiles: UnsafeFile[]]
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
						title: 'Technical Review link copied',
						text: 'The link to this review has been copied to your clipboard.',
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

type Tab = 'Thread' | 'Files' | 'File'
const tabs: readonly ('Thread' | 'Files')[] = ['Thread', 'Files']
const currentTab = ref<Tab>('Thread')

const isThreadCollapsed = ref(true)

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

const severityOrder = { severe: 3, high: 2, medium: 1, low: 0 } as Record<string, number>

const detailDecisions = ref<Map<string, 'safe' | 'malware'>>(new Map())
const updatingDetails = ref<Set<string>>(new Set())

function getFileHighestSeverity(
	file: FlattenedFileReport,
): Labrinth.TechReview.Internal.DelphiSeverity {
	const severities = file.issues
		.flatMap((i) => i.details ?? [])
		.map((d) => d.severity)
		.filter((s): s is Labrinth.TechReview.Internal.DelphiSeverity => !!s)

	return severities.sort((a, b) => (severityOrder[b] ?? 0) - (severityOrder[a] ?? 0))[0] || 'low'
}

const allFiles = ref<FlattenedFileReport[]>([])

watch(
	() => props.item.reports,
	(reports) => {
		allFiles.value = [...reports].sort((a, b) => {
			const aComplete = getFileMarkedCount(a) === getFileDetailCount(a)
			const bComplete = getFileMarkedCount(b) === getFileDetailCount(b)
			if (aComplete !== bComplete) return aComplete ? 1 : -1
			const aSeverity = getFileHighestSeverity(a)
			const bSeverity = getFileHighestSeverity(b)
			return (severityOrder[bSeverity] ?? 0) - (severityOrder[aSeverity] ?? 0)
		})
	},
	{ immediate: true },
)

const highestSeverity = computed(() => {
	const severities = props.item.reports
		.flatMap((r) => r.issues ?? [])
		.flatMap((i) => i.details ?? [])
		.map((d) => d.severity)
		.filter((s): s is Labrinth.TechReview.Internal.DelphiSeverity => !!s)

	return severities.sort((a, b) => (severityOrder[b] ?? 0) - (severityOrder[a] ?? 0))[0] || 'low'
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
	if (currentTab.value === 'File' && selectedFile.value) {
		return navTabsLinks.value.length - 1
	}
	const idx = tabs.indexOf(currentTab.value as 'Thread' | 'Files')
	return idx >= 0 ? idx : 0
})

function handleTabClick(index: number) {
	if (index < tabs.length) {
		const newTab = tabs[index]
		currentTab.value = newTab

		if (newTab === 'Thread') {
			isThreadCollapsed.value = false
		}
	} else if (index === tabs.length && selectedFile.value) {
		// Clicked the file tab
		currentTab.value = 'File' as Tab
	}
}

function getSeverityBadgeColor(severity: Labrinth.TechReview.Internal.DelphiSeverity): string {
	switch (severity) {
		case 'severe':
			return 'border-red/60 border bg-highlight-red text-red'
		case 'high':
			return 'border-orange/60 border bg-highlight-orange text-orange'
		case 'medium':
			return 'border-green/60 border bg-highlight-green text-green'
		case 'low':
		default:
			return 'border-blue/60 border bg-highlight-blue text-blue'
	}
}

const severityColor = computed(() => {
	switch (highestSeverity.value) {
		case 'severe':
			return 'text-red bg-highlight-red border-solid border-[1px] border-red'
		case 'high':
			return 'text-orange bg-highlight-orange border-solid border-[1px] border-orange'
		case 'medium':
			return 'text-green bg-highlight-green border-solid border-[1px] border-green'
		case 'low':
		default:
			return 'text-blue bg-highlight-blue border-solid border-[1px] border-blue'
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

function viewFileFlags(file: FlattenedFileReport) {
	selectedFileId.value = file.id
	currentTab.value = 'File'
	emit('loadFileSources', file.id)
}

function backToFileList() {
	selectedFileId.value = null
	if (currentTab.value === 'File') {
		currentTab.value = 'Files'
	}
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

function getDetailDecision(
	detailId: string,
	backendStatus: Labrinth.TechReview.Internal.DelphiReportIssueStatus,
): 'safe' | 'malware' | 'pending' {
	const localDecision = detailDecisions.value.get(detailId)
	if (localDecision) return localDecision
	if (backendStatus === 'safe') return 'safe'
	if (backendStatus === 'unsafe') return 'malware'
	return 'pending'
}

function isPreReviewed(
	detailId: string,
	backendStatus: Labrinth.TechReview.Internal.DelphiReportIssueStatus,
): boolean {
	return (
		(backendStatus === 'safe' || backendStatus === 'unsafe') && !detailDecisions.value.has(detailId)
	)
}

function getMarkedFlagsCount(flags: ClassGroup['flags']): number {
	return flags.filter((f) => getDetailDecision(f.detail.id, f.detail.status) !== 'pending').length
}

function getFileDetailCount(file: FlattenedFileReport): number {
	return file.issues.reduce((sum, issue) => sum + issue.details.length, 0)
}

function getFileMarkedCount(file: FlattenedFileReport): number {
	let count = 0
	for (const issue of file.issues) {
		for (const detail of issue.details) {
			const detailWithStatus = detail as typeof detail & {
				status: Labrinth.TechReview.Internal.DelphiReportIssueStatus
			}
			if (getDetailDecision(detailWithStatus.id, detailWithStatus.status) !== 'pending') {
				count++
			}
		}
	}
	return count
}

async function updateDetailStatus(detailId: string, verdict: 'safe' | 'unsafe') {
	updatingDetails.value.add(detailId)

	try {
		await client.labrinth.tech_review_internal.updateIssueDetail(detailId, { verdict })

		const decision = verdict === 'safe' ? 'safe' : 'malware'

		let detailKey: string | null = null
		for (const report of props.item.reports) {
			for (const issue of report.issues) {
				const detail = issue.details.find((d) => d.id === detailId)
				if (detail) {
					detailKey = detail.key
					break
				}
			}
			if (detailKey) break
		}

		if (detailKey) {
			for (const report of props.item.reports) {
				for (const issue of report.issues) {
					for (const detail of issue.details) {
						if (detail.key === detailKey) {
							detailDecisions.value.set(detail.id, decision)
						}
					}
				}
			}
		} else {
			detailDecisions.value.set(detailId, decision)
		}

		for (const classGroup of groupedByClass.value) {
			const hasThisDetail = classGroup.flags.some((f) => f.detail.id === detailId)
			if (hasThisDetail && getMarkedFlagsCount(classGroup.flags) === classGroup.flags.length) {
				expandedClasses.value.delete(classGroup.filePath)
				break
			}
		}

		if (verdict === 'safe') {
			addNotification({
				type: 'success',
				title: 'Issue marked as pass',
				text: 'This issue has been marked as a false positive.',
			})
		} else {
			addNotification({
				type: 'success',
				title: 'Issue marked as fail',
				text: 'This issue has been flagged as malicious.',
			})
		}
	} catch (error) {
		console.error('Failed to update detail status:', error)
		addNotification({
			type: 'error',
			title: 'Failed to update issue',
			text: 'An error occurred while updating the issue status.',
		})
	} finally {
		updatingDetails.value.delete(detailId)
	}
}

const expandedClasses = ref<Set<string>>(new Set())
const showCopyFeedback = ref<Map<string, boolean>>(new Map())

interface ClassGroup {
	filePath: string
	flags: Array<{
		issueId: string
		issueType: string
		detail: Labrinth.TechReview.Internal.ReportIssueDetail & {
			status: Labrinth.TechReview.Internal.DelphiReportIssueStatus
		}
	}>
}

const groupedByClass = computed<ClassGroup[]>(() => {
	if (!selectedFile.value) return []

	const classMap = new Map<string, ClassGroup>()

	for (const issue of selectedFile.value.issues) {
		for (const detail of issue.details) {
			if (!classMap.has(detail.file_path)) {
				classMap.set(detail.file_path, { filePath: detail.file_path, flags: [] })
			}
			// Cast detail to include status (backend will provide this field)
			const detailWithStatus = detail as Labrinth.TechReview.Internal.ReportIssueDetail & {
				status: Labrinth.TechReview.Internal.DelphiReportIssueStatus
			}
			classMap.get(detail.file_path)!.flags.push({
				issueId: issue.id,
				issueType: issue.issue_type,
				detail: detailWithStatus,
			})
		}
	}

	for (const classGroup of classMap.values()) {
		classGroup.flags.sort((a, b) => {
			const aPreReviewed = isPreReviewed(a.detail.id, a.detail.status)
			const bPreReviewed = isPreReviewed(b.detail.id, b.detail.status)

			if (aPreReviewed !== bPreReviewed) {
				return aPreReviewed ? 1 : -1
			}

			return (severityOrder[b.detail.severity] ?? 0) - (severityOrder[a.detail.severity] ?? 0)
		})
	}

	return Array.from(classMap.values()).sort((a, b) => {
		const aSeverity = getHighestSeverityInClass(a.flags)
		const bSeverity = getHighestSeverityInClass(b.flags)
		return (severityOrder[bSeverity] ?? 0) - (severityOrder[aSeverity] ?? 0)
	})
})

function getHighestSeverityInClass(
	flags: ClassGroup['flags'],
): Labrinth.TechReview.Internal.DelphiSeverity {
	return flags.reduce(
		(highest, flag) =>
			(severityOrder[flag.detail.severity] ?? 0) > (severityOrder[highest] ?? 0)
				? flag.detail.severity
				: highest,
		'low' as Labrinth.TechReview.Internal.DelphiSeverity,
	)
}

function toggleClass(filePath: string) {
	if (expandedClasses.value.has(filePath)) {
		expandedClasses.value.delete(filePath)
	} else {
		expandedClasses.value.add(filePath)
	}
}

function getClassDecompiledSource(classItem: ClassGroup): string | undefined {
	for (const flag of classItem.flags) {
		const source = props.decompiledSources.get(flag.detail.id)
		if (source) return source
	}
	return undefined
}

function handleThreadUpdate() {
	emit('refetch')
}

const techReviewContext = computed<TechReviewContext>(() => ({
	project: props.item.project,
	project_owner: props.item.project_owner,
	reports: props.item.reports,
}))

const threadViewRef = ref<{
	setReplyContent: (content: string) => void
	getReplyContent: () => string
} | null>(null)

const unsafeFiles = computed<UnsafeFile[]>(() => {
	return props.item.reports
		.filter((report) =>
			report.issues.some((issue) =>
				issue.details.some((detail) => {
					const detailWithStatus = detail as typeof detail & {
						status: Labrinth.TechReview.Internal.DelphiReportIssueStatus
					}
					const decision = getDetailDecision(detailWithStatus.id, detailWithStatus.status)
					return decision === 'malware'
				}),
			),
		)
		.map((report) => ({
			file: report,
			projectName: props.item.project.name,
			projectId: props.item.project.id,
			userId: props.item.project_owner.id,
			username: props.item.project_owner.name,
		}))
})

const reviewSummaryPreview = computed(() => {
	const fileDecisions = new Map<string, FileDecisions>()
	let totalSafe = 0
	let totalUnsafe = 0

	for (const report of props.item.reports) {
		if (!fileDecisions.has(report.id)) {
			fileDecisions.set(report.id, {
				fileName: report.file_name,
				fileSize: report.file_size,
				decisions: [],
				maxSeverity: 'low',
			})
		}
		const fileData = fileDecisions.get(report.id)!

		for (const issue of report.issues) {
			for (const detail of issue.details) {
				// TODO: proper types when backend pushes
				const detailWithStatus = detail as typeof detail & {
					status: Labrinth.TechReview.Internal.DelphiReportIssueStatus
				}
				const decision = getDetailDecision(detailWithStatus.id, detailWithStatus.status)
				if (decision === 'pending') continue

				fileData.decisions.push({
					filePath: detail.file_path,
					issueType: issue.issue_type.replace(/_/g, ' '),
					severity: detail.severity,
					decision,
				})

				if ((severityOrder[detail.severity] ?? 0) > (severityOrder[fileData.maxSeverity] ?? 0)) {
					fileData.maxSeverity = detail.severity
				}

				if (decision === 'safe') totalSafe++
				else totalUnsafe++
			}
		}
	}

	const totalDecisions = totalSafe + totalUnsafe
	if (totalDecisions === 0) return ''

	const timestamp = dayjs().utc().format('MMMM D, YYYY [at] h:mm A [UTC]')
	let markdown = `## Tech Review Summary\n*${timestamp}*\n\n`

	for (const [, fileData] of fileDecisions) {
		if (fileData.decisions.length === 0) continue

		const fileSafe = fileData.decisions.filter((d) => d.decision === 'safe').length
		const fileUnsafe = fileData.decisions.filter((d) => d.decision === 'malware').length
		const fileVerdict = fileUnsafe > 0 ? 'Unsafe' : 'Safe'

		markdown += `### ${fileData.fileName}\n`
		markdown += `> ${formatFileSize(fileData.fileSize)} • ${fileData.decisions.length} issues • Max severity: ${fileData.maxSeverity} • **Verdict:** ${fileVerdict}\n\n`
		markdown += `<details>\n<summary>Issues (${fileSafe} safe, ${fileUnsafe} unsafe)</summary>\n\n`
		markdown += `| Class | Issue Type | Severity | Decision |\n`
		markdown += `|-------|------------|----------|----------|\n`

		for (const d of fileData.decisions) {
			const decisionText = d.decision === 'safe' ? '✅ Safe' : '❌ Unsafe'
			markdown += `| \`${d.filePath}\` | ${d.issueType} | ${capitalizeString(d.severity)} | ${decisionText} |\n`
		}

		markdown += `\n</details>\n\n`
	}

	markdown += `---\n\n**Total:** ${totalDecisions} issues reviewed (${totalSafe} safe, ${totalUnsafe} unsafe)\n\n`

	return markdown
})

const threadWithPreview = computed(() => {
	if (!reviewSummaryPreview.value) return props.item.thread

	const user = auth.value?.user as User | null
	if (!user) return props.item.thread

	const previewMessage: ThreadMessage & { preview: true } = {
		id: 'preview-message',
		author_id: user.id,
		body: {
			type: 'text',
			body: reviewSummaryPreview.value,
			private: false,
			replying_to: null,
			associated_images: [],
		},
		created: new Date().toISOString(),
		hide_identity: false,
		preview: true,
	}

	return {
		...props.item.thread,
		messages: [...props.item.thread.messages, previewMessage],
		members: props.item.thread.members.some((m) => m.id === user.id)
			? props.item.thread.members
			: [...props.item.thread.members, user],
	}
})

const allIssuesResolved = computed(() => {
	for (const report of props.item.reports) {
		for (const issue of report.issues) {
			for (const detail of issue.details) {
				const detailWithStatus = detail as typeof detail & {
					status: Labrinth.TechReview.Internal.DelphiReportIssueStatus
				}
				const decision = getDetailDecision(detailWithStatus.id, detailWithStatus.status)
				if (decision === 'pending') return false
			}
		}
	}
	return true
})

const canSubmitReview = computed(() => {
	const totalIssues = props.item.reports.reduce((sum, r) => sum + r.issues.length, 0)
	if (totalIssues === 0) return true
	return allIssuesResolved.value
})

async function handleSubmitReview(verdict: 'safe' | 'unsafe') {
	const editorContent = threadViewRef.value?.getReplyContent() || ''

	let message: string | undefined
	if (reviewSummaryPreview.value && editorContent) {
		message = `${reviewSummaryPreview.value}${editorContent}`
	} else if (reviewSummaryPreview.value) {
		message = reviewSummaryPreview.value
	} else if (editorContent) {
		message = editorContent
	}

	try {
		await client.labrinth.tech_review_internal.submitProject(props.item.project.id, {
			verdict,
			message,
		})
		emit('markComplete', props.item.project.id)
		addNotification({
			type: 'success',
			title: 'Review submitted',
			text: 'Technical review completed successfully.',
		})

		if (verdict === 'unsafe') {
			emit('showMaliciousSummary', unsafeFiles.value)
		}
	} catch (error: unknown) {
		const err = error as { response?: { data?: { issues?: string[] } } }
		if (err.response?.data?.issues) {
			const missedCount = err.response.data.issues.length
			addNotification({
				type: 'error',
				title: 'Pending issues remain',
				text: `${missedCount} issue(s) still need a verdict before submitting.`,
			})
		} else {
			addNotification({
				type: 'error',
				title: 'Submit failed',
				text: 'Failed to submit review. Please try again.',
			})
		}
	}
}
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
							<span class="text-tertiary text-sm">({{ item.project_owner.id }})</span>
						</div>
					</div>
				</div>

				<div class="flex items-center gap-3">
					<span class="text-base text-secondary">{{ formattedDate }}</span>
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
					<div class="bg-surface-2 p-4 pt-0">
						<!-- DEV-531 -->
						<!-- @vue-expect-error TODO: will convert ThreadView to use api-client types at a later date -->
						<ThreadView
							ref="threadViewRef"
							:thread="threadWithPreview"
							:quick-replies="techReviewQuickReplies"
							:quick-reply-context="techReviewContext"
							@update-thread="handleThreadUpdate"
						>
							<template #additionalActions>
								<ButtonStyled color="brand">
									<button
										v-tooltip="!canSubmitReview ? 'There are still pending flags!' : undefined"
										:disabled="!canSubmitReview"
										@click="handleSubmitReview('safe')"
									>
										<ShieldCheckIcon /> Pass
									</button>
								</ButtonStyled>
								<ButtonStyled color="red">
									<button
										v-tooltip="!canSubmitReview ? 'There are still pending flags!' : undefined"
										:disabled="!canSubmitReview"
										@click="handleSubmitReview('unsafe')"
									>
										<BugIcon /> Fail
									</button>
								</ButtonStyled>
								<ButtonStyled v-if="featureFlags.developerMode" type="outlined">
									<button @click="emit('showMaliciousSummary', unsafeFiles)">Debug Summary</button>
								</ButtonStyled>
							</template>
						</ThreadView>
					</div>
				</CollapsibleRegion>
			</template>

			<template v-else-if="currentTab === 'Files'">
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
						<span
							class="font-medium text-contrast"
							:class="{ 'cursor-pointer hover:underline': getFileDetailCount(file) > 0 }"
							@click="getFileDetailCount(file) > 0 && viewFileFlags(file)"
						>
							{{ file.file_name }}
						</span>
						<div class="rounded-full border border-solid border-surface-5 bg-surface-3 px-2.5 py-1">
							<span class="text-sm font-medium text-secondary">{{
								formatFileSize(file.file_size)
							}}</span>
						</div>
						<div
							v-if="getFileDetailCount(file) > 0"
							class="rounded-full border-solid px-2.5 py-1"
							:class="getSeverityBadgeColor(getFileHighestSeverity(file))"
						>
							<span class="text-sm font-medium">{{
								capitalizeString(getFileHighestSeverity(file))
							}}</span>
						</div>
						<div
							v-if="getFileDetailCount(file) > 0"
							class="flex items-center gap-1 rounded-full border border-solid px-2.5 py-1 text-sm"
							:class="
								getFileMarkedCount(file) === getFileDetailCount(file)
									? 'border-green/60 bg-highlight-green text-green'
									: 'border-red/60 bg-highlight-red text-red'
							"
						>
							<CheckIcon
								v-if="getFileMarkedCount(file) === getFileDetailCount(file)"
								class="size-4"
							/>
							{{ getFileMarkedCount(file) }}/{{ getFileDetailCount(file) }} flags
						</div>
						<!-- TODO: remove toString when backend supports it properly -->
						<div
							v-else-if="file.flag_reason.toString() === 'manual'"
							class="border-blue/60 flex items-center gap-1 rounded-full border border-solid bg-highlight-blue px-2.5 py-1 text-sm text-blue"
						>
							Manual review
						</div>
						<div
							v-else
							class="border-green/60 flex items-center gap-1 rounded-full border border-solid bg-highlight-green px-2.5 py-1 text-sm text-green"
						>
							No flags
						</div>
					</div>

					<div class="flex items-center gap-2">
						<ButtonStyled v-if="getFileDetailCount(file) > 0">
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

			<template v-else-if="currentTab === 'File' && selectedFile">
				<div
					v-for="(classItem, idx) in groupedByClass"
					:key="classItem.filePath"
					class="border-x border-b border-t-0 border-solid border-surface-3 bg-surface-2"
					:class="{ 'rounded-bl-2xl rounded-br-2xl': idx === groupedByClass.length - 1 }"
				>
					<div
						class="flex cursor-pointer items-center justify-between p-4 transition-colors duration-200 hover:bg-surface-4"
						@click="toggleClass(classItem.filePath)"
					>
						<div class="my-auto flex items-center gap-2">
							<ButtonStyled type="transparent" circular>
								<button
									class="transition-transform"
									:class="{ 'rotate-180': expandedClasses.has(classItem.filePath) }"
								>
									<ChevronDownIcon class="h-5 w-5 text-contrast" />
								</button>
							</ButtonStyled>

							<span class="font-mono font-semibold">{{ classItem.filePath }}</span>

							<div
								class="rounded-full border-solid px-2.5 py-1"
								:class="getSeverityBadgeColor(getHighestSeverityInClass(classItem.flags))"
							>
								<span class="text-sm font-medium">{{
									capitalizeString(getHighestSeverityInClass(classItem.flags))
								}}</span>
							</div>

							<div
								class="flex items-center gap-1 rounded-full border border-solid px-2.5 py-1 text-sm"
								:class="
									getMarkedFlagsCount(classItem.flags) === classItem.flags.length
										? 'border-green/60 bg-highlight-green text-green'
										: 'border-red/60 bg-highlight-red text-red'
								"
							>
								<CheckIcon
									v-if="getMarkedFlagsCount(classItem.flags) === classItem.flags.length"
									class="size-4"
								/>
								{{ getMarkedFlagsCount(classItem.flags) }}/{{ classItem.flags.length }} flags
							</div>

							<Transition name="fade">
								<div
									v-if="classItem.flags.some((f) => loadingIssues.has(f.issueId))"
									class="rounded-full border border-solid border-surface-5 bg-surface-3 px-2.5 py-1"
								>
									<span class="flex items-center gap-1.5 text-sm font-medium text-secondary">
										<LoaderCircleIcon class="size-4 animate-spin" />
										Loading source...
									</span>
								</div>
							</Transition>
						</div>
					</div>

					<Collapsible :collapsed="!expandedClasses.has(classItem.filePath)">
						<div class="mt-2 flex flex-col gap-2 px-4 pb-4">
							<div
								v-for="flag in classItem.flags"
								:key="`${flag.issueId}-${flag.detail.id}`"
								class="grid grid-cols-[1fr_auto_auto] items-center rounded-lg border-[1px] border-b border-solid border-surface-5 bg-surface-3 py-2 pl-4 last:border-b-0"
								:class="{
									'opacity-50': isPreReviewed(flag.detail.id, flag.detail.status),
								}"
							>
								<span class="text-base font-semibold text-contrast">{{
									flag.issueType.replace(/_/g, ' ')
								}}</span>

								<div class="flex w-20 justify-center">
									<div
										class="rounded-full border-solid px-2.5 py-1"
										:class="getSeverityBadgeColor(flag.detail.severity)"
									>
										<span class="text-sm font-medium">{{
											capitalizeString(flag.detail.severity)
										}}</span>
									</div>
								</div>

								<div class="flex w-40 items-center justify-center gap-2">
									<ButtonStyled
										color="brand"
										:type="
											getDetailDecision(flag.detail.id, flag.detail.status) === 'safe'
												? undefined
												: 'outlined'
										"
									>
										<button
											class="!border-[1px]"
											:disabled="updatingDetails.has(flag.detail.id)"
											@click="updateDetailStatus(flag.detail.id, 'safe')"
										>
											Pass
										</button>
									</ButtonStyled>

									<ButtonStyled
										color="red"
										:type="
											getDetailDecision(flag.detail.id, flag.detail.status) === 'malware'
												? undefined
												: 'outlined'
										"
									>
										<button
											class="!border-[1px]"
											:disabled="updatingDetails.has(flag.detail.id)"
											@click="updateDetailStatus(flag.detail.id, 'unsafe')"
										>
											Fail
										</button>
									</ButtonStyled>
								</div>
							</div>

							<div
								v-if="getClassDecompiledSource(classItem)"
								class="relative inset-0 overflow-hidden rounded-lg border border-solid border-surface-5 bg-surface-4"
							>
								<ButtonStyled circular type="transparent">
									<button
										v-tooltip="`Copy code`"
										class="absolute right-2 top-2 border-[1px]"
										@click="
											copyToClipboard(getClassDecompiledSource(classItem)!, classItem.filePath)
										"
									>
										<CopyIcon v-if="!showCopyFeedback.get(classItem.filePath)" />
										<CheckIcon v-else />
									</button>
								</ButtonStyled>

								<div class="overflow-x-auto bg-surface-3 py-3">
									<div
										v-for="(line, n) in highlightCodeLines(
											getClassDecompiledSource(classItem)!,
											'java',
										)"
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
							<div v-else class="rounded-lg border border-solid border-surface-5 bg-surface-3 p-4">
								<p class="text-sm text-secondary">
									Source code not available or failed to decompile for this file.
								</p>
							</div>
						</div>
					</Collapsible>
				</div>
			</template>
		</div>
	</div>
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
