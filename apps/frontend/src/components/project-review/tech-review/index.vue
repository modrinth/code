<template>
	<div
		class="review-scrollbars flex h-full min-h-0 min-w-0 flex-col gap-3"
		:class="{ 'overflow-auto': !selectedFile }"
	>
		<p v-if="!selection" class="m-0 text-secondary">No project selected</p>
		<div v-else-if="error || reportsQuery.isError.value" role="alert">
			<p class="m-0">Could not load project data</p>
			<Button @click="retry">Retry</Button>
		</div>
		<p v-else-if="isLoading || reportsQuery.isPending.value" role="status" class="m-0">
			Loading...
		</p>
		<p v-else-if="!reports.length" class="m-0 text-secondary">
			No technical-review reports are available for this project
		</p>
		<FileDetail
			v-else-if="project && selectedFile"
			:key="selectedFile.id"
			:file="selectedFile"
			file-tree
			:focused-detail-id="focusedDetailId"
			:loading-issues="loadingIssues"
			:settled-issues="settledIssues"
			:decompiled-sources="decompiledSources"
			@refetch="refreshReports"
		>
			<template #header="{ verdictProps }">
				<div
					class="mb-2.5 flex shrink-0 items-center gap-2 border-0 border-b border-solid border-surface-3 pb-2.5"
				>
					<div class="group/file-actions flex min-w-0 items-center gap-1">
						<h3
							class="m-0 min-w-0 truncate pl-1 text-base font-semibold text-contrast"
							:title="selectedFile.file_name"
						>
							{{ selectedFile.file_name }}
						</h3>
						<FileActions :file="selectedFile" />
					</div>
					<div
						class="ml-auto flex shrink-0 items-center gap-0.5"
						role="group"
						aria-label="Archive navigation"
					>
						<Button
							v-tooltip="'Previous archive'"
							type="quiet"
							size="sm"
							icon-only
							aria-label="Previous archive"
							:disabled="!previousFile"
							@click="previousFile && selectFile(previousFile)"
						>
							<ChevronLeftIcon aria-hidden="true" />
						</Button>
						<FloatingMenu ref="archiveMenu" placement="bottom-end" :arrow="false" bare>
							<Button
								type="quiet"
								size="sm"
								class="tabular-nums"
								:aria-label="`Choose archive (${selectedFileIndex + 1}/${reports.length})`"
							>
								{{ selectedFileIndex + 1 }}/{{ reports.length }}
							</Button>
							<template #popper>
								<section
									aria-label="Archives"
									class="review-scrollbars w-[30rem] max-w-[calc(100vw-2rem)] rounded-xl border border-solid border-surface-5 bg-surface-2 p-1.5 shadow-2xl"
								>
									<h4 class="m-0 px-3 pb-2 pt-1.5 text-xs font-medium text-secondary">
										Archives ({{ reports.length }})
									</h4>
									<div class="flex max-h-[60vh] flex-col gap-1 overflow-y-auto">
										<FileItem
											v-for="file in reports"
											:key="file.id"
											:file="file"
											:selected="file.id === selectedFile.id"
											@view-flags="selectFile"
										/>
									</div>
								</section>
							</template>
						</FloatingMenu>
						<Button
							v-tooltip="'Next archive'"
							type="quiet"
							size="sm"
							icon-only
							aria-label="Next archive"
							:disabled="!nextFile"
							@click="nextFile && selectFile(nextFile)"
						>
							<ChevronRightIcon aria-hidden="true" />
						</Button>
					</div>
					<VerdictButtons
						v-if="verdictProps.remainingCount > 0"
						class="shrink-0"
						v-bind="verdictProps"
					/>
				</div>
				<div v-if="sourceQueries.some((query) => query.isError)" role="alert">
					<p class="m-0">Could not load source code for some files.</p>
					<Button @click="retrySources">Retry</Button>
				</div>
			</template>
		</FileDetail>
	</div>
</template>

