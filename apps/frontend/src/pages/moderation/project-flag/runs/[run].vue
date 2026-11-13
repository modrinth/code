<template>
	<div class="flex min-w-0 flex-col">
		<NuxtLink
			:to="runsList"
			class="inline-flex w-fit cursor-pointer items-center gap-1 rounded border-0 bg-transparent p-0 pb-2.5 text-sm font-medium text-link hover:underline focus-visible:outline focus-visible:outline-2 focus-visible:outline-brand"
		>
			<ChevronLeftIcon class="size-4" aria-hidden="true" />
			All runs
		</NuxtLink>

		<p
			v-if="!run && (historyPending || historyFetching)"
			role="status"
			class="universal-card flex items-center gap-2"
		>
			<LoaderCircleIcon class="size-4 animate-spin" /> Loading project flag run…
		</p>
		<div
			v-else-if="!run && historyError"
			role="alert"
			class="universal-card flex flex-col items-start gap-3"
		>
			<p class="m-0 text-red">Could not load the run: {{ historyError.message }}</p>
			<Button @click="refetchHistory()">Try again</Button>
		</div>
		<p v-else-if="!run" class="universal-card m-0 text-secondary">Project flag run not found.</p>
		<ProjectFlagRunActions v-else v-slot="{ runOptions, busy }" @deleted="returnToRuns">
			<ProjectFlagRunResults
				:run="run"
				:category="category"
				:project-id="selectedProjectId"
				@change-category="showCategory"
				@select-project="selectProject"
			>
				<template #default="{ scannedCount, scanErrorCount, flaggedCount, approvableCount }">
					<header class="universal-card !mb-4 flex min-w-0 flex-col gap-3">
						<div class="flex flex-wrap items-center justify-between gap-3">
							<div class="flex flex-wrap items-center gap-2">
								<h2 ref="heading" tabindex="-1" class="m-0 text-xl font-semibold text-contrast">
									<time v-tooltip="formatDate(run.started_at, true)" :datetime="run.started_at">
										{{ formatDate(run.started_at) }}
									</time>
								</h2>
								<ProjectFlagScanBadge
									:run="run"
									:scanned="scannedCount"
									:num-consumers="activity?.num_consumers"
								/>
							</div>
							<div class="flex flex-wrap items-center gap-2">
								<TagItem
									v-if="scanErrorCount > 0"
									v-tooltip="'Scan errors'"
									style="--_color: var(--color-red)"
								>
									<XCircleIcon aria-hidden="true" /><span class="sr-only">Scan errors: </span>
									{{ scanErrorCount.toLocaleString('en-US') }}
								</TagItem>
								<TagItem v-tooltip="'Flagged'" style="--_color: var(--color-purple)">
									<TagCategoryFlagIcon aria-hidden="true" /><span class="sr-only">Flagged: </span>
									{{ flaggedCount?.toLocaleString('en-US') ?? '—' }}
								</TagItem>
								<TagItem v-tooltip="'Approvable'" style="--_color: var(--color-green)">
									<BadgeCheckIcon aria-hidden="true" /><span class="sr-only">Approvable: </span>
									{{ approvableCount?.toLocaleString('en-US') ?? '—' }}
								</TagItem>
								<TagItem
									v-if="run.applied_at"
									v-tooltip="formatDate(run.applied_at, true)"
									style="--_bg-color: var(--color-green-bg); --_color: var(--color-green)"
								>
									Applied
								</TagItem>
								<TeleportOverflowMenu
									type="quiet"
									label="Run options"
									tooltip="Run options"
									:options="runOptions(run)"
									:disabled="busy"
								>
									<MoreHorizontalIcon aria-hidden="true" />
								</TeleportOverflowMenu>
							</div>
						</div>
						<div class="flex flex-wrap items-center gap-2 text-sm text-secondary">
							<span>Started by</span>
							<NuxtLink
								v-if="run.started_by"
								:to="`/user/${run.started_by.id}`"
								class="inline-flex items-center gap-2 font-medium text-contrast hover:underline"
							>
								<Avatar :src="starter?.avatar_url" size="1.5rem" circle no-shadow />
								{{ run.started_by.username }}
							</NuxtLink>
							<span v-else>unknown user</span>
						</div>
						<p v-if="run.error" role="alert" class="m-0 whitespace-pre-wrap break-words text-red">
							{{ run.error }}
						</p>
					</header>
				</template>
			</ProjectFlagRunResults>
		</ProjectFlagRunActions>
	</div>
</template>

<script setup lang="ts">
import {
	BadgeCheckIcon,
	ChevronLeftIcon,
	LoaderCircleIcon,
	MoreHorizontalIcon,
	TagCategoryFlagIcon,
	XCircleIcon,
} from '@modrinth/assets'
import { Avatar, Button, injectModrinthClient, TagItem, TeleportOverflowMenu } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'

import ProjectFlagRunActions from '~/components/ui/moderation/project-flag/run-actions.vue'
import { formatDate } from '~/components/ui/moderation/project-flag/run-format'
import ProjectFlagRunResults from '~/components/ui/moderation/project-flag/run-results/index.vue'
import ProjectFlagScanBadge from '~/components/ui/moderation/project-flag/scan-badge.vue'
import {
	projectFlagRunActivityOptions,
	projectFlagRunHistoryOptions,
} from '~/composables/queries/project-flag-run'

useHead({ title: 'Project flag run - Modrinth' })

const route = useRoute()
const router = useRouter()
const runId = computed(() => String(route.params.run))
const runsList = { path: '/moderation/project-flag', query: { tab: 'runs' } }
const client = injectModrinthClient()
const {
	data: history,
	isPending: historyPending,
	error: historyError,
	isFetching: historyFetching,
	refetch: refetchHistory,
} = useQuery(projectFlagRunHistoryOptions(client))
const run = computed(() => history.value?.find(({ id }) => id === runId.value))
const { data: activity } = useQuery({
	...projectFlagRunActivityOptions(client),
	enabled: computed(() => !!run.value && run.value.status !== 'done'),
})
const starterId = computed(() => run.value?.started_by?.id ?? '')
const { data: starter } = useQuery({
	queryKey: computed(() => ['moderation', 'project-flag', 'run-starter', starterId.value] as const),
	queryFn: ({ queryKey: [, , , id] }) => client.labrinth.users_v2.get(id),
	enabled: computed(() => !!starterId.value),
	staleTime: 5 * 60_000,
})

function returnToRuns(deletedRunId: string) {
	if (deletedRunId === runId.value) return navigateTo(runsList, { replace: true })
}

const heading = useTemplateRef<HTMLHeadingElement>('heading')
watch(heading, (element) => element?.focus(), { flush: 'post' })

const category = computed(() =>
	route.query.tab === 'approvable' || route.query.tab === 'errors' ? route.query.tab : 'flagged',
)
const selectedProjectId = computed(() =>
	typeof route.query.project === 'string' ? route.query.project : undefined,
)

function showCategory(category: 'flagged' | 'approvable' | 'errors') {
	return router.replace({
		query: { ...route.query, tab: category, project: undefined },
		hash: route.hash,
	})
}

function selectProject(projectId: string) {
	return router.replace({ query: { ...route.query, project: projectId }, hash: route.hash })
}
</script>
