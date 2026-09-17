<template>
	<div class="flex min-w-0 flex-col gap-3">
		<p v-if="!selection" class="m-0 text-secondary">
			{{ formatMessage(messages.empty) }}
		</p>
		<div v-else-if="error || reportsQuery.isError.value" role="alert">
			<p class="m-0">{{ formatMessage(messages.loadError) }}</p>
			<Button @click="retry">{{ formatMessage(messages.retry) }}</Button>
		</div>
		<p v-else-if="isLoading || reportsQuery.isPending.value" role="status" class="m-0">
			{{ formatMessage(messages.loading) }}
		</p>
		<p v-else-if="!reports.length" class="m-0 text-secondary">
			{{ formatMessage(messages.noReports) }}
		</p>
		<template v-else-if="project">
			<p class="m-0 text-sm text-secondary">
				{{ formatMessage(messages.reviewProgress) }}
			</p>
			<template v-if="selectedFile">
				<div class="flex flex-wrap items-center justify-between gap-2">
					<Button @click="selectedFileId = null"
						><LeftArrowIcon />{{ formatMessage(messages.backToFiles) }}</Button
					>
					<TechRevFileActions :file="selectedFile" />
				</div>
				<h3 class="m-0 break-all text-lg font-semibold text-contrast">
					{{ selectedFile.file_name }}
				</h3>
				<div v-if="sourceQueries.some((query) => query.isError)" role="alert">
					<p class="m-0">{{ formatMessage(messages.sourceError) }}</p>
					<Button @click="retrySources">{{ formatMessage(messages.retry) }}</Button>
				</div>
				<TechRevFileDetailTab
					:key="selectedFile.id"
					:file="selectedFile"
					:loading-issues="loadingIssues"
					:decompiled-sources="decompiledSources"
					@load-issue-sources="loadSources"
					@refetch="refreshReports"
				/>
			</template>
			<template v-else>
				<h3 class="m-0 text-lg font-semibold text-contrast">
					{{
						formatMessage(messages.pendingFiles, {
							count: pendingReports.length,
						})
					}}
				</h3>
				<TechRevFilesTab
					v-if="pendingReports.length"
					:reports="pendingReports"
					:project="project"
					@view-flags="selectFile"
				/>
				<p v-else class="m-0 text-secondary">
					{{ formatMessage(messages.noPendingFiles) }}
				</p>
				<details v-if="completedReports.length">
					<summary class="cursor-pointer py-3 font-semibold text-contrast">
						{{
							formatMessage(messages.completedFiles, {
								count: completedReports.length,
							})
						}}
					</summary>
					<TechRevFilesTab
						:reports="completedReports"
						:project="project"
						@view-flags="selectFile"
					/>
				</details>
				<details v-if="unflaggedReports.length">
					<summary class="cursor-pointer py-3 font-semibold text-contrast">
						{{
							formatMessage(messages.noFlagsFiles, {
								count: unflaggedReports.length,
							})
						}}
					</summary>
					<TechRevFilesTab
						:reports="unflaggedReports"
						:project="project"
						@view-flags="selectFile"
					/>
				</details>
			</template>
		</template>
	</div>
</template>

<script setup lang="ts">
import { LeftArrowIcon } from '@modrinth/assets'
import { Button, injectModrinthClient, useVIntl } from '@modrinth/ui'
import { useQueries } from '@tanstack/vue-query'
import { computed, provide, ref } from 'vue'

import TechRevFileActions from '~/components/ui/moderation/tech-review/TechRevFileActions.vue'
import TechRevFileDetailTab from '~/components/ui/moderation/tech-review/TechRevFileDetailTab.vue'
import TechRevFilesTab from '~/components/ui/moderation/tech-review/TechRevFilesTab.vue'
import type { FlattenedFileReport } from '~/components/ui/moderation/tech-review/types'
import { TECH_REVIEW_DECISIONS_KEY } from '~/components/ui/moderation/tech-review/use-tech-review-decisions'
import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages as messages } from '../messages'

const { formatMessage } = useVIntl()
const client = injectModrinthClient()
const {
	selection,
	project,
	isLoading,
	error,
	refresh,
	reportsQuery,
	reports,
	decisions,
	pendingReports,
	completedReports,
	unflaggedReports,
	refreshReports,
} = injectProjectReviewPageContext()
provide(TECH_REVIEW_DECISIONS_KEY, decisions)
const selectedFileId = ref<string | null>(null)
const selectedFile = computed(() => reports.value.find((file) => file.id === selectedFileId.value))
const requestedIssues = ref(new Set<string>())
const issueIds = computed(() => [...requestedIssues.value])
const sourceQueries = useQueries({
	queries: computed(() =>
		issueIds.value.map((id) => ({
			queryKey: ['tech-review', 'issue', id, { includeHidden: true }],
			queryFn: () =>
				client.labrinth.tech_review_internal.getIssue(id, {
					include_hidden: true,
				}),
			staleTime: 24 * 60 * 60 * 1000,
		})),
	),
})
const loadingIssues = computed(
	() => new Set(issueIds.value.filter((_id, index) => sourceQueries.value[index]?.isFetching)),
)
const decompiledSources = computed(
	() =>
		new Map(
			sourceQueries.value.flatMap((query) =>
				(query.data?.details ?? []).flatMap((detail) =>
					detail.decompiled_source ? [[detail.id, detail.decompiled_source] as const] : [],
				),
			),
		),
)
function loadSources(ids: string[]) {
	for (const id of ids) requestedIssues.value.add(id)
}
function selectFile(file: FlattenedFileReport) {
	selectedFileId.value = file.id
}
async function retrySources() {
	await Promise.all(
		sourceQueries.value.filter((query) => query.isError).map((query) => query.refetch()),
	)
}
async function retry() {
	await Promise.all([refresh(), reportsQuery.refetch()])
}
</script>
