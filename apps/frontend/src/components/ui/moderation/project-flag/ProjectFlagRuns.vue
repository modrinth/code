<template>
	<div class="flex min-w-0 flex-col gap-4">
		<p v-if="activityError" role="alert" class="m-0 text-red">
			Could not load scan activity: {{ errorMessage(activityError) }}
		</p>
		<p v-if="startError" role="alert" class="m-0 text-red">
			Could not start the run: {{ errorMessage(startError) }}
		</p>
		<div v-if="isPending" role="status" class="universal-card flex items-center gap-2">
			<LoaderCircleIcon class="animate-spin" /> Loading project flag runs…
		</div>
		<div v-if="isError" role="alert" class="universal-card flex flex-col items-start gap-3">
			<p class="m-0 text-red">Could not load runs: {{ errorMessage(error) }}</p>
			<Button :disabled="isFetching" @click="refetch()">Try again</Button>
		</div>
		<Table
			v-if="data"
			:columns="columns"
			:data="runs"
			row-key="id"
			row-clickable
			row-class="cursor-pointer"
			table-layout="auto"
			table-min-width="60rem"
			@row-click="openRun"
		>
			<template #cell-id="{ row }"
				><code class="text-xs">{{ row.id }}</code></template
			>
			<template #cell-started_at="{ row }">{{ formatDate(row.started_at) }}</template>
			<template #cell-started_by="{ row }">
				<NuxtLink
					v-if="row.started_by"
					data-no-row-click
					:to="`/user/${row.started_by.id}`"
					class="text-link"
					>{{ row.started_by.username }}</NuxtLink
				>
				<span v-else class="text-secondary">Unknown</span>
			</template>
			<template #cell-status="{ row }">
				<span class="inline-flex items-center gap-2"
					><LoaderCircleIcon v-if="row.status !== 'done'" class="size-4 animate-spin" />{{
						formatStatus(row.status)
					}}</span
				>
			</template>
			<template #cell-applied_at="{ row }">
				<div class="flex flex-col gap-1">
					<span>{{ formatDate(row.applied_at) }}</span>
					<NuxtLink
						v-if="row.applied_by"
						data-no-row-click
						:to="`/user/${row.applied_by.id}`"
						class="text-sm text-link"
						>{{ row.applied_by.username }}</NuxtLink
					>
				</div>
			</template>
			<template #cell-view="{ row }">
				<div class="flex items-center gap-2" data-no-row-click>
					<Button :aria-label="`View run ${row.id}`" @click="openRun(row)">View</Button>
					<IconButton
						type="quiet"
						color="red"
						:circular="false"
						:disabled="isDeleting"
						:label="`Delete run ${row.id}`"
						@click="confirmDelete(row)"
					>
						<TrashIcon />
					</IconButton>
				</div>
			</template>
			<template #empty-state
				><p class="p-6 text-center text-secondary">
					{{ offset === 0 ? 'No project flag runs yet.' : 'No runs on this page.' }}
				</p></template
			>
		</Table>
		<div class="flex flex-wrap items-center justify-between gap-3">
			<p role="status" class="m-0 text-sm text-secondary">
				<template v-if="runs.length">Showing {{ offset + 1 }}–{{ offset + runs.length }}</template>
				<template v-else>Page {{ offset / PAGE_SIZE + 1 }}</template>
			</p>
			<div class="flex gap-2">
				<Button :disabled="offset === 0 || isFetching" @click="offset -= PAGE_SIZE"
					>Previous</Button
				>
				<Button :disabled="!hasNextPage || isFetching" @click="offset += PAGE_SIZE">Next</Button>
			</div>
		</div>

		<NewModal
			ref="deleteModal"
			header="Delete run"
			:disable-close="isDeleting && runToDelete !== null"
			:on-hide="closeDelete"
		>
			<div v-if="runToDelete" class="flex w-[32rem] max-w-full flex-col gap-4">
				<p class="m-0 break-words">
					Delete run <span class="font-mono">{{ runToDelete.id }}</span
					>?
				</p>
				<p class="m-0 text-secondary">
					This permanently deletes the run, its results, and its inactive project flags. Active
					flags are preserved and detached from the run. This cannot be undone.
				</p>
				<p v-if="deleteError" role="alert" class="m-0 text-red">
					Could not delete the run: {{ errorMessage(deleteError) }}
				</p>
				<div class="flex justify-end gap-2">
					<Button :disabled="isDeleting" @click="deleteModal?.hide()">Cancel</Button>
					<Button
						type="colored"
						color="red"
						:disabled="isDeleting"
						@click="deleteRun(runToDelete.id)"
					>
						<LoaderCircleIcon v-if="isDeleting" class="animate-spin" />
						<TrashIcon v-else />
						{{ isDeleting ? 'Deleting…' : 'Delete run' }}
					</Button>
				</div>
			</div>
		</NewModal>

		<NewModal ref="runModal" header="Project flag run" :on-hide="closeRun">
			<div v-if="selectedRun" class="flex w-[64rem] max-w-full flex-col gap-4">
				<dl class="m-0 grid grid-cols-[auto_minmax(0,1fr)] gap-x-4 gap-y-2 text-sm">
					<dt class="font-semibold text-contrast">Run ID</dt>
					<dd class="m-0 break-all font-mono">{{ selectedRun.id }}</dd>
					<dt class="font-semibold text-contrast">Status</dt>
					<dd class="m-0">{{ formatStatus(selectedRun.status) }}</dd>
					<dt class="font-semibold text-contrast">Started by</dt>
					<dd class="m-0">
						<NuxtLink
							v-if="selectedRun.started_by"
							:to="`/user/${selectedRun.started_by.id}`"
							class="text-link"
							>{{ selectedRun.started_by.username }}</NuxtLink
						>
						<span v-else>Unknown</span>
					</dd>
					<dt class="font-semibold text-contrast">Started</dt>
					<dd class="m-0">{{ formatDate(selectedRun.started_at) }}</dd>
					<dt class="font-semibold text-contrast">Completed</dt>
					<dd class="m-0">{{ formatDate(selectedRun.completed_at) }}</dd>
					<dt class="font-semibold text-contrast">Applied</dt>
					<dd class="m-0">{{ formatDate(selectedRun.applied_at) }}</dd>
					<dt class="font-semibold text-contrast">Applied by</dt>
					<dd class="m-0">
						<NuxtLink
							v-if="selectedRun.applied_by"
							:to="`/user/${selectedRun.applied_by.id}`"
							class="text-link"
							>{{ selectedRun.applied_by.username }}</NuxtLink
						>
						<span v-else>—</span>
					</dd>
					<dt class="font-semibold text-contrast">Projects</dt>
					<dd class="m-0">{{ selectedRun.total_projects }}</dd>
					<dt class="font-semibold text-contrast">Errors</dt>
					<dd class="m-0">{{ selectedRun.num_errors }}</dd>
				</dl>
				<p
					v-if="selectedRun.error !== null"
					role="alert"
					class="m-0 whitespace-pre-wrap break-words text-red"
				>
					Run error: {{ selectedRun.error }}
				</p>
				<details v-if="selectedRun.output !== null">
					<summary class="cursor-pointer font-semibold text-contrast">Run output</summary>
					<pre class="max-h-64 overflow-auto whitespace-pre-wrap break-words text-sm">{{
						json(selectedRun.output)
					}}</pre>
				</details>
				<h2 class="m-0 text-lg font-semibold text-contrast">Project results</h2>
				<p v-if="resultsPending" role="status" class="m-0 flex items-center gap-2">
					<LoaderCircleIcon class="animate-spin" /> Loading project results…
				</p>
				<div v-if="resultsError" role="alert" class="flex flex-col items-start gap-3">
					<p class="m-0 text-red">
						Could not load project results: {{ errorMessage(resultsErrorValue) }}
					</p>
					<Button :disabled="resultsFetching" @click="refetchResults()">Try again</Button>
				</div>
				<div v-if="resultsData" class="flex flex-col gap-3">
					<details
						v-for="row in resultRows"
						:key="`${selectedRun.id}:${row.projectId}`"
						class="group rounded-xl border border-solid border-surface-5 bg-surface-2"
					>
						<summary
							class="flex cursor-pointer list-none items-center gap-3 p-4 [&::-webkit-details-marker]:hidden"
						>
							<ChevronRightIcon
								aria-hidden="true"
								class="size-5 shrink-0 transition-transform group-open:rotate-90"
							/>
							<div class="flex min-w-0 items-center gap-2">
								<img
									v-if="row.project.icon_url"
									:src="row.project.icon_url"
									alt=""
									class="size-10 rounded-lg"
								/>
								<div class="min-w-0">
									<p class="m-0 break-words font-semibold text-contrast">{{ row.project.title }}</p>
									<p class="m-0 break-all font-mono text-xs">{{ row.project.id }}</p>
								</div>
							</div>
							<span class="ml-auto text-sm text-secondary"
								>{{ row.flags.length }} {{ row.flags.length === 1 ? 'flag' : 'flags' }}</span
							>
							<span v-if="row.error !== null" class="text-sm text-red">Error</span>
						</summary>
						<div
							class="flex min-w-0 flex-col gap-4 border-0 border-t border-solid border-surface-5 p-4"
						>
							<div class="flex flex-wrap items-center justify-between gap-2 text-sm">
								<NuxtLink
									:to="`/project/${encodeURIComponent(row.project.slug ?? row.project.id)}`"
									class="text-link"
									>View project</NuxtLink
								>
								<span class="text-secondary">Evaluated {{ formatDate(row.evaluated_at) }}</span>
							</div>
							<p v-if="row.flags.length === 0" class="m-0 text-secondary">No flags found.</p>
							<article
								v-for="entry in row.flags"
								:key="`${entry.ruleId}:${entry.index}`"
								class="flex min-w-0 flex-col gap-2 rounded-lg bg-surface-3 p-3"
							>
								<div class="flex flex-wrap items-center justify-between gap-2">
									<h3 class="m-0 break-all text-base font-semibold text-contrast">
										{{ entry.flag.key }}
									</h3>
									<span class="break-all text-sm text-secondary">{{ entry.ruleName }}</span>
								</div>
								<div
									class="max-h-80 overflow-auto whitespace-pre-wrap break-words font-mono text-sm"
									v-text="json(entry.flag)"
								/>
							</article>
							<p v-if="row.error !== null" class="m-0 whitespace-pre-wrap break-words text-red">
								Project error: {{ row.error }}
							</p>
							<p v-if="Object.keys(row.rules).length === 0" class="m-0 text-secondary">
								No rule results.
							</p>
							<details v-if="row.diagnostics.length">
								<summary class="cursor-pointer font-semibold text-contrast">
									Rule logs and errors
								</summary>
								<div class="flex flex-col gap-3 pt-3">
									<section
										v-for="{ rule, ruleId } in row.diagnostics"
										:key="ruleId"
										class="flex min-w-0 flex-col gap-3 rounded-lg bg-surface-3 p-4"
									>
										<h3 class="m-0 break-all text-base font-semibold text-contrast">
											{{ rule.name ?? 'Deleted rule' }}
										</h3>
										<p
											v-if="rule.error !== null"
											class="m-0 whitespace-pre-wrap break-words text-red"
										>
											Rule error: {{ rule.error }}
										</p>

										<div class="flex min-w-0 flex-col gap-2">
											<h4 class="m-0 text-sm font-semibold text-contrast">Logs</h4>
											<div
												v-if="rule.log.length"
												class="m-0 max-h-80 overflow-auto whitespace-pre-wrap break-words rounded-lg bg-surface-2 p-3 font-mono text-sm"
												v-text="rule.log.join('\n')"
											/>
											<p v-else class="m-0 text-sm text-secondary">No logs.</p>
										</div>
									</section>
								</div>
							</details>
						</div>
					</details>
					<p v-if="resultRows.length === 0" class="p-6 text-center text-secondary">
						{{
							selectedRun.status !== 'done'
								? 'No project results on this page yet. Results refresh automatically.'
								: 'No project results on this page.'
						}}
					</p>
				</div>
				<div class="flex flex-wrap items-center justify-between gap-3">
					<p role="status" class="m-0 text-sm text-secondary">
						<template v-if="resultsData">{{
							resultRows.length
								? `Showing ${resultsOffset + 1}–${resultsOffset + resultRows.length} of ${resultsData.total}`
								: `${resultsData.total} results`
						}}</template>
						<template v-else>Page {{ resultsOffset / PAGE_SIZE + 1 }}</template>
					</p>
					<div class="flex gap-2">
						<Button
							:disabled="resultsOffset === 0 || resultsFetching"
							@click="resultsOffset -= PAGE_SIZE"
							>Previous</Button
						>
						<Button
							:disabled="!hasNextResults || resultsFetching"
							@click="resultsOffset += PAGE_SIZE"
							>Next</Button
						>
					</div>
				</div>
			</div>
		</NewModal>
	</div>
