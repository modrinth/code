<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ChevronRightIcon } from '@modrinth/assets'
import { injectNotificationManager, Toggle } from '@modrinth/ui'
import { computed, nextTick, reactive, ref, watch } from 'vue'

import { canUpdateGlobalDetail, getFileDetailCount, verdictToDecision } from './helpers'
import TechRevClassItem from './TechRevClassItem.vue'
import TechRevVerdictButtons from './TechRevVerdictButtons.vue'
import type {
	ClassGroup,
	FlagItem,
	FlattenedFileReport,
	JarGroup,
	TraceVerdictEvent,
} from './types'
import { injectTechReviewDecisions } from './use-tech-review-decisions'

const props = defineProps<{
	file: FlattenedFileReport
	focusedDetailId?: string | null
	loadingIssues: Set<string>
	decompiledSources: Map<string, string>
}>()

const emit = defineEmits<{
	refetch: []
	loadIssueSources: [issueIds: string[]]
	allFlagsResolved: []
}>()

const { addNotification } = injectNotificationManager()
const {
	updatingDetails,
	updatingGlobalDetailKeys,
	getDetailDecision,
	isPreReviewed,
	getFileMarkedCount,
	getMarkedFlagsCount,
	isDetailGloballyPassed,
	isDetailGloballyResolved,
	applyDecisionToRelatedDetails,
	getToggledDetailVerdict,
	updateIssueDetails,
	updateGlobalIssueDetails,
} = injectTechReviewDecisions()

const hideGloballyPassed = ref(true)
const isBatchUpdating = ref(false)
const expandedClasses = reactive<Set<string>>(new Set())
const autoExpandedFileIds = reactive<Set<string>>(new Set())
const LAZY_LOAD_CLASS_SOURCE_MINIMUM = 2

const globallyPassedCount = computed(() => {
	return props.file.issues.reduce(
		(count, issue) => count + issue.details.filter(isDetailGloballyPassed).length,
		0,
	)
})

const globallyResolvedCount = computed(() => {
	return props.file.issues.reduce(
		(count, issue) => count + issue.details.filter(isDetailGloballyResolved).length,
		0,
	)
})

const remainingUnmarkedCount = computed(() => {
	return getFileDetailCount(props.file) - getFileMarkedCount(props.file)
})

const selectedFileFlags = computed<FlagItem[]>(() =>
	props.file.issues.flatMap((issue) =>
		issue.details.map((detail) => ({
			issueId: issue.id,
			issueType: issue.issue_type,
			detail,
		})),
	),
)

function getJarFlags(jarGroup: JarGroup): FlagItem[] {
	return jarGroup.classes.flatMap((classItem) => classItem.flags)
}

function getJarRemainingUnmarkedCount(jarGroup: JarGroup): number {
	const flags = getJarFlags(jarGroup)
	return flags.length - getMarkedFlagsCount(flags)
}

function getRemainingGlobalDetailCount(flags: FlagItem[]): number {
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

function notifySuccess(title: string, text: string) {
	addNotification({ type: 'success', title, text, autoCloseMs: 3000 })
}

function notifyError(title: string, text: string) {
	addNotification({ type: 'error', title, text })
}

function maybeReturnToFileList() {
	if (getFileMarkedCount(props.file) === getFileDetailCount(props.file)) {
		emit('allFlagsResolved')
	}
}

function collapseCompletedClasses() {
	for (const classGroup of groupedByClass.value) {
		if (getMarkedFlagsCount(classGroup.flags) === classGroup.flags.length) {
			expandedClasses.delete(classGroup.key)
		}
	}
}

function afterSuccessfulMark(previousMarkedCount: number) {
	collapseCompletedClasses()
	if (previousMarkedCount !== getFileMarkedCount(props.file)) {
		maybeReturnToFileList()
	}
}

async function batchMarkRemainingGlobally(flags: FlagItem[], verdict: 'safe' | 'unsafe') {
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
		await updateGlobalIssueDetails(details.map((detail) => ({ detail_key: detail.key, verdict })))

		applyDecisionToRelatedDetails(
			details.map((detail) => detail.id),
			verdictToDecision(verdict),
			'global',
		)

		notifySuccess(
			`Globally marked ${details.length} trace keys as ${verdict}`,
			`All remaining eligible traces have been globally marked as ${
				verdict === 'safe' ? 'false positives' : 'malicious'
			}.`,
		)

		maybeReturnToFileList()
		emit('refetch')
	} catch (error) {
		console.error('Failed to batch update global traces:', error)
		notifyError('Global batch update failed', 'An error occurred while globally updating traces.')
	} finally {
		isBatchUpdating.value = false
	}
}

