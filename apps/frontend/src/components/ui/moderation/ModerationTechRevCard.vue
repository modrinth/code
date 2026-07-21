<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	BanIcon,
	BugIcon,
	CheckCheckIcon,
	CheckIcon,
	ChevronDownIcon,
	ChevronRightIcon,
	ClipboardCopyIcon,
	CodeIcon,
	CopyIcon,
	DownloadIcon,
	ExternalIcon,
	EyeOffIcon,
	LoaderCircleIcon,
	ScaleIcon,
	ShieldAlertIcon,
	ShieldCheckIcon,
	SpinnerIcon,
	TimerIcon,
	XIcon,
} from '@modrinth/assets'
import { type TechReviewContext, techReviewQuickReplies } from '@modrinth/moderation'
import {
	Avatar,
	ButtonStyled,
	Collapsible,
	CollapsibleRegion,
	commonMessages,
	getProjectTypeIcon,
	injectModrinthClient,
	injectNotificationManager,
	NavTabs,
	OverflowMenu,
	type OverflowMenuOption,
	Toggle,
	useFormatBytes,
	useFormatDateTime,
	useVIntl,
} from '@modrinth/ui'
import {
	capitalizeString,
	formatProjectType,
	highlightCodeLines,
	type ThreadMessage,
	type User,
} from '@modrinth/utils'
import dayjs from 'dayjs'
import { computed, nextTick, reactive, ref, watch } from 'vue'

import type { UnsafeFile } from '~/components/ui/moderation/MaliciousSummaryModal.vue'
import ThreadView from '~/components/ui/thread/ThreadView.vue'

const auth = await useAuth()
const featureFlags = useFeatureFlags()
const { formatMessage } = useVIntl()

const formatDateTimeUtc = useFormatDateTime({
	year: 'numeric',
	month: 'long',
	day: 'numeric',
	hour: 'numeric',
	minute: '2-digit',
	timeZoneName: 'short',
	timeZone: 'UTC',
})
const formatBytes = useFormatBytes()

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
	focusedDetailId?: string | null
	loadingIssues: Set<string>
	decompiledSources: Map<string, string>
}>()

const { addNotification } = injectNotificationManager()

const emit = defineEmits<{
	refetch: []
	loadIssueSources: [issueIds: string[]]
	markComplete: [projectId: string]
	showMaliciousSummary: [unsafeFiles: UnsafeFile[]]
}>()

const projectStatus = ref<Labrinth.Projects.v2.ProjectStatus>(props.item.project.status)
const isProjectApproved = computed(() => {
	return (
		projectStatus.value === 'approved' ||
		projectStatus.value === 'archived' ||
		projectStatus.value === 'unlisted' ||
		projectStatus.value === 'private'
	)
})

const isLoadingStatusAction = ref(false)
const projectStatusActions = computed<OverflowMenuOption[]>(() => [
	{
		id: 'approve',
		color: 'green',
		action: () => setStatus('approved'),
		hoverFilled: true,
		disabled: isStatusActionDisabled('approved'),
	},
	{
		id: 'withhold',
		color: 'orange',
		action: () => setStatus('withheld'),
		hoverFilled: true,
		disabled: isStatusActionDisabled('withheld'),
	},
	{
		id: 'send-to-review',
		action: () => setStatus('processing'),
		hoverFilled: true,
		disabled: isStatusActionDisabled('processing'),
	},
	{
		id: 'reject',
		color: 'red',
		action: () => setStatus('rejected'),
		hoverFilled: true,
		disabled: isStatusActionDisabled('rejected'),
	},
])

function isStatusActionDisabled(status: Labrinth.Projects.v2.ProjectStatus): boolean {
	const currentStatus = projectStatus.value
	const isLoading = isLoadingStatusAction.value
	return currentStatus === status || isLoading
}

async function setStatus(status: Labrinth.Projects.v2.ProjectStatus) {
	isLoadingStatusAction.value = true
	try {
		await client.labrinth.projects_v2.edit(props.item.project.id, { status })
		emit('refetch')

		projectStatus.value = status
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: (err as any)?.data?.description ? (err as any).data.description : String(err),
			type: 'error',
		})
	}
	isLoadingStatusAction.value = false
}

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

async function updateIssueDetails(
	data: {
		detail_id: string
		verdict: Labrinth.TechReview.Internal.DelphiReportIssueStatus
	}[],
) {
	await client.request('/moderation/tech-review/issue-detail', {
		api: 'labrinth',
		version: 'internal',
		method: 'PATCH',
		body: data,
	})
}

async function updateGlobalIssueDetail(
	detailKey: string,
	verdict: Labrinth.TechReview.Internal.DelphiReportIssueStatus,
) {
	await client.labrinth.tech_review_internal.updateGlobalIssueDetails([
		{ detail_key: detailKey, verdict },
	])
}

const severityOrder = { severe: 3, high: 2, medium: 1, low: 0 } as Record<string, number>

type DetailDecision = 'safe' | 'malware' | 'pending'
type DetailDecisionScope = 'local' | 'global'

const detailDecisions = reactive<Map<string, DetailDecision>>(new Map())
const detailDecisionScopes = reactive<Map<string, DetailDecisionScope>>(new Map())
const updatingDetails = reactive<Set<string>>(new Set())
const updatingGlobalDetailKeys = reactive<Set<string>>(new Set())

function verdictToDecision(
	verdict: Labrinth.TechReview.Internal.DelphiReportIssueStatus,
): DetailDecision {
	if (verdict === 'safe') return 'safe'
	if (verdict === 'unsafe') return 'malware'
	return 'pending'
}

function getAllDetails(): Labrinth.TechReview.Internal.ReportIssueDetail[] {
	return props.item.reports.flatMap((report) => report.issues.flatMap((issue) => issue.details))
}

const hideGloballyPassed = ref(true)

function isDetailGloballyPassed(detail: Labrinth.TechReview.Internal.ReportIssueDetail): boolean {
	if (detailDecisionScopes.get(detail.id) === 'global') {
		return detailDecisions.get(detail.id) === 'safe'
	}

	return detail.global_status === 'safe'
}

