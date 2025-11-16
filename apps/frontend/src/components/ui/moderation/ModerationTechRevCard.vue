<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	CheckIcon,
	ChevronDownIcon,
	CopyIcon,
	DownloadIcon,
	MoreVerticalIcon,
	ShieldCheckIcon,
	TriangleAlertIcon,
} from '@modrinth/assets'
import { Avatar, ButtonStyled, getProjectTypeIcon, injectModrinthClient } from '@modrinth/ui'
import { capitalizeString, formatProjectType, highlightCodeLines } from '@modrinth/utils'
import { computed, ref } from 'vue'

const props = defineProps<{
	item: Labrinth.TechReview.Internal.ProjectReview
}>()

const emit = defineEmits<{
	refetch: []
}>()

type Tab = 'Thread' | 'Files'
const tabs: readonly Tab[] = ['Thread', 'Files']
const currentTab = ref<Tab>('Thread')

type SelectedFile = Labrinth.TechReview.Internal.FileReview | null
const selectedFile = ref<SelectedFile>(null)

const client = injectModrinthClient()

const allFiles = computed(() => {
	return props.item.reports.flatMap((report) => report.files)
})

const highestSeverity = computed(() => {
	const severities = props.item.reports
		.flatMap((r) => r.files)
		.flatMap((f) => f.issues)
		.flatMap((i) => i.details)
		.map((d) => d.severity)

	const order = { SEVERE: 3, HIGH: 2, MEDIUM: 1, LOW: 0 } as Record<string, number>
	return severities.sort((a, b) => (order[b] ?? 0) - (order[a] ?? 0))[0] || 'LOW'
})

const severityColor = computed(() => {
	switch (highestSeverity.value) {
		case 'SEVERE':
			return 'text-red bg-highlight-red border-solid border-[1px] border-red'
		case 'HIGH':
			return 'text-orange bg-highlight-orange border-solid border-[1px] border-orange'
		case 'MEDIUM':
			return 'text-blue bg-highlight-blue border-solid border-[1px] border-blue'
		case 'LOW':
		default:
			return 'text-green bg-highlight-green border-solid border-[1px] border-green'
	}
})

