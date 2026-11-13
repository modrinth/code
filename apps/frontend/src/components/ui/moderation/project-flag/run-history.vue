<template>
	<div class="flex min-w-0 flex-col gap-4">
		<p v-if="activityError" role="alert" class="m-0 text-red">
			Could not load scan activity: {{ errorMessage(activityError) }}
		</p>
		<p v-if="startError" role="alert" class="m-0 text-red">
			Could not start the run: {{ errorMessage(startError) }}
		</p>
		<ProjectFlagRunActions v-slot="{ runOptions, busy }">
			<div class="flex flex-wrap items-center gap-2">
				<Input
					v-model="search"
					:icon="SearchIcon"
					type="search"
					aria-label="Search flag rule runs"
					placeholder="Search runs by ID, user, or status…"
					autocomplete="off"
					wrapper-class="min-w-0 grow basis-full sm:max-w-md sm:basis-48"
					clearable
				/>
				<TimeFramePicker
					v-model:mode="timeframe.mode"
					v-model:preset="timeframe.preset"
					v-model:last-amount="timeframe.lastAmount"
					v-model:last-unit="timeframe.lastUnit"
					v-model:custom-start-date="timeframe.customStartDate"
					v-model:custom-end-date="timeframe.customEndDate"
					:now-timestamp="now.getTime()"
					class="min-w-0 max-w-[14rem] flex-1 sm:flex-none sm:basis-56"
				>
					<template #prefix>
						<CalendarIcon class="size-4" aria-hidden="true" />
					</template>
				</TimeFramePicker>
				<Button
					type="colored"
					color="brand"
					:disabled="isStarting"
					class="ml-auto max-w-full shrink-0"
					@click="startRun()"
				>
					<LoaderCircleIcon v-if="isStarting" class="animate-spin" />
					<PlayIcon v-else />
					{{ isStarting ? 'Starting run…' : 'Start run' }}
				</Button>
			</div>
			<div v-if="isPending" role="status" class="universal-card flex items-center gap-2">
				<LoaderCircleIcon class="animate-spin" /> Loading project flag runs…
			</div>
			<div v-if="isError" role="alert" class="universal-card flex flex-col items-start gap-3">
				<p class="m-0 text-red">Could not load runs: {{ errorMessage(error) }}</p>
				<Button :disabled="isFetching" @click="refetch()">Try again</Button>
			</div>
			<ul v-if="runs.length" class="m-0 flex list-none flex-col gap-3 p-0">
				<li
					v-for="(run, index) in runs"
					:key="run.id"
					class="shadow-card flex min-w-0 cursor-pointer flex-wrap items-center gap-x-4 gap-y-2 rounded-2xl border border-solid border-surface-4 bg-surface-3 p-4 transition-[border-color,filter] hover:brightness-125"
					@click="navigateTo(runPath(run.id))"
				>
					<div class="flex flex-wrap items-center gap-2">
						<NuxtLink
							:to="runPath(run.id)"
							:aria-label="`View run from ${formatDate(run.started_at, true)}`"
							class="cursor-pointer rounded-md border-0 bg-transparent p-0 text-left text-contrast focus-visible:outline focus-visible:outline-2 focus-visible:outline-brand"
							@click.stop
						>
							<time
								v-tooltip="formatDate(run.started_at, true)"
								:datetime="run.started_at"
								class="text-lg font-semibold"
							>
								{{ formatDate(run.started_at) }}
							</time>
						</NuxtLink>
						<ProjectFlagScanBadge
							:run="run"
							:scanned="resultCounts[index]?.data?.scanned"
							:num-consumers="activity?.num_consumers"
						/>
					</div>

					<div class="flex flex-wrap items-center gap-2 sm:ml-auto">
						<TagItem
							v-if="(resultCounts[index]?.data?.errors ?? run.num_errors) > 0"
							v-tooltip="'Scan errors'"
							style="--_color: var(--color-red)"
						>
							<XCircleIcon aria-hidden="true" /><span class="sr-only">Scan errors: </span>
							{{ (resultCounts[index]?.data?.errors ?? run.num_errors).toLocaleString('en-US') }}
						</TagItem>
						<TagItem
							style="--_color: var(--color-purple)"
							v-tooltip="resultCounts[index]?.data?.flagTooltip ?? 'Flagged count unavailable'"
						>
							<TagCategoryFlagIcon aria-hidden="true" /><span class="sr-only">Flagged: </span>
							{{ resultCounts[index]?.data?.flagged.toLocaleString('en-US') ?? '—' }}
						</TagItem>
						<TagItem
							style="--_color: var(--color-green)"
							v-tooltip="
								resultCounts[index]?.isError ? 'Approvable count unavailable' : 'Approvable'
							"
						>
							<BadgeCheckIcon aria-hidden="true" /><span class="sr-only">Approvable: </span>
							{{ resultCounts[index]?.data?.approvable.toLocaleString('en-US') ?? '—' }}
						</TagItem>
						<div class="shrink-0" @click.stop>
							<TeleportOverflowMenu
								type="quiet"
								:disabled="busy"
								:label="`Options for run ${run.id}`"
								tooltip="Run options"
								:options="runOptions(run, resultCounts[index]?.data?.results)"
							>
								<MoreHorizontalIcon aria-hidden="true" />
							</TeleportOverflowMenu>
						</div>
					</div>

					<div
						class="flex w-full min-w-0 flex-wrap items-center gap-x-2 gap-y-1 text-sm text-secondary"
					>
						<span>Started by</span>
						<NuxtLink
							v-if="run.started_by"
							:to="`/user/${run.started_by.id}`"
							class="inline-flex min-w-0 items-center gap-2 font-medium text-contrast hover:underline"
							@click.stop
						>
							<Avatar :src="starterAvatars.get(run.started_by.id)" size="1.5rem" circle no-shadow />
							<span class="break-all">{{ run.started_by.username }}</span>
						</NuxtLink>
						<span v-else>unknown user</span>
					</div>
				</li>
			</ul>
			<p v-else-if="data" class="universal-card m-0 py-6 text-center text-secondary">
				{{
					data.length ? 'No runs match your search and time period.' : 'No project flag runs yet.'
				}}
			</p>
			<div
				v-if="filteredRuns.length > PAGE_SIZE"
				class="flex flex-wrap items-center justify-between gap-3"
			>
				<p role="status" class="m-0 text-sm text-secondary">
					Showing {{ offset + 1 }}–{{ offset + runs.length }} of {{ filteredRuns.length }}
				</p>
				<div class="flex gap-2">
					<Button :disabled="offset === 0 || isFetching" @click="offset -= PAGE_SIZE"
						>Previous</Button
					>
					<Button :disabled="!hasNextPage || isFetching" @click="offset += PAGE_SIZE">Next</Button>
				</div>
			</div>
		</ProjectFlagRunActions>
	</div>