</template>

<script setup lang="ts">
import { ChevronRightIcon, LoaderCircleIcon, TrashIcon } from '@modrinth/assets'
import { Button, IconButton, injectModrinthClient, NewModal, Table } from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'

const api = injectModrinthClient().labrinth.project_flags_internal
type Run = Awaited<ReturnType<typeof api.listRuns>>[number]
const PAGE_SIZE = 25
const REFRESH_INTERVAL = 10_000
const offset = ref(0)
const { data: activity, error: activityError } = useQuery({
	queryKey: ['moderation', 'project-flag', 'runs', 'activity'],
	queryFn: () => api.getRunActivity(),
	refetchInterval: REFRESH_INTERVAL,
})
const numScanning = computed(() => activity.value?.num_scanning ?? 0)
const numConsumers = computed(() => activity.value?.num_consumers ?? 0)
const queryClient = useQueryClient()
const {
	mutate: startRun,
	isPending: isStarting,
	error: startError,
} = useMutation({
	mutationFn: () => api.startRun(),
	retry: false,
	onSuccess: async () => {
		offset.value = 0
		await queryClient.invalidateQueries({ queryKey: ['moderation', 'project-flag', 'runs'] })
	},
})
const selectedSnapshot = ref<Run | null>(null)
const resultsOffset = ref(0)
const runModal = useTemplateRef<InstanceType<typeof NewModal>>('runModal')
const deleteModal = useTemplateRef<InstanceType<typeof NewModal>>('deleteModal')
const runToDelete = ref<Run | null>(null)
const {
	mutate: deleteRun,
	isPending: isDeleting,
	error: deleteError,
	reset: resetDelete,
} = useMutation({
	mutationFn: (id: string) => api.deleteRun(id),
	retry: false,
	onSuccess: async (_, id) => {
		if (selectedSnapshot.value?.id === id) {
			runModal.value?.hide()
			closeRun()
		}
		runToDelete.value = null
		await nextTick()
		deleteModal.value?.hide()
		if (runs.value.length === 1 && offset.value > 0) offset.value -= PAGE_SIZE
		queryClient.removeQueries({ queryKey: ['moderation', 'project-flag', 'runs', id] })
		await queryClient.invalidateQueries({ queryKey: ['moderation', 'project-flag'] })
	},
})
const { data, isPending, isError, error, isFetching, refetch } = useQuery({
	queryKey: computed(() => [
		'moderation',
		'project-flag',
		'runs',
		{ limit: PAGE_SIZE + 1, offset: offset.value },
	]),
	queryFn: () => api.listRuns({ limit: PAGE_SIZE + 1, offset: offset.value }),
	refetchInterval: (query) =>
		query.state.data?.some((run) => run.status !== 'done') ? REFRESH_INTERVAL : false,
})
const runs = computed(() => data.value?.slice(0, PAGE_SIZE) ?? [])
const hasNextPage = computed(() => (data.value?.length ?? 0) > PAGE_SIZE)
const selectedRun = computed(
	() => data.value?.find((run) => run.id === selectedSnapshot.value?.id) ?? selectedSnapshot.value,
)
const {
	data: resultsData,
	isPending: resultsPending,
	isError: resultsError,
	error: resultsErrorValue,
	isFetching: resultsFetching,
	refetch: refetchResults,
} = useQuery({
	queryKey: computed(() => [
		'moderation',
		'project-flag',
		'runs',
		selectedRun.value?.id,
		'results',
		{ limit: PAGE_SIZE, offset: resultsOffset.value },
	]),
	queryFn: () => {
		if (!selectedRun.value) throw new Error('No run selected.')
		return api.getRunResults(selectedRun.value.id, {
			limit: PAGE_SIZE,
			offset: resultsOffset.value,
		})
	},
	enabled: computed(() => selectedRun.value !== null),
	refetchInterval: () =>
		selectedRun.value && selectedRun.value.status !== 'done' ? REFRESH_INTERVAL : false,
})
const resultRows = computed(
	() =>
		resultsData.value?.results.map((result) => ({
			...result,
			projectId: result.project.id,
			flags: Object.entries(result.rules).flatMap(([ruleId, rule]) =>
				rule.flags.map((flag, index) => ({
					ruleId,
					ruleName: rule.name ?? 'Deleted rule',
					flag,
					index,
				})),
			),
			diagnostics: Object.entries(result.rules)
				.filter(([, rule]) => rule.error !== null || rule.log.length > 0)
				.map(([ruleId, rule]) => ({ ruleId, rule })),
		})) ?? [],
)
const hasNextResults = computed(
	() => resultsOffset.value + PAGE_SIZE < (resultsData.value?.total ?? 0),
)
const columns = [
	{ key: 'id', label: 'Run ID' },
	{ key: 'started_at', label: 'Started' },
	{ key: 'started_by', label: 'Started by' },
	{ key: 'status', label: 'Status' },
	{ key: 'total_projects', label: 'Projects' },
	{ key: 'num_errors', label: 'Errors' },
	{ key: 'applied_at', label: 'Applied' },
	{ key: 'view', label: 'Actions', cellClass: 'whitespace-nowrap' },
]

watch(
	() => selectedRun.value?.status,
	(status, previousStatus) => {
		if (status === 'done' && previousStatus && previousStatus !== 'done') refetchResults()
	},
)

defineExpose({ startRun, isStarting, numScanning, numConsumers })

function confirmDelete(run: Run) {
	resetDelete()
	runToDelete.value = run
	deleteModal.value?.show()
}

function closeDelete() {
	runToDelete.value = null
	resetDelete()
}

function openRun(run: Run) {
	selectedSnapshot.value = run
	resultsOffset.value = 0
	runModal.value?.show()
}

function closeRun() {
	selectedSnapshot.value = null
	resultsOffset.value = 0
}

function formatStatus(status: Run['status']) {
	return status === 'done' ? 'Done' : 'Running'
}

function formatDate(value: string | null) {
	return value === null
		? '—'
		: new Date(value).toLocaleString('en-US', { timeZone: 'UTC', timeZoneName: 'short' })
}

function json(value: unknown) {
	return JSON.stringify(value, null, 2)
}

function errorMessage(value: unknown) {
	return value instanceof Error ? value.message : 'Please try again.'
}
</script>