async function batchMarkRemaining(flags: FlagItem[], verdict: 'safe' | 'unsafe', inJar = false) {
	if (isBatchUpdating.value) return

	const detailIds = flags
		.filter((flag) => getDetailDecision(flag.detail.id, flag.detail.status) === 'pending')
		.map((flag) => flag.detail.id)

	if (detailIds.length === 0) return

	isBatchUpdating.value = true
	try {
		await updateIssueDetails(detailIds.map((detail_id) => ({ detail_id, verdict })))
		applyDecisionToRelatedDetails(detailIds, verdictToDecision(verdict), 'local')

		notifySuccess(
			`Marked ${detailIds.length} traces as ${verdict}`,
			`All remaining traces${inJar ? ' in this JAR' : ''} have been marked as ${
				verdict === 'safe' ? 'false positives' : 'malicious'
			}.`,
		)

		maybeReturnToFileList()
		emit('refetch')
	} catch (error) {
		console.error('Failed to batch update:', error)
		notifyError('Batch update failed', 'An error occurred while updating traces.')
	} finally {
		isBatchUpdating.value = false
	}
}

function updateLocalDetailAction(
	detail: Labrinth.TechReview.Internal.ReportIssueDetail,
	decision: 'safe' | 'malware',
) {
	return updateDetailStatus(detail.id, getToggledDetailVerdict(detail, decision, 'local'))
}

function updateGlobalDetailAction(
	detail: Labrinth.TechReview.Internal.ReportIssueDetail,
	decision: 'safe' | 'malware',
) {
	return updateGlobalDetailStatus(detail, getToggledDetailVerdict(detail, decision, 'global'))
}

function handleTraceVerdict({ detail, decision, scope }: TraceVerdictEvent) {
	if (scope === 'global') {
		return updateGlobalDetailAction(detail, decision)
	}
	return updateLocalDetailAction(detail, decision)
}

async function updateDetailStatus(
	detailId: string,
	verdict: Labrinth.TechReview.Internal.DelphiReportIssueStatus,
) {
	updatingDetails.add(detailId)

	const previousMarkedCount = getFileMarkedCount(props.file)

	try {
		await updateIssueDetails([{ detail_id: detailId, verdict }])

		const { otherMatchedCount } = applyDecisionToRelatedDetails(
			[detailId],
			verdictToDecision(verdict),
			'local',
		)

		if (verdict !== 'pending') {
			afterSuccessfulMark(previousMarkedCount)
		}

		const otherText =
			otherMatchedCount > 0
				? ` (${otherMatchedCount} other trace${otherMatchedCount === 1 ? '' : 's'} also marked)`
				: ''

		if (verdict === 'pending') {
			notifySuccess(
				'Local trace verdict unset',
				`The project-local verdict has been removed.${otherText}`,
			)
		} else if (verdict === 'safe') {
			notifySuccess(
				'Issue marked as pass',
				`This issue has been marked as a false positive.${otherText}`,
			)
		} else {
			notifySuccess('Issue marked as fail', `This issue has been flagged as malicious.${otherText}`)
		}

		emit('refetch')
	} catch (error) {
		console.error('Failed to update detail status:', error)
		notifyError('Failed to update issue', 'An error occurred while updating the issue status.')
	} finally {
		updatingDetails.delete(detailId)
	}
}

async function updateGlobalDetailStatus(
	detail: Labrinth.TechReview.Internal.ReportIssueDetail,
	verdict: Labrinth.TechReview.Internal.DelphiReportIssueStatus,
) {
	if (!canUpdateGlobalDetail(detail)) {
		notifyError('Global update unavailable', 'Generated trace keys cannot be marked globally.')
		return
	}

	updatingGlobalDetailKeys.add(detail.key)

	const previousMarkedCount = getFileMarkedCount(props.file)

	try {
		await updateGlobalIssueDetails([{ detail_key: detail.key, verdict }])

		const { otherMatchedCount } = applyDecisionToRelatedDetails(
			[detail.id],
			verdictToDecision(verdict),
			'global',
		)

		if (verdict !== 'pending') {
			afterSuccessfulMark(previousMarkedCount)
		}

		const otherText =
			otherMatchedCount > 0
				? ` (${otherMatchedCount} other trace${otherMatchedCount === 1 ? '' : 's'} also marked in this project)`
				: ''

		if (verdict === 'pending') {
			notifySuccess(
				'Global trace verdict unset',
				`The global verdict for this trace key has been removed.${otherText}`,
			)
		} else {
			notifySuccess(
				verdict === 'safe' ? 'Trace globally marked as pass' : 'Trace globally marked as fail',
				verdict === 'safe'
					? `This trace key has been marked as a global false positive.${otherText}`
					: `This trace key has been globally flagged as malicious.${otherText}`,
			)
		}

		emit('refetch')
	} catch (error) {
		console.error('Failed to update global detail status:', error)
		notifyError(
			'Failed to update global trace',
			'An error occurred while updating the global trace status.',
		)
	} finally {
		updatingGlobalDetailKeys.delete(detail.key)
	}
}

