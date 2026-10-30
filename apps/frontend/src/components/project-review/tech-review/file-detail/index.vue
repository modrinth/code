<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { injectNotificationManager, Toggle } from '@modrinth/ui'
import { useLocalStorage } from '@vueuse/core'
import { computed, nextTick, reactive, ref, watch } from 'vue'

import IssueDetailPath from '~/components/ui/moderation/IssueDetailPath.vue'

import { canUpdateGlobalDetail, getFileDetailCount, verdictToDecision } from '../helpers'
import PanelDivider from '../panel-divider.vue'
import type {
	ClassGroup,
	FlagItem,
	FlattenedFileReport,
	JarGroup,
	TraceVerdictEvent,
} from '../types'
import VerdictButtons from '../verdict/buttons.vue'
import { injectTechReviewDecisions } from '../verdict/use-decisions'
import ArchiveTree from './archive-tree.vue'
import ClassItem from './class-item.vue'
import { isUsableDecompiledSource } from './source-status'

const props = defineProps<{
	file: FlattenedFileReport
	fileTree?: boolean
	focusedDetailId?: string | null
	loadingIssues: Set<string>
	settledIssues: Set<string>
	decompiledSources: Map<string, string>
}>()

const emit = defineEmits<{
	refetch: []
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

const treeWidth = useLocalStorage('moderation-tech-review-tree-width', 256)
const tracesWidth = useLocalStorage('moderation-tech-review-traces-width', 360)
const panel = ref<HTMLElement>()
const archiveTree = ref<InstanceType<typeof ArchiveTree>>()
const classDetail = ref<InstanceType<typeof ClassItem>>()

defineExpose({
	navigateFile: (direction: number) => archiveTree.value?.navigateFile(direction) ?? false,
	navigateHighlight: (direction: number) => classDetail.value?.navigateHighlight(direction) ?? false,
})
const selectedClassKey = ref<string>()
const selectedClass = computed(() =>
	groupedByClass.value.find((item) => item.key === selectedClassKey.value),
)

function selectClass(classItem: ClassGroup) {
	selectedClassKey.value = classItem.key
}

const showGloballyPassed = useLocalStorage('moderation-tech-review-show-globally-passed', false)
const showHidden = useLocalStorage('moderation-tech-review-show-hidden', false)
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

const filterCounts = computed(() => {
	let globallyPassed = 0
	let hidden = 0
	for (const { detail } of selectedFileFlags.value) {
		const passed = isDetailGloballyPassed(detail)
		if (passed && (showHidden.value || detail.severity !== 'hidden')) globallyPassed++
		if (detail.severity === 'hidden' && (showGloballyPassed.value || !passed)) hidden++
	}
	return { globallyPassed, hidden }
})

const fileVerdictProps = computed(() => ({
	variant: 'remaining' as const,
	remainingCount: remainingUnmarkedCount.value,
	remainingFlags: selectedFileFlags.value.filter(
		({ detail }) => getDetailDecision(detail.id, detail.status) === 'pending',
	),
	onSelectFile: async (fileKey: string) => {
		const flags = selectedFileFlags.value.filter(
			({ detail }) => `${detail.jar ?? ''}::${detail.file_path}` === fileKey,
		)
		if (flags.every(({ detail }) => detail.severity === 'hidden')) showHidden.value = true
		if (flags.every(({ detail }) => isDetailGloballyPassed(detail))) showGloballyPassed.value = true
		await nextTick()
		const group = groupedByClass.value.find((item) => item.key === fileKey)
		if (group) {
			selectClass(group)
			expandClass(group)
		}
	},
	globalDisabled:
		isBatchUpdating.value || getRemainingGlobalDetailCount(selectedFileFlags.value) === 0,
	localDisabled: isBatchUpdating.value,
	onGlobalSafe: () => batchMarkRemainingGlobally(selectedFileFlags.value, 'safe'),
	onLocalSafe: () => batchMarkRemaining(selectedFileFlags.value, 'safe'),
	onLocalUnsafe: () => batchMarkRemaining(selectedFileFlags.value, 'unsafe'),
	onGlobalUnsafe: () => batchMarkRemainingGlobally(selectedFileFlags.value, 'unsafe'),
}))

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
		.split('#')
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
			if (!showGloballyPassed.value && isDetailGloballyPassed(detail)) {
				continue
			}

			if (props.fileTree && !showHidden.value && detail.severity === 'hidden') continue

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

const unavailableSources = computed(
	() =>
		new Set(
			groupedByClass.value
				.filter(
					(group) =>
						group.flags.length > 0 &&
						group.flags.every(
							(flag) =>
								props.settledIssues.has(flag.issueId) && !props.loadingIssues.has(flag.issueId),
						) &&
						!group.flags.some((flag) =>
							isUsableDecompiledSource(props.decompiledSources.get(flag.detail.id)),
						),
				)
				.map((group) => group.key),
		),
)

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

function expandClass(classItem: ClassGroup) {
	if (expandedClasses.has(classItem.key)) return
	expandedClasses.add(classItem.key)
}

function toggleClass(classItem: ClassGroup) {
	if (expandedClasses.has(classItem.key)) {
		expandedClasses.delete(classItem.key)
	} else {
		expandClass(classItem)
	}
}

async function focusDetail(detailId: string) {
	const detail = props.file.issues
		.flatMap((issue) => issue.details)
		.find((item) => String(item.id) === detailId)
	if (detail?.severity === 'hidden') showHidden.value = true
	if (detail && isDetailGloballyPassed(detail)) showGloballyPassed.value = true
	await nextTick()

	const classItem = groupedByClass.value.find((group) =>
		group.flags.some((flag) => String(flag.detail.id) === detailId),
	)

	if (classItem) {
		if (props.fileTree) selectClass(classItem)
		expandClass(classItem)
	}

	await nextTick()

	const view = panel.value?.ownerDocument.defaultView
	if (!view) return

	view.requestAnimationFrame(() => {
		const trace = panel.value?.ownerDocument.getElementById(`tech-review-detail-${detailId}`)
		if (!trace) return
		const reducedMotion = view.matchMedia('(prefers-reduced-motion: reduce)').matches
		trace.scrollIntoView({
			behavior: reducedMotion ? 'auto' : 'smooth',
			block: 'center',
		})
		if (!reducedMotion) {
			trace.getAnimations().forEach((animation) => animation.cancel())
			trace.animate(
				[
					{ backgroundColor: 'color-mix(in srgb, var(--color-orange) 25%, transparent)' },
					{ backgroundColor: 'transparent' },
				],
				{ duration: 1100, delay: 150, easing: 'ease-out', fill: 'backwards' },
			)
		}
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
	groupedByClass,
	(classes) => {
		if (!props.fileTree || classes.some((item) => item.key === selectedClassKey.value)) return
		const first = classes[0]
		if (first) selectClass(first)
		else selectedClassKey.value = undefined
	},
	{ immediate: true },
)

watch(
	[() => props.file.id, groupedByClass],
	([fileId, classes]) => {
		if (props.fileTree || !fileId || classes.length === 0 || autoExpandedFileIds.has(fileId)) return

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
	<div ref="panel" :class="{ 'flex min-h-0 flex-1 flex-col': fileTree }">
		<slot name="header" :verdict-props="fileVerdictProps" />
		<div
			v-if="getFileDetailCount(file) > 0 && (!$slots.header || !fileTree)"
			class="flex shrink-0 flex-wrap items-center justify-between gap-3 pb-3"
		>
			<VerdictButtons v-bind="fileVerdictProps" />
			<label v-if="!fileTree" class="ml-auto flex cursor-pointer items-center gap-3 text-sm">
				<span class="text-right text-secondary">
					Show globally passed
					<span class="block text-xs text-secondary">
						{{ globallyResolvedCount }}/{{ getFileDetailCount(file) }} traces globally resolved
					</span>
				</span>
				<Toggle v-model="showGloballyPassed" :disabled="globallyPassedCount === 0" small />
			</label>
		</div>
		<div
			v-if="fileTree"
			class="archive-workspace grid min-h-0 flex-1"
			:style="{ '--tree-width': `${treeWidth}px` }"
		>
			<ArchiveTree
				ref="archiveTree"
				v-model:show-globally-passed="showGloballyPassed"
				v-model:show-hidden="showHidden"
				:classes="groupedByClass"
				:unavailable-sources="unavailableSources"
				:globally-passed-count="filterCounts.globallyPassed"
				:hidden-count="filterCounts.hidden"
				:selected-key="selectedClassKey"
				:file-name="file.file_name ?? ''"
				@select="selectClass"
			/>
			<PanelDivider
				v-model="treeWidth"
				label="File tree width"
				width-property="--tree-width"
				side="left"
				:min="140"
				:max="480"
				:reserve="320"
				:default-width="256"
			/>
			<ClassItem
				v-if="selectedClass"
				ref="classDetail"
				:key="selectedClass.key"
				v-model:traces-width="tracesWidth"
				v-model:show-hidden-traces="showHidden"
				:file="file"
				:class-item="selectedClass"
				expanded
				split-view
				:focused-detail-id="focusedDetailId"
				:loading-issues="loadingIssues"
				:decompiled-sources="decompiledSources"
				@verdict="handleTraceVerdict"
			/>
			<p v-else class="m-0 p-4 text-secondary">No files match the current filters.</p>
		</div>
		<template v-else>
			<div v-for="jarGroup in groupedByJar" :key="jarGroup.key" class="flex flex-col gap-1 pb-4">
				<div v-if="jarGroup.segments.length > 0" class="my-2">
					<div class="flex flex-wrap items-center justify-between gap-3">
						<IssueDetailPath
							:segments="jarGroup.segments"
							:decode="false"
							class="font-mono text-sm"
							emphasize-last
						/>

						<VerdictButtons
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

				<ClassItem
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
	</div>
</template>

<style scoped>
.archive-workspace {
	grid-template-columns: min(var(--tree-width), max(140px, calc(100% - 328px))) 8px minmax(0, 1fr);
}
</style>
