<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	CheckIcon,
	ChevronDownIcon,
	ChevronRightIcon,
	CopyIcon,
	LoaderCircleIcon,
} from '@modrinth/assets'
import { Collapsible, IconButton, injectNotificationManager, Toggle } from '@modrinth/ui'
import { capitalizeString, highlightCodeLines } from '@modrinth/utils'
import { computed, nextTick, reactive, ref, watch } from 'vue'

import {
	canUpdateGlobalDetail,
	getFileDetailCount,
	getSeverityBadgeColor,
	severityOrder,
	truncateMiddle,
	verdictToDecision,
} from './helpers'
import TechRevVerdictButtons from './TechRevVerdictButtons.vue'
import type { ClassGroup, FlagItem, FlattenedFileReport, JarGroup } from './types'
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
const showCopyFeedback = reactive<Map<string, boolean>>(new Map())
const highlightedSourceCache = reactive<Map<string, { source: string; lines: string[] }>>(new Map())
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

function maybeReturnToFileList() {
	if (getFileMarkedCount(props.file) === getFileDetailCount(props.file)) {
		emit('allFlagsResolved')
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

		addNotification({
			type: 'success',
			title: `Globally marked ${details.length} trace keys as ${verdict}`,
			text: `All remaining eligible traces have been globally marked as ${
				verdict === 'safe' ? 'false positives' : 'malicious'
			}.`,
		})

		maybeReturnToFileList()
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

		addNotification({
			type: 'success',
			title: `Marked ${detailIds.length} traces as ${verdict}`,
			text: `All remaining traces${inJar ? ' in this JAR' : ''} have been marked as ${
				verdict === 'safe' ? 'false positives' : 'malicious'
			}.`,
		})

		maybeReturnToFileList()
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

async function updateDetailStatus(
	detailId: string,
	verdict: Labrinth.TechReview.Internal.DelphiReportIssueStatus,
) {
	const detail = props.file.issues.flatMap((issue) => issue.details).find((d) => d.id === detailId)
	const priorDecision = detail ? getDetailDecision(detail.id, detail.status) : 'pending'

	updatingDetails.add(detailId)

	const previousMarkedCount = getFileMarkedCount(props.file)

	try {
		await updateIssueDetails([{ detail_id: detailId, verdict }])

		const { otherMatchedCount } = applyDecisionToRelatedDetails(
			[detailId],
			verdictToDecision(verdict),
			'local',
		)

		if (verdict !== 'pending' && priorDecision === 'pending') {
			for (const classGroup of groupedByClass.value) {
				const hasThisDetail = classGroup.flags.some((f) => f.detail.id === detailId)
				if (hasThisDetail && getMarkedFlagsCount(classGroup.flags) === classGroup.flags.length) {
					expandedClasses.delete(classGroup.key)
					break
				}
			}
		}

		if (verdict !== 'pending') {
			const markedCount = getFileMarkedCount(props.file)
			const totalCount = getFileDetailCount(props.file)
			if (previousMarkedCount != markedCount && markedCount === totalCount) {
				emit('allFlagsResolved')
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

	const previousMarkedCount = getFileMarkedCount(props.file)

	try {
		await updateGlobalIssueDetails([{ detail_key: detail.key, verdict }])

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

		if (verdict !== 'pending') {
			const markedCount = getFileMarkedCount(props.file)
			const totalCount = getFileDetailCount(props.file)
			if (previousMarkedCount != markedCount && markedCount === totalCount) {
				emit('allFlagsResolved')
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

function getHighestSeverityInClass(flags: FlagItem[]): Labrinth.TechReview.Internal.DelphiSeverity {
	return flags.reduce(
		(highest, flag) =>
			severityOrder[flag.detail.severity] > severityOrder[highest] ? flag.detail.severity : highest,
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

		<div
			v-for="classItem in jarGroup.classes"
			:key="classItem.key"
			class="overflow-clip rounded-xl border border-solid border-surface-4"
		>
			<div
				class="flex cursor-pointer items-center justify-between bg-surface-3 p-2 transition-colors duration-200 hover:bg-surface-4"
				@click="toggleClass(classItem)"
			>
				<div class="my-auto flex items-center gap-2">
					<IconButton
						type="quiet"
						label="Toggle details"
						class="transition-transform"
						:class="{ 'rotate-180': expandedClasses.has(classItem.key) }"
					>
						<ChevronDownIcon class="h-5 w-5 text-contrast" />
					</IconButton>

					<span v-tooltip="classItem.filePath" class="font-mono text-sm font-semibold">{{
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
				<div class="flex flex-col gap-2 border-0 border-t border-solid border-surface-4 p-2">
					<div
						v-for="flag in classItem.flags"
						:id="`tech-review-detail-${flag.detail.id}`"
						:key="`${flag.issueId}-${flag.detail.id}`"
						class="flex flex-col gap-2 rounded-lg border border-solid border-surface-5 bg-surface-3 py-2 pl-4 last:border-b-0"
						:class="{
							'!border-brand bg-brand-highlight': focusedDetailId === flag.detail.id,
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

							<div class="me-2 flex items-center justify-end gap-2">
								<TechRevVerdictButtons
									variant="trace"
									:detail="flag.detail"
									@global-safe="updateGlobalDetailAction(flag.detail, 'safe')"
									@local-safe="updateLocalDetailAction(flag.detail, 'safe')"
									@local-unsafe="updateLocalDetailAction(flag.detail, 'malware')"
									@global-unsafe="updateGlobalDetailAction(flag.detail, 'malware')"
								/>
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
						<IconButton
							v-tooltip="`Copy code`"
							type="quiet"
							:label="`Copy code`"
							class="!absolute right-2 top-2 border-[1px]"
							@click="copyToClipboard(getClassDecompiledSource(classItem)!, classItem.key)"
						>
							<CopyIcon v-if="!showCopyFeedback.get(classItem.key)" />
							<CheckIcon v-else />
						</IconButton>

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
					<div v-else class="rounded-lg border border-solid border-surface-5 bg-surface-3 p-4">
						<p class="text-sm text-secondary">
							Source code not available or failed to decompile for this file.
						</p>
					</div>
				</div>
			</Collapsible>
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