function isDetailGloballyResolved(detail: Labrinth.TechReview.Internal.ReportIssueDetail): boolean {
	if (detailDecisionScopes.get(detail.id) === 'global') {
		return detailDecisions.get(detail.id) !== 'pending'
	}

	return detail.global_status === 'safe' || detail.global_status === 'unsafe'
}

const globallyPassedSelectedFileCount = computed(() => {
	if (!selectedFile.value) return 0

	return selectedFile.value.issues.reduce(
		(count, issue) => count + issue.details.filter(isDetailGloballyPassed).length,
		0,
	)
})

const globallyResolvedSelectedFileCount = computed(() => {
	if (!selectedFile.value) return 0

	return selectedFile.value.issues.reduce(
		(count, issue) => count + issue.details.filter(isDetailGloballyResolved).length,
		0,
	)
})

function applyDecisionToRelatedDetails(
	detailIds: string[],
	decision: DetailDecision,
	scope: DetailDecisionScope,
): { otherMatchedCount: number } {
	const allDetails = getAllDetails()
	const selectedDetailIds = new Set(detailIds)
	const updatedDetailIds = new Set<string>()

	for (const detailId of detailIds) {
		const detail = allDetails.find((candidate) => candidate.id === detailId)
		let matchingDetails: Labrinth.TechReview.Internal.ReportIssueDetail[] = []

		if (detail?.key) {
			matchingDetails = allDetails.filter((candidate) => candidate.key === detail.key)
		} else if (detail) {
			matchingDetails = [detail]
		}

		if (matchingDetails.length === 0) {
			detailDecisions.set(detailId, decision)
			detailDecisionScopes.set(detailId, scope)
			updatedDetailIds.add(detailId)
			continue
		}

		for (const matchingDetail of matchingDetails) {
			detailDecisions.set(matchingDetail.id, decision)
			detailDecisionScopes.set(matchingDetail.id, scope)
			updatedDetailIds.add(matchingDetail.id)
		}
	}

	return {
		otherMatchedCount: [...updatedDetailIds].filter((detailId) => !selectedDetailIds.has(detailId))
			.length,
	}
}

function statusMatchesDecision(
	status: Labrinth.TechReview.Internal.DelphiReportIssueStatus | null,
	decision: DetailDecision,
): boolean {
	if (status === 'safe') return decision === 'safe'
	if (status === 'unsafe') return decision === 'malware'
	return false
}

function isDetailActionSelected(
	detail: Labrinth.TechReview.Internal.ReportIssueDetail,
	decision: DetailDecision,
	scope: DetailDecisionScope,
): boolean {
	const localDecision = detailDecisions.get(detail.id)
	const localScope = detailDecisionScopes.get(detail.id)
	if (localDecision && localScope) {
		if (localDecision === 'pending') {
			if (localScope === 'local') {
				if (scope === 'local') return false
				return statusMatchesDecision(detail.global_status, decision)
			}

			if (scope === 'global') return false
			return statusMatchesDecision(detail.local_status, decision)
		}

		return localDecision === decision && localScope === scope
	}

	if (scope === 'global') {
		return statusMatchesDecision(detail.global_status, decision)
	}

	if (detail.global_status) {
		return false
	}

	return statusMatchesDecision(detail.local_status, decision)
}

function decisionToVerdict(
	decision: Exclude<DetailDecision, 'pending'>,
): Labrinth.TechReview.Internal.DelphiReportIssueStatus {
	return decision === 'safe' ? 'safe' : 'unsafe'
}

function getToggledDetailVerdict(
	detail: Labrinth.TechReview.Internal.ReportIssueDetail,
	decision: Exclude<DetailDecision, 'pending'>,
	scope: DetailDecisionScope,
): Labrinth.TechReview.Internal.DelphiReportIssueStatus {
	return isDetailActionSelected(detail, decision, scope) ? 'pending' : decisionToVerdict(decision)
}

function getDetailActionTooltip(
	detail: Labrinth.TechReview.Internal.ReportIssueDetail,
	decision: Exclude<DetailDecision, 'pending'>,
	scope: DetailDecisionScope,
): string {
	const action = decision === 'safe' ? 'pass' : 'fail'
	const scopeLabel = scope === 'global' ? 'Global' : 'Local'

	if (scope === 'global' && !canUpdateGlobalDetail(detail)) {
		return 'Global verdict unavailable for generated trace keys'
	}

	if (isDetailActionSelected(detail, decision, scope)) {
		return `Unset ${scopeLabel.toLowerCase()} ${action}`
	}

	return `${scopeLabel} ${action}`
}

function updateLocalDetailAction(
	detail: Labrinth.TechReview.Internal.ReportIssueDetail,
	decision: Exclude<DetailDecision, 'pending'>,
) {
	return updateDetailStatus(detail.id, getToggledDetailVerdict(detail, decision, 'local'))
}

function updateGlobalDetailAction(
	detail: Labrinth.TechReview.Internal.ReportIssueDetail,
	decision: Exclude<DetailDecision, 'pending'>,
) {
	return updateGlobalDetailStatus(detail, getToggledDetailVerdict(detail, decision, 'global'))
}

function canUpdateGlobalDetail(detail: Labrinth.TechReview.Internal.ReportIssueDetail): boolean {
	return detail.key.length > 0 && !detail.key.startsWith('<no-key-')
}

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
			return aComplete === bComplete ? 0 : aComplete ? 1 : -1
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

function truncateMiddle(str: string, maxLength: number = 120): string {
	if (str.length <= maxLength) return str
	const separator = '...'
	const sepLen = separator.length
	const charsToShow = maxLength - sepLen
	const frontChars = Math.ceil(charsToShow / 3)
	const backChars = Math.floor((charsToShow * 2) / 3)
	return str.slice(0, frontChars) + separator + str.slice(-backChars)
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

function viewFileFlags(file: FlattenedFileReport) {
	selectedFileId.value = file.id
	currentTab.value = 'File'
}

function getDetailElementId(detailId: string) {
	return `tech-review-detail-${detailId}`
}

function findFileForDetail(detailId: string): FlattenedFileReport | null {
	for (const report of props.item.reports) {
		for (const issue of report.issues) {
			if (issue.details.some((detail) => detail.id === detailId)) {
				return report
			}
		}
	}

	return null
}

async function focusDetail(detailId: string) {
	const file = findFileForDetail(detailId)
	if (!file) return

	viewFileFlags(file)
	await nextTick()

	const classItem = groupedByClass.value.find((group) =>
		group.flags.some((flag) => flag.detail.id === detailId),
	)

	if (classItem) {
		expandClass(classItem)
	}

	await nextTick()

	if (!import.meta.client) return

	window.requestAnimationFrame(() => {
		document.getElementById(getDetailElementId(detailId))?.scrollIntoView({
			behavior: 'smooth',
			block: 'center',
		})
	})
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
		showCopyFeedback.set(detailId, true)
		setTimeout(() => {
			showCopyFeedback.delete(detailId)
		}, 2000)
	} catch (error) {
		console.error('Failed to copy code:', error)
	}
}