</template>

<script setup lang="ts">
import type { ProjectFlagRunResult } from '@modrinth/api-client'
import {
	CalendarIcon,
	LoaderCircleIcon,
	MoreHorizontalIcon,
	PlayIcon,
	SearchIcon,
	XCircleIcon,
	TagCategoryFlagIcon,
	BadgeCheckIcon,
} from '@modrinth/assets'
import {
	Avatar,
	Button,
	injectModrinthClient,
	Input,
	TagItem,
	TeleportOverflowMenu,
	TimeFramePicker,
	type TimeFramePickerSelection,
} from '@modrinth/ui'
import { useMutation, useQueries, useQuery, useQueryClient } from '@tanstack/vue-query'
import { useNow } from '@vueuse/core'
import Fuse from 'fuse.js'

import { getAnalyticsTimeRange } from '~/components/analytics-dashboard/query-builder/timeframe'
import {
	projectFlagResultCategory,
	projectFlagRunActivityOptions,
	projectFlagRunHistoryOptions,
	projectFlagRunResultsOptions,
} from '~/composables/queries/project-flag-run'

import ProjectFlagRunActions from './run-actions.vue'
import { countUniqueFlagKeys, flagCountTooltip, formatDate } from './run-format'
import ProjectFlagScanBadge from './scan-badge.vue'

