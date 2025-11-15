<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	ChevronDownIcon,
	DownloadIcon,
	MoreVerticalIcon,
	ShieldCheckIcon,
	TriangleAlertIcon,
} from '@modrinth/assets'
import { Avatar, ButtonStyled, injectModrinthClient } from '@modrinth/ui'
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

function toggleIssue(issueId: string) {
	if (expandedIssues.value.has(issueId)) {
		expandedIssues.value.delete(issueId)
	} else {
		expandedIssues.value.add(issueId)
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
						class="rounded-2xl border border-surface-5 bg-surface-4"
						size="4rem"
					/>

					<div class="flex flex-col gap-1.5">
						<div class="flex items-center gap-2">
							<span class="text-xl font-semibold text-contrast">{{ item.project.title }}</span>

							<div
								class="flex items-center gap-1 rounded-full border border-surface-5 bg-surface-4 px-2.5 py-1"
							>
								<div class="h-4 w-4"></div>
								<span class="text-sm font-medium text-secondary">{{
									String(item.project.project_type).toUpperCase()
								}}</span>
							</div>

							<div class="rounded-full border border-surface-5 bg-surface-4 px-2.5 py-1">
								<span class="text-sm font-medium text-secondary">Auto-Flagged</span>
							</div>

							<div class="rounded-full px-2.5 py-1" :class="severityColor">
								<span class="text-sm font-medium">{{
									highestSeverity.charAt(0) + highestSeverity.slice(1).toLowerCase()
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
				<button
					v-for="tab in tabs"
					:key="tab"
					class="rounded-full px-3 py-1.5 text-base font-semibold transition-colors"
					:class="{
						'bg-green/30 text-green': currentTab === tab,
						'text-contrast': currentTab !== tab,
					}"
					@click="
						() => {
							currentTab = tab
							backToFileList()
						}
					"
				>
					{{ tab }}
				</button>

				<span
					v-if="currentTab === 'Files' && selectedFile"
					class="ml-2 text-sm font-medium text-secondary"
				>
					{{ selectedFile.file_name }}
				</span>
			</div>
		</div>

		<div class="border-t border-surface-3 bg-surface-2">
			<div v-if="currentTab === 'Thread'" class="p-4">
				<div
					v-if="item.thread.messages.length === 0"
					class="flex min-h-[200px] items-center justify-center"
				>
					<div class="text-center text-secondary">
						<p class="text-sm">No messages yet</p>
					</div>
				</div>

				<div v-else class="flex flex-col gap-6">
					<div v-for="message in item.thread.messages" :key="message.id" class="flex gap-3">
						<Avatar
							src="https://via.placeholder.com/40"
							class="rounded-full border border-surface-5"
							size="2.5rem"
							circle
						/>

						<div class="flex flex-1 flex-col">
							<div class="flex items-end gap-2">
								<span class="font-semibold text-contrast">{{ message.author_id || 'System' }}</span>
								<span class="text-xs text-secondary">{{
									new Date(message.created).toLocaleTimeString()
								}}</span>
							</div>

							<p v-if="message.body.type === 'text'" class="text-secondary">
								{{ message.body.body }}
							</p>
							<p v-else-if="message.body.type === 'status_change'" class="italic text-secondary">
								Status changed from {{ message.body.old_status }} to {{ message.body.new_status }}
							</p>
							<p v-else class="italic text-secondary">
								{{ message.body.type }}
							</p>
						</div>
					</div>

					<div class="flex flex-col gap-3">
						<div class="flex items-center gap-2">
							<div
								class="flex h-8 w-8 items-center justify-center rounded-full bg-surface-4 text-xs font-medium text-secondary"
							>
								Aa
							</div>
							<div class="flex h-8 w-8 items-center justify-center rounded-full bg-surface-4"></div>
						</div>

						<div class="rounded-2xl bg-surface-4 px-4 py-2.5">
							<span class="text-secondary/60 text-sm">Message @{{ item.project_owner.name }}</span>
						</div>

						<div class="flex items-center justify-between">
							<div class="flex items-center gap-2">
								<ButtonStyled color="green" class="opacity-50">
									<button>Reply</button>
								</ButtonStyled>

								<ButtonStyled class="opacity-50">
									<button>Add note</button>
								</ButtonStyled>

								<ButtonStyled>
									<button>Quick reply <ChevronDownIcon class="-scale-y-100" /></button>
								</ButtonStyled>
							</div>
						</div>
					</div>
				</div>
			</div>

			<div v-else-if="currentTab === 'Files' && !selectedFile" class="flex flex-col">
				<div
					v-for="(file, idx) in allFiles"
					:key="idx"
					class="flex items-center justify-between border-x border-b border-surface-3 bg-surface-2 p-4"
					:class="{ 'rounded-bl-2xl rounded-br-2xl': idx === allFiles.length - 1 }"
				>
					<div class="flex items-center gap-3">
						<span class="font-medium text-contrast">{{ file.file_name }}</span>
						<div class="rounded-full border border-surface-5 bg-surface-3 px-2.5 py-1">
							<span class="text-sm font-medium text-secondary">{{
								formatFileSize(file.file_size)
							}}</span>
						</div>
						<div class="border-red/60 rounded-full border bg-highlight-red px-2.5 py-1">
							<span class="text-sm font-medium text-red">{{ file.issues.length }} flags</span>
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
					class="border-x border-b border-surface-3 bg-surface-2"
					:class="{ 'rounded-bl-2xl rounded-br-2xl': idx === selectedFile.issues.length - 1 }"
				>
					<div class="flex items-center justify-between p-4">
						<div class="flex items-center gap-2">
							<button
								class="transition-transform"
								:class="{ 'rotate-180': !expandedIssues.has(issue.issue_id) }"
								@click="toggleIssue(issue.issue_id)"
							>
								<ChevronDownIcon class="h-5 w-5 text-contrast" />
							</button>

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
							<ButtonStyled
								:outline="issue.status !== 'safe'"
								:color="issue.status === 'safe' ? 'green' : undefined"
							>
								<button @click="updateIssueStatus(issue.issue_id, 'safe')">Safe</button>
							</ButtonStyled>

							<ButtonStyled
								:outline="issue.status !== 'unsafe'"
								:color="issue.status === 'unsafe' ? 'red' : undefined"
							>
								<button @click="updateIssueStatus(issue.issue_id, 'unsafe')">Malware</button>
							</ButtonStyled>
						</div>
					</div>

					<div v-if="expandedIssues.has(issue.issue_id)" class="flex flex-col gap-4 px-4 pb-4">
						<div
							v-for="(detail, detailIdx) in issue.details"
							:key="detailIdx"
							class="flex flex-col gap-3"
						>
							<div class="flex items-center gap-4">
								<p class="font-mono text-sm text-secondary">{{ detail.class_name }}</p>
							</div>

							<div
								v-if="detail.decompiled_source"
								class="flex gap-3 overflow-x-auto rounded-2xl bg-surface-3 p-3 font-mono text-sm"
							>
								<div class="flex flex-col border-r border-surface-5 pr-3 text-right text-secondary">
									<span v-for="n in detail.decompiled_source.split('\n').length" :key="n">{{
										n
									}}</span>
								</div>

								<pre class="flex-1 text-secondary"><code>{{ detail.decompiled_source }}</code></pre>
							</div>
						</div>
					</div>
				</div>
			</div>
		</div>
	</div>
</template>
