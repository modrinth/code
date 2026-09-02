<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { CheckIcon, CodeIcon, ExternalIcon, TimerIcon, VersionIcon } from '@modrinth/assets'
import { ButtonLink, CopyCode, CopyLinkButton, getProjectTypeIcon, NavTabs } from '@modrinth/ui'
import { capitalizeString, formatProjectType } from '@modrinth/utils'
import { computed, provide, ref, watch } from 'vue'

import type { UnsafeFile } from '~/components/ui/moderation/MaliciousSummaryModal.vue'
import ModerationItemHeader from '~/components/ui/moderation/ModerationItemHeader.vue'
import {
	getHighestSeverity,
	getSeverityBadgeColor,
	getVersionPageHref,
} from '~/components/ui/moderation/tech-review/helpers'
import TechRevFileActions from '~/components/ui/moderation/tech-review/TechRevFileActions.vue'
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

const highestSeverity = computed(() =>
	getHighestSeverity(
		props.item.reports.flatMap((report) => report.issues.flatMap((issue) => issue.details)),
	),
)

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
			class="flex flex-col gap-3 border-0 border-b border-solid border-surface-4 bg-surface-3 p-4 pb-3"
		>
			<div class="flex flex-wrap items-start justify-between">
				<ModerationItemHeader
					:avatar-url="item.project.icon_url"
					:title="item.project.name"
					:title-to="`/${item.project.project_types[0]}/${item.project.slug ?? item.project.id}`"
					:owner="item.project_owner"
				>
					<template #badges>
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
							<span class="text-sm font-medium">
								{{ capitalizeString(highestSeverity.toLowerCase()) }}
							</span>
						</div>
					</template>
				</ModerationItemHeader>

				<div class="flex flex-col items-end gap-2">
					<div class="flex flex-wrap items-center justify-end gap-3">
						<span class="text-base text-secondary">{{ formattedDate }}</span>
						<div class="flex items-center gap-2">
							<ButtonLink
								v-if="props.item.project.link_urls?.['source']?.url"
								v-tooltip="'Open sources in new tab'"
								:href="props.item.project.link_urls?.['source']?.url"
								target="_blank"
								circular
								icon-only
							>
								<CodeIcon />
							</ButtonLink>
							<CopyLinkButton
								copy-label="Copy tech review link"
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
					<CopyCode v-tooltip="'Copy project ID'" :text="item.project.id" />
				</div>
			</div>
			<div class="flex flex-row flex-wrap justify-between">
				<NavTabs
					mode="local"
					:links="navTabsLinks"
					:active-index="activeTabIndex"
					class="border border-solid border-surface-4 bg-surface-2"
					@tab-click="handleTabClick"
				/>
				<div v-if="currentTab === 'File' && selectedFile" class="flex items-center gap-2">
					<div class="flex items-center gap-1">
						<ButtonLink
							v-tooltip="'View version'"
							type="outlined"
							target="_blank"
							:href="getVersionPageHref(item.project, selectedFile.version_id)"
						>
							<VersionIcon aria-hidden="true" />
							{{ selectedFile.version_number }}
						</ButtonLink>
					</div>
					<TechRevFileActions :file="selectedFile" />
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