const client = injectModrinthClient()
const api = client.labrinth.project_flags_internal
const PAGE_SIZE = 25
const offset = ref(0)
const search = ref('')
const searchTerm = computed(() => search.value.trim())
const now = useNow({ interval: 30_000 })
const timeframe = reactive<TimeFramePickerSelection>({
	mode: 'preset',
	preset: 'all_time',
	lastAmount: 7,
	lastUnit: 'days',
	customStartDate: '',
	customEndDate: '',
})
const { data: activity, error: activityError } = useQuery(projectFlagRunActivityOptions(client))
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
const { data, isPending, isError, error, isFetching, refetch } = useQuery(
	projectFlagRunHistoryOptions(client),
)
const timeFilteredRuns = computed(() => {
	const history = data.value ?? []
	if (timeframe.mode === 'preset' && timeframe.preset === 'all_time') return history

	const { start, end } = getAnalyticsTimeRange({ ...timeframe, nowTimestamp: now.value.getTime() })
	const isFixedRange =
		timeframe.mode === 'custom_range' ||
		timeframe.mode === 'custom_datetime_range' ||
		(timeframe.mode === 'preset' && timeframe.preset === 'yesterday')
	const upperBound = isFixedRange ? end.getTime() : now.value.getTime()
	return history.filter((run) => {
		const startedAt = Date.parse(run.started_at)
		return startedAt >= start.getTime() && startedAt < upperBound
	})
})
const runSearch = computed(
	() =>
		new Fuse(
			timeFilteredRuns.value.map((run) => ({
				run,
				status: run.status === 'done' ? 'Done' : 'Running',
			})),
			{
				keys: ['run.id', 'run.started_by.username', 'run.applied_by.username', 'status'],
				threshold: 0.3,
				ignoreLocation: true,
			},
		),
)
const filteredRuns = computed(() =>
	searchTerm.value
		? runSearch.value.search(searchTerm.value).map(({ item }) => item.run)
		: timeFilteredRuns.value,
)
const runs = computed(() => filteredRuns.value.slice(offset.value, offset.value + PAGE_SIZE))
const hasNextPage = computed(() => offset.value + PAGE_SIZE < filteredRuns.value.length)
watch([searchTerm, timeframe], () => {
	offset.value = 0
})
watch(
	() => filteredRuns.value.length,
	(total) => {
		const lastPage = Math.max(0, Math.ceil(total / PAGE_SIZE) - 1)
		offset.value = Math.min(offset.value, lastPage * PAGE_SIZE)
	},
)
const starterIds = computed(() =>
	[...new Set(runs.value.flatMap((run) => (run.started_by ? [run.started_by.id] : [])))].sort(),
)
const { data: starters } = useQuery({
	queryKey: computed(
		() => ['moderation', 'project-flag', 'run-starters', starterIds.value] as const,
	),
	queryFn: ({ queryKey: [, , , ids] }) => client.labrinth.users_v2.getMultiple(ids),
	enabled: computed(() => starterIds.value.length > 0),
	staleTime: 5 * 60_000,
})
const starterAvatars = computed(
	() => new Map(starters.value?.map((user) => [user.id, user.avatar_url])),
)
const resultCounts = useQueries({
	queries: computed(() =>
		runs.value.map((run) => ({
			...projectFlagRunResultsOptions(client, run),
			select: countResults,
		})),
	),
})

function countResults(results: ProjectFlagRunResult[]) {
	const counts = { scanned: results.length, flagged: 0, approvable: 0, errors: 0 }
	for (const result of results) {
		counts[projectFlagResultCategory(result)]++
	}
	return {
		...counts,
		results,
		flagTooltip: flagCountTooltip(counts.flagged, countUniqueFlagKeys(results)),
	}
}

function runPath(id: string) {
	return `/moderation/project-flag/runs/${encodeURIComponent(id)}`
}

function errorMessage(value: unknown) {
	return value instanceof Error ? value.message : 'Please try again.'
}
</script>