function splitJarSegments(jar: string | null, currentFileName: string | null): string[] {
	if (!jar) return []
	const segments = jar
		.split(/[/#]/)
		.map((s) => decodeURIComponent(s.trim()))
		.filter((s) => s.length > 0)
	if (segments.length > 0 && currentFileName && segments[0] === currentFileName) {
		return segments.slice(1)
	}
	return segments
}

const groupedByClass = computed<ClassGroup[]>(() => {
	const classMap = new Map<string, ClassGroup>()

	for (const issue of props.file.issues) {
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
			classMap.get(classKey)!.flags.push({
				issueId: issue.id,
				issueType: issue.issue_type,
				detail,
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
				segments: splitJarSegments(classItem.jar, props.file.file_name),
				classes: [],
			})
		}
		jarMap.get(jarKey)!.classes.push(classItem)
	}

	return Array.from(jarMap.values()).sort((a, b) => {
		const aRoot = a.segments.length === 0
		const bRoot = b.segments.length === 0
		return aRoot === bRoot ? 0 : aRoot ? -1 : 1
	})
})

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

async function focusDetail(detailId: string) {
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
		document.getElementById(`tech-review-detail-${detailId}`)?.scrollIntoView({
			behavior: 'smooth',
			block: 'center',
		})
	})
}

watch(
	[() => props.focusedDetailId, () => props.file.id],
	([detailId]) => {
		if (detailId) {
			focusDetail(detailId)
		}
	},
	{ immediate: true },
)

watch(
	[() => props.file.id, groupedByClass],
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
</script>

<template>
	<div
		v-if="getFileDetailCount(file) > 0"
		class="flex flex-wrap items-center justify-between gap-3 p-4"
	>
		<TechRevVerdictButtons
			v-if="remainingUnmarkedCount > 0"
			variant="remaining"
			:remaining-count="remainingUnmarkedCount"
			:global-disabled="isBatchUpdating || getRemainingGlobalDetailCount(selectedFileFlags) === 0"
			:local-disabled="isBatchUpdating"
			@global-safe="batchMarkRemainingGlobally(selectedFileFlags, 'safe')"
			@local-safe="batchMarkRemaining(selectedFileFlags, 'safe')"
			@local-unsafe="batchMarkRemaining(selectedFileFlags, 'unsafe')"
			@global-unsafe="batchMarkRemainingGlobally(selectedFileFlags, 'unsafe')"
		/>
		<label class="ml-auto flex cursor-pointer items-center gap-3 text-sm">
			<span class="text-right text-secondary">
				Hide globally passed
				<span class="text-tertiary block text-xs">
					{{ globallyResolvedCount }}/{{ getFileDetailCount(file) }} traces globally resolved
				</span>
			</span>
			<Toggle v-model="hideGloballyPassed" :disabled="globallyPassedCount === 0" small />
		</label>
	</div>
	<div v-for="jarGroup in groupedByJar" :key="jarGroup.key" class="flex flex-col gap-1 px-4 pb-4">
		<div v-if="jarGroup.segments.length > 0" class="my-2">
			<div class="flex flex-wrap items-center justify-between gap-3">
				<div class="flex flex-wrap items-center gap-1">
					<template v-for="(segment, index) in jarGroup.segments" :key="`${jarGroup.key}-${index}`">
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

				<TechRevVerdictButtons
					v-if="getJarRemainingUnmarkedCount(jarGroup) > 0"
					variant="remaining"
					jar
					:remaining-count="getJarRemainingUnmarkedCount(jarGroup)"
					:global-disabled="
						isBatchUpdating || getRemainingGlobalDetailCount(getJarFlags(jarGroup)) === 0
					"
					:local-disabled="isBatchUpdating"
					@global-safe="batchMarkRemainingGlobally(getJarFlags(jarGroup), 'safe')"
					@local-safe="batchMarkRemaining(getJarFlags(jarGroup), 'safe', true)"
					@local-unsafe="batchMarkRemaining(getJarFlags(jarGroup), 'unsafe', true)"
					@global-unsafe="batchMarkRemainingGlobally(getJarFlags(jarGroup), 'unsafe')"
				/>
			</div>
		</div>

		<TechRevClassItem
			v-for="classItem in jarGroup.classes"
			:key="classItem.key"
			:file="file"
			:class-item="classItem"
			:expanded="expandedClasses.has(classItem.key)"
			:focused-detail-id="focusedDetailId"
			:loading-issues="loadingIssues"
			:decompiled-sources="decompiledSources"
			@toggle="toggleClass(classItem)"
			@verdict="handleTraceVerdict"
		/>
	</div>
</template>
