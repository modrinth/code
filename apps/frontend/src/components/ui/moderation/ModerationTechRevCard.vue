<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	CheckIcon,
	CodeIcon,
	DownloadIcon,
	ExternalIcon,
	TimerIcon,
	VersionIcon,
} from '@modrinth/assets'
import {
	Avatar,
	ButtonLink,
	CopyCode,
	CopyLinkButton,
	getProjectTypeIcon,
	NavTabs,
	useFormatBytes,
} from '@modrinth/ui'
import { capitalizeString, formatProjectType } from '@modrinth/utils'
import { computed, provide, ref, watch } from 'vue'

import type { UnsafeFile } from '~/components/ui/moderation/MaliciousSummaryModal.vue'
import {
	getFileHighestSeverity,
	getSeverityBadgeColor,
	getVersionLabel,
	getVersionPageHref,
	severityOrder,
} from '~/components/ui/moderation/tech-review/helpers'
import TechRevFileDetailTab from '~/components/ui/moderation/tech-review/TechRevFileDetailTab.vue'
import TechRevFilesTab from '~/components/ui/moderation/tech-review/TechRevFilesTab.vue'
import TechRevThreadTab from '~/components/ui/moderation/tech-review/TechRevThreadTab.vue'
import type { FlattenedFileReport } from '~/components/ui/moderation/tech-review/types'
import {
	TECH_REVIEW_DECISIONS_KEY,
	useTechReviewDecisions,
} from '~/components/ui/moderation/tech-review/use-tech-review-decisions'

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
	collapsed: boolean
	disableCollapsing?: boolean
}>()

const formatBytes = useFormatBytes()

const emit = defineEmits<{
	refetch: []
	loadIssueSources: [issueIds: string[]]
	markComplete: [projectId: string]
	showMaliciousSummary: [unsafeFiles: UnsafeFile[]]
}>()

const decisions = useTechReviewDecisions(() => props.item.reports)
provide(TECH_REVIEW_DECISIONS_KEY, decisions)

const projectStatus = ref<Labrinth.Projects.v2.ProjectStatus>(props.item.project.status)
const isProjectApproved = computed(() => {
	return (
		projectStatus.value === 'approved' ||
		projectStatus.value === 'archived' ||
		projectStatus.value === 'unlisted' ||
		projectStatus.value === 'private'
	)
})

const currentTab = ref<'Thread' | 'Files' | 'File'>('Thread')
const tabs = ['Thread', 'Files'] as const
const isThreadCollapsed = ref(props.collapsed)

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

const highestSeverity = computed(() => {
	let highest: Labrinth.TechReview.Internal.DelphiSeverity = 'low'
	for (const report of props.item.reports) {
		const severity = getFileHighestSeverity(report)
		if (severityOrder[severity] > severityOrder[highest]) highest = severity
	}
	return highest
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
	const idx = tabs.indexOf(currentTab.value as (typeof tabs)[number])
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
		currentTab.value = 'File'
	}
}

const severityColor = computed(() => getSeverityBadgeColor(highestSeverity.value))

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

function backToFileList() {
	selectedFileId.value = null
	if (currentTab.value === 'File') {
		currentTab.value = 'Files'
	}
}

watch(
	() => props.focusedDetailId,
	(detailId) => {
		if (detailId) {
			const file = findFileForDetail(detailId)
			if (file) viewFileFlags(file)
		}
	},
	{ immediate: true },
)
</script>