<script setup lang="ts">
import { ChevronLeftIcon, ChevronRightIcon } from '@modrinth/assets'
import { Button, FloatingMenu, injectModrinthClient } from '@modrinth/ui'
import { type QueryFunctionContext, useQueries } from '@tanstack/vue-query'
import { computed, nextTick, provide, ref, watch } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { injectProjectReviewContext } from '../layout/context'
import FileDetail from './file-detail/index.vue'
import FileActions from './files/actions.vue'
import FileItem from './files/item.vue'
import { createSourceRequestQueue } from './source-request-queue'
import type { FlattenedFileReport } from './types'
import VerdictButtons from './verdict/buttons.vue'
import { TECH_REVIEW_DECISIONS_KEY } from './verdict/use-decisions'

const client = injectModrinthClient()
const route = useRoute()
const { openTab } = injectProjectReviewContext()
const focusedDetailId = computed(() =>
	typeof route.query.detail === 'string' ? route.query.detail : null,
)
const {
	selection,
	project,
	isLoading,
	error,
	refresh,
	reportsQuery,
	reports,
	decisions,
	refreshReports,
} = injectProjectReviewPageContext()
provide(TECH_REVIEW_DECISIONS_KEY, decisions)
const archiveMenu = ref<InstanceType<typeof FloatingMenu>>()
const selectedFileId = ref<string>()
const selectedFile = computed(
	() => reports.value.find((file) => file.id === selectedFileId.value) ?? reports.value[0],
)
const selectedFileIndex = computed(() =>
	reports.value.findIndex((file) => file.id === selectedFile.value?.id),
)
const previousFile = computed(() => reports.value[selectedFileIndex.value - 1])
const nextFile = computed(() => reports.value[selectedFileIndex.value + 1])
let detailLinkHandled = false
watch(
	[focusedDetailId, reports],
	async ([detailId, files], [previousDetailId]) => {
		if (detailId !== previousDetailId) detailLinkHandled = false
		if (!detailId || detailLinkHandled) return
		const file = files.find((file) =>
			file.issues.some((issue) => issue.details.some((detail) => detail.id === detailId)),
		)
		if (!file) return
		detailLinkHandled = true
		selectedFileId.value = file.id
		await nextTick()
		openTab('tech-review')
	},
	{ immediate: true },
)
const requestSource = createSourceRequestQueue()
const issueIds = computed(() => [
	...new Set(selectedFile.value?.issues.map((issue) => issue.id) ?? []),
])
const sourceQueries = useQueries({
	queries: computed(() =>
		issueIds.value.map((id) => ({
			queryKey: ['tech-review', 'issue', id, { includeHidden: true }] as const,
			queryFn: ({ signal }: QueryFunctionContext) =>
				requestSource(
					() => client.labrinth.tech_review_internal.getIssue(id, { include_hidden: true }),
					signal,
				),
			staleTime: 24 * 60 * 60 * 1000,
		})),
	),
})
const loadingIssues = computed(
	() => new Set(issueIds.value.filter((_id, index) => sourceQueries.value[index]?.isFetching)),
)
const settledIssues = computed(
	() =>
		new Set(
			issueIds.value.filter(
				(_id, index) =>
					sourceQueries.value[index]?.isSuccess || sourceQueries.value[index]?.isError,
			),
		),
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
function selectFile(file: FlattenedFileReport) {
	archiveMenu.value?.hide()
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

<style scoped>
@property --review-scrollbar-color {
	syntax: '<color>';
	inherits: true;
	initial-value: transparent;
}

.review-scrollbars {
	--review-scrollbar-color: transparent;
	--overlay-scrollbar-color: var(--review-scrollbar-color);
	transition: --review-scrollbar-color 1750ms ease;
}

.review-scrollbars:hover {
	--review-scrollbar-color: var(--color-scrollbar);
	transition-duration: 200ms;
}

.review-scrollbars,
.review-scrollbars :deep(*) {
	scrollbar-color: var(--review-scrollbar-color) transparent;
}

.review-scrollbars::-webkit-scrollbar-thumb,
.review-scrollbars :deep(*::-webkit-scrollbar-thumb) {
	background: var(--review-scrollbar-color);
}

@media (hover: none) {
	.review-scrollbars {
		--review-scrollbar-color: var(--color-scrollbar);
	}
}

@media (prefers-reduced-motion: reduce) {
	.review-scrollbars {
		transition: none;
	}
}
</style>