function getDetailDecision(
	detailId: string,
	backendStatus: Labrinth.TechReview.Internal.DelphiReportIssueStatus,
): 'safe' | 'malware' | 'pending' {
	const localDecision = detailDecisions.get(detailId)
	if (localDecision) return localDecision
	if (backendStatus === 'safe') return 'safe'
	if (backendStatus === 'unsafe') return 'malware'
	return 'pending'
}

function isPreReviewed(
	detailId: string,
	backendStatus: Labrinth.TechReview.Internal.DelphiReportIssueStatus,
): boolean {
	return (backendStatus === 'safe' || backendStatus === 'unsafe') && !detailDecisions.has(detailId)
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

const remainingUnmarkedCount = computed(() => {
	if (!selectedFile.value) return 0
	return getFileDetailCount(selectedFile.value) - getFileMarkedCount(selectedFile.value)
})

function getSelectedFileFlags(): ClassGroup['flags'] {
	if (!selectedFile.value) return []

	return selectedFile.value.issues.flatMap((issue) =>
		issue.details.map((detail) => ({
			issueId: issue.id,
			issueType: issue.issue_type,
			detail,
		})),
	)
}

function getJarFlags(jarGroup: JarGroup): ClassGroup['flags'] {
	return jarGroup.classes.flatMap((classItem) => classItem.flags)
}

function getJarMarkedCount(jarGroup: JarGroup): number {
	return getMarkedFlagsCount(getJarFlags(jarGroup))
}

function getJarRemainingUnmarkedCount(jarGroup: JarGroup): number {
	return getJarFlags(jarGroup).length - getJarMarkedCount(jarGroup)
}

const isBatchUpdating = ref(false)

function getRemainingGlobalDetailCount(flags: ClassGroup['flags']): number {
	return new Set(
		flags
			.filter(
				(flag) =>
					getDetailDecision(flag.detail.id, flag.detail.status) === 'pending' &&
					canUpdateGlobalDetail(flag.detail),
			)
			.map((flag) => flag.detail.key),
	).size
}

async function batchMarkRemainingGlobally(flags: ClassGroup['flags'], verdict: 'safe' | 'unsafe') {
	if (isBatchUpdating.value) return

	const detailsByKey = new Map(
		flags
			.filter(
				(flag) =>
					getDetailDecision(flag.detail.id, flag.detail.status) === 'pending' &&
					canUpdateGlobalDetail(flag.detail),
			)
			.map((flag) => [flag.detail.key, flag.detail]),
	)
	const details = [...detailsByKey.values()]

	if (details.length === 0) return

	isBatchUpdating.value = true
	try {
		await client.labrinth.tech_review_internal.updateGlobalIssueDetails(
			details.map((detail) => ({ detail_key: detail.key, verdict })),
		)

		applyDecisionToRelatedDetails(
			details.map((detail) => detail.id),
			verdictToDecision(verdict),
			'global',
		)

		addNotification({
			type: 'success',
			title: `Globally marked ${details.length} trace keys as ${verdict}`,
			text: `All remaining eligible traces have been globally marked as ${
				verdict === 'safe' ? 'false positives' : 'malicious'
			}.`,
		})

		if (
			selectedFile.value &&
			getFileMarkedCount(selectedFile.value) === getFileDetailCount(selectedFile.value)
		) {
			backToFileList()
		}

		emit('refetch')
	} catch (error) {
		console.error('Failed to batch update global traces:', error)
		addNotification({
			type: 'error',
			title: 'Global batch update failed',
			text: 'An error occurred while globally updating traces.',
		})
	} finally {
		isBatchUpdating.value = false
	}
}

async function batchMarkRemaining(verdict: 'safe' | 'unsafe') {
	if (!selectedFile.value || isBatchUpdating.value) return

	const detailIds: string[] = []
	for (const issue of selectedFile.value.issues) {
		for (const detail of issue.details) {
			const detailWithStatus = detail as typeof detail & {
				status: Labrinth.TechReview.Internal.DelphiReportIssueStatus
			}
			if (getDetailDecision(detailWithStatus.id, detailWithStatus.status) === 'pending') {
				detailIds.push(detail.id)
			}
		}
	}

	if (detailIds.length === 0) return

	isBatchUpdating.value = true
	try {
		await updateIssueDetails(detailIds.map((detailId) => ({ detail_id: detailId, verdict })))

		applyDecisionToRelatedDetails(detailIds, verdictToDecision(verdict), 'local')

		addNotification({
			type: 'success',
			title: `Marked ${detailIds.length} traces as ${verdict}`,
			text: `All remaining traces have been marked as ${verdict === 'safe' ? 'false positives' : 'malicious'}.`,
		})

		// Jump back to Files tab when all flags in the current file are marked
		if (selectedFile.value) {
			const markedCount = getFileMarkedCount(selectedFile.value)
			const totalCount = getFileDetailCount(selectedFile.value)
			if (markedCount === totalCount) {
				backToFileList()
			}
		}

		emit('refetch')
	} catch (error) {
		console.error('Failed to batch update:', error)
		addNotification({
			type: 'error',
			title: 'Batch update failed',
			text: 'An error occurred while updating traces.',
		})
	} finally {
		isBatchUpdating.value = false
	}
}

async function batchMarkRemainingInJar(jarGroup: JarGroup, verdict: 'safe' | 'unsafe') {
	if (isBatchUpdating.value) return

	const detailIds = getJarFlags(jarGroup)
		.filter((flag) => getDetailDecision(flag.detail.id, flag.detail.status) === 'pending')
		.map((flag) => flag.detail.id)

	if (detailIds.length === 0) return

	isBatchUpdating.value = true
	try {
		await updateIssueDetails(detailIds.map((detailId) => ({ detail_id: detailId, verdict })))

		applyDecisionToRelatedDetails(detailIds, verdictToDecision(verdict), 'local')

		addNotification({
			type: 'success',
			title: `Marked ${detailIds.length} traces as ${verdict}`,
			text: `All remaining traces in this JAR have been marked as ${
				verdict === 'safe' ? 'false positives' : 'malicious'
			}.`,
		})

		if (selectedFile.value) {
			const markedCount = getFileMarkedCount(selectedFile.value)
			const totalCount = getFileDetailCount(selectedFile.value)
			if (markedCount === totalCount) {
				backToFileList()
			}
		}

		emit('refetch')
	} catch (error) {
		console.error('Failed to batch update JAR traces:', error)
		addNotification({
			type: 'error',
			title: 'Batch update failed',
			text: 'An error occurred while updating JAR traces.',
		})
	} finally {
		isBatchUpdating.value = false
	}
}

async function updateDetailStatus(
	detailId: string,
	verdict: Labrinth.TechReview.Internal.DelphiReportIssueStatus,
) {
	let priorDecision: 'safe' | 'malware' | 'pending' = 'pending'
	outer: for (const report of props.item.reports) {
		for (const issue of report.issues) {
			const detail = issue.details.find((d) => d.id === detailId)
			if (detail) {
				priorDecision = getDetailDecision(detail.id, detail.status)
				break outer
			}
		}
	}

	updatingDetails.add(detailId)

	try {
		await updateIssueDetails([{ detail_id: detailId, verdict }])

		const { otherMatchedCount } = applyDecisionToRelatedDetails(
			[detailId],
			verdictToDecision(verdict),
			'local',
		)

		// Only collapse if the prior state was 'pending' (new decision, not updating existing)
		if (verdict !== 'pending' && priorDecision === 'pending') {
			for (const classGroup of groupedByClass.value) {
				const hasThisDetail = classGroup.flags.some((f) => f.detail.id === detailId)
				if (hasThisDetail && getMarkedFlagsCount(classGroup.flags) === classGroup.flags.length) {
					expandedClasses.delete(classGroup.key)
					break
				}
			}
		}

		// Jump back to Files tab when all flags in the current file are marked
		if (verdict !== 'pending' && selectedFile.value) {
			const markedCount = getFileMarkedCount(selectedFile.value)
			const totalCount = getFileDetailCount(selectedFile.value)
			if (markedCount === totalCount) {
				backToFileList()
			}
		}

		const otherText =
			otherMatchedCount > 0
				? ` (${otherMatchedCount} other trace${otherMatchedCount === 1 ? '' : 's'} also marked)`
				: ''

		if (verdict === 'pending') {
			addNotification({
				type: 'success',
				title: 'Local trace verdict unset',
				text: `The project-local verdict has been removed.${otherText}`,
			})
		} else if (verdict === 'safe') {
			addNotification({
				type: 'success',
				title: 'Issue marked as pass',
				text: `This issue has been marked as a false positive.${otherText}`,
			})
		} else {
			addNotification({
				type: 'success',
				title: 'Issue marked as fail',
				text: `This issue has been flagged as malicious.${otherText}`,
			})
		}

		emit('refetch')
	} catch (error) {
		console.error('Failed to update detail status:', error)
		addNotification({
			type: 'error',
			title: 'Failed to update issue',
			text: 'An error occurred while updating the issue status.',
		})
	} finally {
		updatingDetails.delete(detailId)
	}
}

async function updateGlobalDetailStatus(
	detail: Labrinth.TechReview.Internal.ReportIssueDetail,
	verdict: Labrinth.TechReview.Internal.DelphiReportIssueStatus,
) {
	if (!canUpdateGlobalDetail(detail)) {
		addNotification({
			type: 'error',
			title: 'Global update unavailable',
			text: 'Generated trace keys cannot be marked globally.',
		})
		return
	}

	updatingGlobalDetailKeys.add(detail.key)

	try {
		await updateGlobalIssueDetail(detail.key, verdict)

		const { otherMatchedCount } = applyDecisionToRelatedDetails(
			[detail.id],
			verdictToDecision(verdict),
			'global',
		)

		if (verdict !== 'pending') {
			for (const classGroup of groupedByClass.value) {
				if (getMarkedFlagsCount(classGroup.flags) === classGroup.flags.length) {
					expandedClasses.delete(classGroup.key)
				}
			}
		}

		if (verdict !== 'pending' && selectedFile.value) {
			const markedCount = getFileMarkedCount(selectedFile.value)
			const totalCount = getFileDetailCount(selectedFile.value)
			if (markedCount === totalCount) {
				backToFileList()
			}
		}

		const otherText =
			otherMatchedCount > 0
				? ` (${otherMatchedCount} other trace${otherMatchedCount === 1 ? '' : 's'} also marked in this project)`
				: ''

		if (verdict === 'pending') {
			addNotification({
				type: 'success',
				title: 'Global trace verdict unset',
				text: `The global verdict for this trace key has been removed.${otherText}`,
			})
		} else {
			addNotification({
				type: 'success',
				title:
					verdict === 'safe' ? 'Trace globally marked as pass' : 'Trace globally marked as fail',
				text:
					verdict === 'safe'
						? `This trace key has been marked as a global false positive.${otherText}`
						: `This trace key has been globally flagged as malicious.${otherText}`,
			})
		}

		emit('refetch')
	} catch (error) {
		console.error('Failed to update global detail status:', error)
		addNotification({
			type: 'error',
			title: 'Failed to update global trace',
			text: 'An error occurred while updating the global trace status.',
		})
	} finally {
		updatingGlobalDetailKeys.delete(detail.key)
	}
}

const expandedClasses = reactive<Set<string>>(new Set())
const autoExpandedFileIds = reactive<Set<string>>(new Set())
const showCopyFeedback = reactive<Map<string, boolean>>(new Map())
const highlightedSourceCache = reactive<Map<string, { source: string; lines: string[] }>>(new Map())
const LAZY_LOAD_CLASS_SOURCE_MINIMUM = 2

interface ClassGroup {
	key: string
	jar: string | null
	filePath: string
	flags: Array<{
		issueId: string
		issueType: string
		detail: Labrinth.TechReview.Internal.ReportIssueDetail & {
			status: Labrinth.TechReview.Internal.DelphiReportIssueStatus
		}
	}>
}

interface JarGroup {
	key: string
	jar: string | null
	segments: string[]
	classes: ClassGroup[]
}

function splitJarSegments(jar: string | null, currentFileName: string | null): string[] {
	if (!jar) return []
	const segments = jar
		.split(/[/#]/)
		.map((s) => decodeURIComponent(s.trim()))
		.filter((s) => s.length > 0)
	// Skip the first segment if it matches the current file tab (it's already shown in the file list)
	if (segments.length > 0 && currentFileName && segments[0] === currentFileName) {
		return segments.slice(1)
	}
	return segments
}

function isRootJarGroup(jarGroup: JarGroup): boolean {
	return jarGroup.segments.length === 0
}

const groupedByClass = computed<ClassGroup[]>(() => {
	if (!selectedFile.value) return []

	const classMap = new Map<string, ClassGroup>()

	for (const issue of selectedFile.value.issues) {
		for (const detail of issue.details) {
			if (hideGloballyPassed.value && isDetailGloballyPassed(detail)) {
				continue
			}

			const classKey = `${detail.jar ?? ''}::${detail.file_path}`
			if (!classMap.has(classKey)) {
				classMap.set(classKey, {
					key: classKey,
					jar: detail.jar ?? null,
					filePath: detail.file_path,
					flags: [],
				})
			}
			// Cast detail to include status (backend will provide this field)
			const detailWithStatus = detail as Labrinth.TechReview.Internal.ReportIssueDetail & {
				status: Labrinth.TechReview.Internal.DelphiReportIssueStatus
			}
			classMap.get(classKey)!.flags.push({
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
			return aPreReviewed === bPreReviewed ? 0 : aPreReviewed ? 1 : -1
		})
	}

	return Array.from(classMap.values())
})

const groupedByJar = computed<JarGroup[]>(() => {
	const jarMap = new Map<string, JarGroup>()

	for (const classItem of groupedByClass.value) {
		const jarKey = classItem.jar ?? ''
		if (!jarMap.has(jarKey)) {
			jarMap.set(jarKey, {
				key: jarKey,
				jar: classItem.jar,
				segments: splitJarSegments(classItem.jar, selectedFile.value?.file_name ?? null),
				classes: [],
			})
		}
		jarMap.get(jarKey)!.classes.push(classItem)
	}

	return Array.from(jarMap.values()).sort((a, b) => {
		const aRoot = isRootJarGroup(a)
		const bRoot = isRootJarGroup(b)
		return aRoot === bRoot ? 0 : aRoot ? -1 : 1
	})
})

watch(
	() => props.focusedDetailId,
	(detailId) => {
		if (detailId) {
			focusDetail(detailId)
		}
	},
	{ immediate: true },
)

// Auto-expand/load source for small files; keep larger files lazy.
watch(
	[selectedFileId, groupedByClass],
	([fileId, classes]) => {
		if (!fileId || classes.length === 0 || autoExpandedFileIds.has(fileId)) return

		autoExpandedFileIds.add(fileId)

		if (classes.length < LAZY_LOAD_CLASS_SOURCE_MINIMUM) {
			for (const classItem of classes) {
				expandClass(classItem)
			}
		}
	},
	{ immediate: true },
)

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

function getClassDecompiledSource(classItem: ClassGroup): string | undefined {
	for (const flag of classItem.flags) {
		const source = props.decompiledSources.get(flag.detail.id)
		if (source) return source
	}
	return undefined
}

function getHighlightedClassSource(classItem: ClassGroup): string[] {
	const source = getClassDecompiledSource(classItem)
	if (!source) return []

	const cached = highlightedSourceCache.get(classItem.key)
	if (cached?.source === source) return cached.lines

	const lines = highlightCodeLines(source, 'java')
	highlightedSourceCache.set(classItem.key, { source, lines })
	return lines
}

function isClassLoadingSource(classItem: ClassGroup): boolean {
	return classItem.flags.some((flag) => props.loadingIssues.has(flag.issueId))
}

function loadClassSources(classItem: ClassGroup) {
	const issueIds = [...new Set(classItem.flags.map((flag) => flag.issueId))]
	if (issueIds.length > 0) {
		emit('loadIssueSources', issueIds)
	}
}

function expandClass(classItem: ClassGroup) {
	if (expandedClasses.has(classItem.key)) return
	expandedClasses.add(classItem.key)
	loadClassSources(classItem)
}

function toggleClass(classItem: ClassGroup) {
	if (expandedClasses.has(classItem.key)) {
		expandedClasses.delete(classItem.key)
	} else {
		expandClass(classItem)
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

	const timestamp = formatDateTimeUtc(dayjs().toDate())
	let markdown = `## Tech Review Summary\n*${timestamp}*\n\n`
	markdown += `<details>\n<summary>File Details (${totalSafe} safe, ${totalUnsafe} unsafe)</summary>\n\n`

	for (const [, fileData] of fileDecisions) {
		if (fileData.decisions.length === 0) continue

		const fileSafe = fileData.decisions.filter((d) => d.decision === 'safe').length
		const fileUnsafe = fileData.decisions.filter((d) => d.decision === 'malware').length
		const fileVerdict = fileUnsafe > 0 ? 'Unsafe' : 'Safe'

		markdown += `### ${fileData.fileName}\n`
		markdown += `> ${formatBytes(fileData.fileSize)} • ${fileData.decisions.length} issues • Max severity: ${fileData.maxSeverity} • **Verdict:** ${fileVerdict}\n\n`
		markdown += `<details>\n<summary>Issues (${fileSafe} safe, ${fileUnsafe} unsafe)</summary>\n\n`
		markdown += `| Class | Issue Type | Severity | Decision |\n`
		markdown += `|-------|------------|----------|----------|\n`

		for (const d of fileData.decisions) {
			const decisionText = d.decision === 'safe' ? '✅ Safe' : '❌ Unsafe'
			markdown += `| \`${d.filePath}\` | ${d.issueType} | ${capitalizeString(d.severity)} | ${decisionText} |\n`
		}

		markdown += `\n</details>\n\n`
	}

	markdown += `</details>\n\n`
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
			private: true,
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

function copyId() {
	navigator.clipboard.writeText(props.item.project.id).then(() => {
		addNotification({
			type: 'success',
			title: 'Project ID copied',
			text: 'The ID of this project has been copied to your clipboard.',
		})
	})
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
								class="flex items-center gap-1 rounded-full border border-solid px-2.5 py-1"
								:class="
									isProjectApproved
										? 'border-green bg-highlight-green'
										: 'border-orange bg-highlight-orange'
								"
							>
								<CheckIcon v-if="isProjectApproved" aria-hidden="true" class="h-4 w-4 text-green" />
								<TimerIcon v-else aria-hidden="true" class="h-4 w-4 text-orange" />
								<span
									class="text-sm font-medium"
									:class="isProjectApproved ? 'text-green' : 'text-orange'"
								>
									{{ isProjectApproved ? 'Live' : 'In review' }}
								</span>
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
					<div class="flex items-center gap-2">
						<ButtonStyled v-if="props.item.project.link_urls?.['source']?.url" circular>
							<a
								v-tooltip="'Open sources in new tab'"
								:href="props.item.project.link_urls?.['source']?.url"
								target="_blank"
							>
								<CodeIcon />
							</a>
						</ButtonStyled>
						<ButtonStyled circular>
							<button v-tooltip="'Copy ID'" @click="copyId">
								<ClipboardCopyIcon />
							</button>
						</ButtonStyled>
						<ButtonStyled circular>
							<a
								v-tooltip="'Open in new tab'"
								:href="`/moderation/technical-review/${props.item.project.id}`"
								target="_blank"
							>
								<ExternalIcon />
							</a>
						</ButtonStyled>
					</div>
				</div>
			</div>

			<div class="h-px w-full bg-surface-5"></div>

			<NavTabs
				mode="local"
				:links="navTabsLinks"
				:active-index="activeTabIndex"
				class="bg-surface-3! shadow-none!"
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
					<div class="bg-surface-2 pt-0">
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
								<ButtonStyled color="standard">
									<OverflowMenu
										class="btn-dropdown-animation"
										:disabled="isLoadingStatusAction"
										:options="projectStatusActions"
									>
										<SpinnerIcon
											v-if="isLoadingStatusAction"
											class="animate-spin"
											aria-hidden="true"
										/>
										<ScaleIcon v-else aria-hidden="true" />
										Set Status
										<template #approve>
											<CheckIcon aria-hidden="true" />
											Approve
										</template>
										<template #withhold>
											<EyeOffIcon aria-hidden="true" />
											Withhold
										</template>
										<template #send-to-review>
											<ScaleIcon aria-hidden="true" />
											Send to review
										</template>
										<template #reject>
											<XIcon aria-hidden="true" />
											Reject
										</template>
									</OverflowMenu>
								</ButtonStyled>
								<ButtonStyled v-if="featureFlags.developerMode" type="outlined">
									<button @click="emit('showMaliciousSummary', unsafeFiles)">Debug</button>
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
							v-tooltip="file.file_name"
							class="font-medium text-contrast"
							:class="{ 'cursor-pointer hover:underline': getFileDetailCount(file) > 0 }"
							@click="getFileDetailCount(file) > 0 && viewFileFlags(file)"
						>
							{{ truncateMiddle(file.file_name, 50) }}
						</span>
						<div class="rounded-full border border-solid border-surface-5 bg-surface-3 px-2.5 py-1">
							<span class="text-sm font-medium text-secondary">{{
								formatBytes(file.file_size)
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
					v-if="getFileDetailCount(selectedFile) > 0"
					class="flex flex-wrap items-center justify-between gap-3 border-x border-b border-t-0 border-solid border-surface-3 bg-surface-2 p-4"
				>
					<div
						v-if="remainingUnmarkedCount > 0"
						class="detail-verdict-buttons"
						role="group"
						aria-label="Remaining issue actions"
					>
						<span class="remaining-verdict-label"
							>Remaining issues ({{ remainingUnmarkedCount }})</span
						>
						<button
							v-tooltip="'Remaining globally safe'"
							class="detail-verdict-button detail-verdict-button--safe"
							aria-label="Remaining globally safe"
							:disabled="
								isBatchUpdating || getRemainingGlobalDetailCount(getSelectedFileFlags()) === 0
							"
							@click="batchMarkRemainingGlobally(getSelectedFileFlags(), 'safe')"
						>
							<CheckCheckIcon aria-hidden="true" />
						</button>
						<button
							v-tooltip="'Remaining safe'"
							class="detail-verdict-button detail-verdict-button--safe"
							aria-label="Remaining safe"
							:disabled="isBatchUpdating"
							@click="batchMarkRemaining('safe')"
						>
							<CheckIcon aria-hidden="true" />
						</button>
						<button
							v-tooltip="'Remaining malware'"
							class="detail-verdict-button detail-verdict-button--unsafe"
							aria-label="Remaining malware"
							:disabled="isBatchUpdating"
							@click="batchMarkRemaining('unsafe')"
						>
							<BanIcon aria-hidden="true" />
						</button>
						<button
							v-tooltip="'Remaining globally unsafe'"
							class="detail-verdict-button detail-verdict-button--unsafe"
							aria-label="Remaining globally unsafe"
							:disabled="
								isBatchUpdating || getRemainingGlobalDetailCount(getSelectedFileFlags()) === 0
							"
							@click="batchMarkRemainingGlobally(getSelectedFileFlags(), 'unsafe')"
						>
							<ShieldAlertIcon aria-hidden="true" />
						</button>
					</div>
					<label class="ml-auto flex cursor-pointer items-center gap-3 text-sm">
						<span class="text-right text-secondary">
							Hide globally passed
							<span class="text-tertiary block text-xs">
								{{ globallyResolvedSelectedFileCount }}/{{ getFileDetailCount(selectedFile) }}
								traces globally resolved
							</span>
						</span>
						<Toggle
							v-model="hideGloballyPassed"
							:disabled="globallyPassedSelectedFileCount === 0"
							small
						/>
					</label>
				</div>
				<div
					v-for="jarGroup in groupedByJar"
					:key="jarGroup.key"
					class="border-x border-b-0 border-t-0 border-solid border-surface-3 bg-surface-2"
				>
					<div
						v-if="jarGroup.segments.length > 0"
						class="border-b border-solid border-surface-1 px-4 py-3"
					>
						<div class="flex flex-wrap items-center justify-between gap-3">
							<div class="flex flex-wrap items-center gap-1">
								<template
									v-for="(segment, index) in jarGroup.segments"
									:key="`${jarGroup.key}-${index}`"
								>
									<span
										class="font-mono text-sm"
										:class="
											index === jarGroup.segments.length - 1
												? 'font-semibold text-contrast'
												: 'text-secondary'
										"
									>
										{{ segment }}
									</span>
									<ChevronRightIcon
										v-if="index < jarGroup.segments.length - 1"
										class="size-4 text-secondary"
									/>
								</template>
							</div>

							<div
								v-if="getJarRemainingUnmarkedCount(jarGroup) > 0"
								class="detail-verdict-buttons"
								role="group"
								aria-label="Remaining JAR issue actions"
							>
								<span class="remaining-verdict-label">
									Remaining issues ({{ getJarRemainingUnmarkedCount(jarGroup) }})
								</span>
								<button
									v-tooltip="'Remaining globally safe'"
									class="detail-verdict-button detail-verdict-button--safe"
									aria-label="Remaining globally safe"
									:disabled="
										isBatchUpdating || getRemainingGlobalDetailCount(getJarFlags(jarGroup)) === 0
									"
									@click="batchMarkRemainingGlobally(getJarFlags(jarGroup), 'safe')"
								>
									<CheckCheckIcon aria-hidden="true" />
								</button>
								<button
									v-tooltip="'Remaining safe'"
									class="detail-verdict-button detail-verdict-button--safe"
									aria-label="Remaining safe"
									:disabled="isBatchUpdating"
									@click="batchMarkRemainingInJar(jarGroup, 'safe')"
								>
									<CheckIcon aria-hidden="true" />
								</button>
								<button
									v-tooltip="'Remaining malware'"
									class="detail-verdict-button detail-verdict-button--unsafe"
									aria-label="Remaining malware"
									:disabled="isBatchUpdating"
									@click="batchMarkRemainingInJar(jarGroup, 'unsafe')"
								>
									<BanIcon aria-hidden="true" />
								</button>
								<button
									v-tooltip="'Remaining globally unsafe'"
									class="detail-verdict-button detail-verdict-button--unsafe"
									aria-label="Remaining globally unsafe"
									:disabled="
										isBatchUpdating || getRemainingGlobalDetailCount(getJarFlags(jarGroup)) === 0
									"
									@click="batchMarkRemainingGlobally(getJarFlags(jarGroup), 'unsafe')"
								>
									<ShieldAlertIcon aria-hidden="true" />
								</button>
							</div>
						</div>
					</div>

					<div
						v-for="classItem in jarGroup.classes"
						:key="classItem.key"
						class="border-b border-solid border-surface-1 last:border-b-0"
					>
						<div
							class="flex cursor-pointer items-center justify-between p-4 transition-colors duration-200 hover:bg-surface-4"
							@click="toggleClass(classItem)"
						>
							<div class="my-auto flex items-center gap-2">
								<ButtonStyled type="transparent" circular>
									<button
										class="transition-transform"
										:class="{ 'rotate-180': expandedClasses.has(classItem.key) }"
									>
										<ChevronDownIcon class="h-5 w-5 text-contrast" />
									</button>
								</ButtonStyled>

								<span v-tooltip="classItem.filePath" class="font-mono font-semibold">{{
									truncateMiddle(classItem.filePath)
								}}</span>

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
										v-if="isClassLoadingSource(classItem)"
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

						<Collapsible :collapsed="!expandedClasses.has(classItem.key)">
							<div
								v-if="expandedClasses.has(classItem.key)"
								class="mt-2 flex flex-col gap-2 px-4 pb-4"
							>
								<div
									v-for="flag in classItem.flags"
									:id="getDetailElementId(flag.detail.id)"
									:key="`${flag.issueId}-${flag.detail.id}`"
									class="flex flex-col gap-2 rounded-lg border-[1px] border-b border-solid border-surface-5 bg-surface-3 py-2 pl-4 last:border-b-0"
									:class="{
										'!border-brand bg-brand-highlight': props.focusedDetailId === flag.detail.id,
									}"
								>
									<div class="grid grid-cols-[1fr_auto] items-center">
										<div
											class="flex items-center gap-2"
											:class="{
												'opacity-50': isPreReviewed(flag.detail.id, flag.detail.status),
											}"
										>
											<span class="text-base font-semibold text-contrast">{{
												flag.issueType.replace(/_/g, ' ')
											}}</span>
											<div
												class="rounded-full border-solid px-2.5 py-1"
												:class="getSeverityBadgeColor(flag.detail.severity)"
											>
												<span class="text-sm font-medium">{{
													capitalizeString(flag.detail.severity)
												}}</span>
											</div>
										</div>

										<div class="detail-verdict-action-groups">
											<div
												class="detail-verdict-buttons"
												role="group"
												aria-label="Trace verdict actions"
											>
												<button
													v-tooltip="getDetailActionTooltip(flag.detail, 'safe', 'global')"
													class="detail-verdict-button detail-verdict-button--safe"
													:class="{
														'detail-verdict-button--selected': isDetailActionSelected(
															flag.detail,
															'safe',
															'global',
														),
													}"
													aria-label="Global pass"
													:disabled="
														!canUpdateGlobalDetail(flag.detail) ||
														updatingGlobalDetailKeys.has(flag.detail.key) ||
														updatingDetails.has(flag.detail.id)
													"
													@click="updateGlobalDetailAction(flag.detail, 'safe')"
												>
													<CheckCheckIcon aria-hidden="true" />
												</button>

												<button
													v-tooltip="getDetailActionTooltip(flag.detail, 'safe', 'local')"
													class="detail-verdict-button detail-verdict-button--safe"
													:class="{
														'detail-verdict-button--selected': isDetailActionSelected(
															flag.detail,
															'safe',
															'local',
														),
													}"
													aria-label="Local pass"
													:disabled="
														updatingDetails.has(flag.detail.id) ||
														updatingGlobalDetailKeys.has(flag.detail.key)
													"
													@click="updateLocalDetailAction(flag.detail, 'safe')"
												>
													<CheckIcon aria-hidden="true" />
												</button>

												<button
													v-tooltip="getDetailActionTooltip(flag.detail, 'malware', 'local')"
													class="detail-verdict-button detail-verdict-button--unsafe"
													:class="{
														'detail-verdict-button--selected': isDetailActionSelected(
															flag.detail,
															'malware',
															'local',
														),
													}"
													aria-label="Local fail"
													:disabled="
														updatingDetails.has(flag.detail.id) ||
														updatingGlobalDetailKeys.has(flag.detail.key)
													"
													@click="updateLocalDetailAction(flag.detail, 'malware')"
												>
													<BanIcon aria-hidden="true" />
												</button>

												<button
													v-tooltip="getDetailActionTooltip(flag.detail, 'malware', 'global')"
													class="detail-verdict-button detail-verdict-button--unsafe"
													:class="{
														'detail-verdict-button--selected': isDetailActionSelected(
															flag.detail,
															'malware',
															'global',
														),
													}"
													aria-label="Global fail"
													:disabled="
														!canUpdateGlobalDetail(flag.detail) ||
														updatingGlobalDetailKeys.has(flag.detail.key) ||
														updatingDetails.has(flag.detail.id)
													"
													@click="updateGlobalDetailAction(flag.detail, 'malware')"
												>
													<ShieldAlertIcon aria-hidden="true" />
												</button>
											</div>
										</div>
									</div>
									<div
										v-if="flag.detail.data && Object.keys(flag.detail.data).length > 0"
										class="flex flex-wrap gap-x-4 gap-y-1 pr-4 text-sm"
									>
										<div
											v-for="[key, value] in Object.entries(flag.detail.data).sort(([a], [b]) =>
												a.localeCompare(b),
											)"
											:key="key"
											class="flex items-center gap-1.5"
										>
											<span class="text-secondary">{{ key }}:</span>
											<a
												v-if="typeof value === 'string' && value.startsWith('http')"
												:href="value"
												target="_blank"
												rel="noopener noreferrer"
												class="text-brand-blue hover:underline"
											>
												{{ value }}
											</a>
											<span v-else class="font-mono text-contrast">{{ value }}</span>
										</div>
									</div>
								</div>

								<div
									v-if="getHighlightedClassSource(classItem).length > 0"
									class="relative inset-0 overflow-hidden rounded-lg border border-solid border-surface-5 bg-surface-4"
								>
									<ButtonStyled circular type="transparent">
										<button
											v-tooltip="`Copy code`"
											class="absolute right-2 top-2 border-[1px]"
											@click="copyToClipboard(getClassDecompiledSource(classItem)!, classItem.key)"
										>
											<CopyIcon v-if="!showCopyFeedback.get(classItem.key)" />
											<CheckIcon v-else />
										</button>
									</ButtonStyled>

									<div class="overflow-x-auto bg-surface-3 py-3">
										<div
											v-for="(line, n) in getHighlightedClassSource(classItem)"
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
									v-else-if="isClassLoadingSource(classItem)"
									class="rounded-lg border border-solid border-surface-5 bg-surface-3 p-4"
								>
									<p class="flex items-center gap-2 text-sm text-secondary">
										<LoaderCircleIcon class="size-4 animate-spin" />
										Loading source...
									</p>
								</div>
								<div
									v-else
									class="rounded-lg border border-solid border-surface-5 bg-surface-3 p-4"
								>
									<p class="text-sm text-secondary">
										Source code not available or failed to decompile for this file.
									</p>
								</div>
							</div>
						</Collapsible>
					</div>
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

.detail-verdict-action-groups {
	display: flex;
	align-items: center;
	justify-content: flex-end;
	gap: 0.5rem;
	margin-inline-end: 0.5rem;
}

.detail-verdict-buttons {
	display: flex;
	align-items: center;
	overflow: hidden;
	border: 1px solid var(--surface-5);
	border-radius: var(--radius-md);
	background: var(--surface-3);
}

.remaining-verdict-label {
	padding-inline: 0.75rem;
	font-size: 0.875rem;
	font-weight: 600;
	white-space: nowrap;
	color: var(--color-secondary);
}

.detail-verdict-button {
	display: flex;
	width: 2rem;
	height: 2rem;
	align-items: center;
	justify-content: center;
	border: 0;
	border-left: 1px solid var(--surface-5);
	background: transparent;
	padding: 0;
	cursor: pointer;
	transition:
		background-color 0.15s ease-in-out,
		filter 0.15s ease-in-out;
}

.detail-verdict-button:first-child {
	border-left: 0;
	border-start-start-radius: calc(var(--radius-md) - 1px);
	border-end-start-radius: calc(var(--radius-md) - 1px);
}

.detail-verdict-button:last-child {
	border-start-end-radius: calc(var(--radius-md) - 1px);
	border-end-end-radius: calc(var(--radius-md) - 1px);
}

.detail-verdict-button:hover,
.detail-verdict-button:focus-visible {
	background: var(--surface-4);
}

.detail-verdict-button--selected {
	background: var(--color-green-bg);
	box-shadow: inset 0 0 0 1px var(--color-green);
}

.detail-verdict-button--selected:hover,
.detail-verdict-button--selected:focus-visible {
	background: var(--color-green-bg);
}

.detail-verdict-button:focus-visible {
	outline: none;
	box-shadow: inset 0 0 0 2px var(--color-brand);
}

.detail-verdict-button--selected:focus-visible {
	box-shadow: inset 0 0 0 2px var(--color-green);
}

.detail-verdict-button:disabled {
	cursor: not-allowed;
	opacity: 0.5;
}

.detail-verdict-button svg {
	width: 1rem;
	height: 1rem;
}

.detail-verdict-button--safe {
	color: var(--color-green);
}

.detail-verdict-button--unsafe {
	color: var(--color-red);
}
</style>