<template>
	<div
		class="shadow-card overflow-hidden rounded-2xl border border-solid border-surface-4 bg-surface-3"
	>
		<div
			class="flex flex-col gap-4 border-0 border-b border-solid border-surface-4 bg-surface-3 p-4"
		>
			<div class="flex items-start justify-between">
				<div class="flex items-center gap-3">
					<NuxtLink
						:to="`/${item.project.project_types[0]}/${item.project.slug ?? item.project.id}`"
						target="_blank"
						tabindex="-1"
					>
						<Avatar
							:src="item.project.icon_url"
							class="rounded-2xl border border-surface-5 bg-surface-4 !shadow-none"
							size="4rem"
						/>
					</NuxtLink>

					<div class="flex flex-col gap-1.5">
						<div class="flex items-center gap-2">
							<NuxtLink
								:to="`/${item.project.project_types[0]}/${item.project.slug ?? item.project.id}`"
								target="_blank"
								class="text-lg font-semibold text-contrast hover:underline focus-visible:underline"
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

							<div class="rounded-full border-solid px-2.5 py-1" :class="severityColor">
								<span class="text-sm font-medium">{{
									capitalizeString(highestSeverity.toLowerCase())
								}}</span>
							</div>
						</div>

						<div class="flex items-center gap-2">
							<NuxtLink
								:to="`/${item.project_owner.kind}/${item.project_owner.id}`"
								target="_blank"
								class="flex items-center gap-1 text-sm font-medium text-secondary hover:underline"
							>
								<Avatar
									:src="item.project_owner.icon_url"
									class="rounded-full border border-surface-5 bg-surface-4 !shadow-none"
									size="1.5rem"
									circle
								/>
								{{ item.project_owner.name }}
							</NuxtLink>
							<CopyCode v-tooltip="'Copy user ID'" :text="item.project_owner.id" />
						</div>
					</div>
				</div>

				<div class="flex items-center gap-3">
					<span class="text-base text-secondary">{{ formattedDate }}</span>
					<div class="flex items-center gap-2">
						<ButtonLink
							v-if="props.item.project.link_urls?.['source']?.url"
							v-tooltip="'Open sources in new tab'"
							:href="props.item.project.link_urls?.['source']?.url"
							target="_blank"
							class="!w-9 !rounded-full !px-0"
						>
							<CodeIcon />
						</ButtonLink>
						<CopyCode v-tooltip="'Copy project ID'" :text="item.project.id" />
						<CopyLinkButton
							copy-label="Copy project link"
							:url="`https://modrinth.com/moderation/technical-review/${props.item.project.id}`"
						/>
						<ButtonLink
							v-tooltip="'Open tech review in new tab'"
							:href="`/moderation/technical-review/${props.item.project.id}`"
							target="_blank"
							circular
							icon-only
						>
							<ExternalIcon />
						</ButtonLink>
					</div>
				</div>
			</div>
			<div class="flex flex-row justify-between">
				<NavTabs
					mode="local"
					:links="navTabsLinks"
					:active-index="activeTabIndex"
					class="border border-solid border-surface-4 bg-surface-2"
					@tab-click="handleTabClick"
				/>

				<div v-if="currentTab === 'File' && selectedFile" class="flex flex-row items-end gap-2">
					<ButtonLink
						type="outlined"
						target="_blank"
						:href="getVersionPageHref(item.project, selectedFile.version_id)"
						class="!bg-surface-2"
						:aria-label="`Open version ${getVersionLabel(selectedFile)}`"
					>
						<VersionIcon aria-hidden="true" />
						{{ getVersionLabel(selectedFile) }}
					</ButtonLink>
					<ButtonLink
						type="outlined"
						target="_blank"
						:href="`https://slicer.run/?url=${encodeURIComponent(selectedFile.download_url)}`"
						class="!bg-surface-2"
						aria-label="Open in Slicer"
					>
						<ExternalIcon aria-hidden="true" /> Slicer
					</ButtonLink>
					<ButtonLink
						v-tooltip="
							`Download ${selectedFile.file_name} (${formatBytes(selectedFile.file_size)})`
						"
						type="outlined"
						target="_blank"
						:href="selectedFile.download_url"
						:download="selectedFile.file_name"
						class="!bg-surface-2"
						aria-label="Download"
						icon-only
						circular
					>
						<DownloadIcon aria-hidden="true" />
					</ButtonLink>
				</div>
			</div>
		</div>

		<div class="bg-surface-2">
			<TechRevThreadTab
				v-if="currentTab === 'Thread'"
				v-model:collapsed="isThreadCollapsed"
				:project="item.project"
				:project-owner="item.project_owner"
				:thread="item.thread"
				:reports="item.reports"
				:disable-collapsing="disableCollapsing"
				@refetch="emit('refetch')"
				@mark-complete="emit('markComplete', $event)"
				@show-malicious-summary="emit('showMaliciousSummary', $event)"
				@status-changed="projectStatus = $event"
			/>
			<TechRevFilesTab
				v-else-if="currentTab === 'Files'"
				:reports="item.reports"
				:project="item.project"
				@view-flags="viewFileFlags"
			/>
			<TechRevFileDetailTab
				v-else-if="currentTab === 'File' && selectedFile"
				:file="selectedFile"
				:focused-detail-id="focusedDetailId"
				:loading-issues="loadingIssues"
				:decompiled-sources="decompiledSources"
				@refetch="emit('refetch')"
				@load-issue-sources="emit('loadIssueSources', $event)"
				@all-flags-resolved="backToFileList"
			/>
		</div>
	</div>
</template>