const formattedDate = computed(() => {
	const dates = props.item.reports.map((r) => new Date(r.created_at))
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

function viewFileFlags(file: Labrinth.TechReview.Internal.FileReview) {
	selectedFile.value = file
}

function backToFileList() {
	selectedFile.value = null
}

async function copyToClipboard(code: string) {
	try {
		await navigator.clipboard.writeText(code)
		showCopyFeedback.value = true
		setTimeout(() => {
			showCopyFeedback.value = false
		}, 2000)
	} catch (error) {
		console.error('Failed to copy code:', error)
	}
}

async function updateIssueStatus(
	issueId: string,
	status: Labrinth.TechReview.Internal.DelphiReportIssueStatus,
) {
	try {
		await client.labrinth.tech_review_internal.updateIssue(issueId, { status })
		emit('refetch')
	} catch (error) {
		console.error('Failed to update issue status:', error)
	}
}

const expandedIssues = ref<Set<string>>(new Set())
const showCopyFeedback = ref(false)

function toggleIssue(issueId: string) {
	if (expandedIssues.value.has(issueId)) {
		expandedIssues.value.delete(issueId)
	} else {
		expandedIssues.value.add(issueId)
	}
}

function getSeverityBreakdown(file: Labrinth.TechReview.Internal.FileReview) {
	const counts = {
		SEVERE: 0,
		HIGH: 0,
		MEDIUM: 0,
		LOW: 0,
	}

	file.issues.forEach((issue) => {
		issue.details.forEach((detail) => {
			if (detail.severity in counts) {
				counts[detail.severity as keyof typeof counts]++
			}
		})
	})

	const breakdown = []
	if (counts.SEVERE > 0) breakdown.push({ count: counts.SEVERE, severity: 'SEVERE' })
	if (counts.HIGH > 0) breakdown.push({ count: counts.HIGH, severity: 'HIGH' })
	if (counts.MEDIUM > 0) breakdown.push({ count: counts.MEDIUM, severity: 'MEDIUM' })
	if (counts.LOW > 0) breakdown.push({ count: counts.LOW, severity: 'LOW' })

	return breakdown
}
</script>

<template>
	<div class="shadow-card overflow-hidden rounded-2xl border border-surface-5 bg-surface-3">
		<div class="flex flex-col gap-4 bg-surface-3 p-4">
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-4">
					<Avatar
						:src="item.project.icon_url"
						class="rounded-2xl border border-surface-5 bg-surface-4"
						size="4rem"
					/>

					<div class="flex flex-col gap-1.5">
						<div class="flex items-center gap-2">
							<span class="text-lg font-semibold text-contrast">{{ item.project.name }}</span>

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
								class="rounded-full border border-surface-5 bg-surface-4"
								size="1.5rem"
								circle
							/>
							<span class="text-sm font-medium text-secondary">{{ item.project_owner.name }}</span>
						</div>
					</div>
				</div>

				<div class="flex items-center gap-3">
					<span class="text-base text-secondary">{{ formattedDate }}</span>
					<div class="flex items-center gap-2">
						<ButtonStyled color="green">
							<button><ShieldCheckIcon /> Safe</button>
						</ButtonStyled>

						<ButtonStyled color="red">
							<button><TriangleAlertIcon /> Malware</button>
						</ButtonStyled>

						<ButtonStyled circular>
							<button><MoreVerticalIcon /></button>
						</ButtonStyled>
					</div>
				</div>
			</div>

			<div class="h-px w-full bg-surface-5"></div>

			<div class="flex items-center gap-1 rounded-full bg-surface-3 p-1">
				<div
					v-for="tab in tabs"
					:key="tab"
					class="rounded-full px-3 py-1.5 text-base font-semibold transition-colors hover:cursor-pointer"
					:class="{
						'bg-highlight-green text-green':
							currentTab === tab && !(tab === 'Files' && selectedFile),
						'text-contrast': currentTab !== tab || (tab === 'Files' && selectedFile),
					}"
					@click="
						() => {
							currentTab = tab
							backToFileList()
						}
					"
				>
					{{ tab }}
				</div>

				<div
					v-if="currentTab === 'Files' && selectedFile"
					class="rounded-full bg-highlight-green px-3 py-1.5 text-base font-semibold text-green"
				>
					{{ selectedFile.file_name }}
				</div>
			</div>
		</div>

		<div class="border-t border-surface-3 bg-surface-2">
			<div v-if="currentTab === 'Thread'" class="p-4">
				<div v-if="true" class="flex min-h-[75px] items-center justify-center">
					<div class="text-center text-secondary">
						<p class="text-sm">No messages yet {{ ':(' }}</p>
					</div>
				</div>

				<div v-else class="flex flex-col gap-6">
					<!-- TODO: Report thread stuff -->
				</div>
			</div>

			<div v-else-if="currentTab === 'Files' && !selectedFile" class="flex flex-col">
				<div
					v-for="(file, idx) in allFiles"
					:key="idx"
					class="flex items-center justify-between border-0 border-x border-b border-solid border-surface-3 bg-surface-2 px-4 py-3"
					:class="{ 'rounded-bl-2xl rounded-br-2xl': idx === allFiles.length - 1 }"
				>
					<div class="flex items-center gap-3">
						<span class="font-medium text-contrast">{{ file.file_name }}</span>
						<div class="rounded-full border border-solid border-surface-5 bg-surface-3 px-2.5 py-1">
							<span class="text-sm font-medium text-secondary">{{
								formatFileSize(file.file_size)
							}}</span>
						</div>
						<div
							v-for="severityItem in getSeverityBreakdown(file)"
							:key="severityItem.severity"
							class="rounded-full border border-solid px-2.5 py-1"
							:class="{
								'border-red/60 bg-highlight-red text-red': severityItem.severity === 'SEVERE',
								'border-orange/60 bg-highlight-orange text-orange':
									severityItem.severity === 'HIGH',
								'border-blue/60 bg-highlight-blue text-blue': severityItem.severity === 'MEDIUM',
								'border-green/60 bg-highlight-green text-green': severityItem.severity === 'LOW',
							}"
						>
							<span class="text-sm font-medium"
								>{{ severityItem.count }}
								{{ capitalizeString(severityItem.severity.toLowerCase()) }}</span
							>
						</div>
					</div>

					<div class="flex items-center gap-2">
						<ButtonStyled>
							<button @click="viewFileFlags(file)">Flags</button>
						</ButtonStyled>

						<ButtonStyled outline>
							<button><DownloadIcon /> Download</button>
						</ButtonStyled>
					</div>
				</div>
			</div>

			<div v-else-if="currentTab === 'Files' && selectedFile" class="flex flex-col">
				<div
					v-for="(issue, idx) in selectedFile.issues"
					:key="issue.issue_id"
					class="border-x border-b border-t-0 border-solid border-surface-3 bg-surface-2"
					:class="{ 'rounded-bl-2xl rounded-br-2xl': idx === selectedFile.issues.length - 1 }"
				>
					<div class="flex items-center justify-between p-4">
						<div class="my-auto flex items-center gap-2">
							<ButtonStyled type="transparent" circular>
								<button
									class="transition-transform"
									:class="{ 'rotate-180': !expandedIssues.has(issue.issue_id) }"
									@click="toggleIssue(issue.issue_id)"
								>
									<ChevronDownIcon class="h-5 w-5 text-contrast" />
								</button>
							</ButtonStyled>

							<span class="text-base font-semibold text-contrast">{{
								issue.kind.replace(/_/g, ' ')
							}}</span>

							<div
								v-if="issue.details.length > 0"
								class="rounded-full px-2.5 py-1"
								:class="{
									'border-red/60 border bg-highlight-red text-red':
										issue.details[0].severity === 'SEVERE',
									'border-orange/60 border bg-highlight-orange text-orange':
										issue.details[0].severity === 'HIGH' || issue.details[0].severity === 'MEDIUM',
									'border-green/60 border bg-highlight-green text-green':
										issue.details[0].severity === 'LOW',
								}"
							>
								<span class="text-sm font-medium">{{
									issue.details[0].severity.charAt(0) +
									issue.details[0].severity.slice(1).toLowerCase()
								}}</span>
							</div>
						</div>

						<div class="flex items-center gap-2">
							<ButtonStyled color="brand" type="outlined">
								<button class="!border-[1px]" @click="updateIssueStatus(issue.issue_id, 'safe')">
									Safe
								</button>
							</ButtonStyled>

							<ButtonStyled color="red" type="outlined">
								<button class="!border-[1px]" @click="updateIssueStatus(issue.issue_id, 'unsafe')">
									Malware
								</button>
							</ButtonStyled>
						</div>
					</div>

					<div v-if="expandedIssues.has(issue.issue_id)" class="flex flex-col gap-4 px-4 pb-4">
						<div
							v-for="(detail, detailIdx) in issue.details"
							:key="detailIdx"
							class="flex flex-col"
						>
							<p class="mt-0 pt-0 font-mono text-sm text-secondary">{{ detail.class_name }}</p>

							<div
								v-if="detail.decompiled_source"
								class="relative overflow-hidden rounded-lg border border-solid border-surface-5 bg-surface-4"
							>
								<ButtonStyled circular type="transparent">
									<button
										v-tooltip="`Copy code`"
										class="absolute right-2 top-2 border-[1px]"
										@click="copyToClipboard(detail.decompiled_source)"
									>
										<CopyIcon v-if="!showCopyFeedback" />
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
						</div>
					</div>
				</div>
			</div>
		</div>
	</div>
</template>

<style scoped>
pre {
	all: unset;
	display: inline;
	white-space: pre;
}
</style>
