<template>
	<div class="flex min-w-0 flex-col">
		<NuxtLink
			:to="runsList"
			class="inline-flex w-fit cursor-pointer items-center gap-1 rounded border-0 bg-transparent p-0 pb-2.5 text-sm font-medium text-link hover:underline focus-visible:outline focus-visible:outline-2 focus-visible:outline-brand"
		>
			<ChevronLeftIcon class="size-4" aria-hidden="true" />
			All runs
		</NuxtLink>

		<p v-if="runPending" role="status" class="universal-card flex items-center gap-2">
			<LoaderCircleIcon class="size-4 animate-spin" /> Loading project flag run…
		</p>
		<div
			v-else-if="run === undefined && runError"
			role="alert"
			class="universal-card flex flex-col items-start gap-3"
		>
			<Button @click="refetchRun()">Try again</Button>
		</div>
		<ProjectFlagRunActions v-else v-slot="{ runOptions, busy }" @deleted="returnToRuns">
			<ProjectFlagRunResults
				:run="resultRun"
				:category="category"
				:project-id="selectedProjectId"
				@change-category="showCategory"
				@select-project="selectProject"
			>
				<template #default="{ scanErrorCount, approvableCount, uniqueFlagKeyCount }">
					<header v-if="run" class="universal-card !mb-4 flex min-w-0 flex-col gap-3">
						<div class="flex flex-wrap items-center justify-between gap-3">
							<div class="flex flex-wrap items-center gap-2">
								<h2 ref="heading" tabindex="-1" class="m-0 text-xl font-semibold text-contrast">
									<time v-tooltip="formatDate(run.started_at, true)" :datetime="run.started_at">
										{{ formatDate(run.started_at) }}
									</time>
								</h2>
								<ProjectFlagScanBadge :run="run" :num-workers="activity?.num_workers" />
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
								<TagItem
									v-tooltip="
										uniqueFlagKeyCount === undefined
											? 'Flagged projects'
											: flagCountTooltip(run.num_flagged, uniqueFlagKeyCount)
									"
									style="--_color: var(--color-purple)"
								>
									<TagCategoryFlagIcon aria-hidden="true" /><span class="sr-only">Flagged: </span>
									{{ run.num_flagged.toLocaleString('en-US') }}
								</TagItem>
								<TagItem v-tooltip="'Approvable'" style="--_color: var(--color-green)">
									<BadgeCheckIcon aria-hidden="true" /><span class="sr-only">Approvable: </span>
									{{ approvableCount?.toLocaleString('en-US') ?? '—' }}
								</TagItem>
								<TagItem
									v-if="run.ruleset_id && deployment?.ruleset_id === run.ruleset_id"
									v-tooltip="formatDate(deployment.deployed_at, true)"
									style="--_bg-color: var(--color-green-bg); --_color: var(--color-green)"
								>
									Rules deployed
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
							<span v-if="run.ruleset_id"
								>· Tested revision <code>{{ run.ruleset_id }}</code></span
							>
							<span v-if="run.num_skipped">· {{ run.num_skipped }} skipped</span>
						</div>
					</header>
					<header v-else class="universal-card !mb-4 flex flex-col gap-2">
						<h2
							ref="heading"
							tabindex="-1"
							class="m-0 break-all text-xl font-semibold text-contrast"
						>
							Run {{ runId }}
						</h2>
						<p class="m-0 text-sm text-secondary">
							Temporal history is unavailable. Any stored results are shown below.
						</p>
					</header>
				</template>
			</ProjectFlagRunResults>
		</ProjectFlagRunActions>
	</div>
</template>

<script setup lang="ts">
import { useFlagErrorNotification } from '~/components/ui/moderation/project-flag/use-flag-error-notification'

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
import { flagCountTooltip, formatDate } from '~/components/ui/moderation/project-flag/run-format'
import ProjectFlagRunResults from '~/components/ui/moderation/project-flag/run-results/index.vue'
import ProjectFlagScanBadge from '~/components/ui/moderation/project-flag/scan-badge.vue'
import { projectFlagDeploymentQueryOptions } from '~/composables/queries/project-flag'
import {
	projectFlagRunActivityOptions,
	projectFlagRunDetailOptions,
} from '~/composables/queries/project-flag-run'

useHead({ title: 'Project flag run - Modrinth' })

const route = useRoute()
const router = useRouter()
const runId = computed(() => String(route.params.run))
const runsList = { path: '/moderation/project-flag', query: { tab: 'runs' } }
const client = injectModrinthClient()
const {
	data: run,
	isPending: runPending,
	error: runError,
	refetch: refetchRun,
} = useQuery(computed(() => projectFlagRunDetailOptions(client, runId.value)))
const { data: deployment } = useQuery(projectFlagDeploymentQueryOptions(client))
const resultRun = computed(() => run.value ?? { id: runId.value, status: 'unknown' as const })
const { data: activity, error: activityError } = useQuery({
	...projectFlagRunActivityOptions(client),
	enabled: computed(() => typeof run.value?.status === 'object'),
})
const starterId = computed(() => run.value?.started_by?.id ?? '')
const { data: starter, error: starterError } = useQuery({
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
	route.query.tab === 'flagged' || route.query.tab === 'approvable' || route.query.tab === 'errors'
		? route.query.tab
		: 'all',
)
const selectedProjectId = computed(() =>
	typeof route.query.project === 'string' ? route.query.project : undefined,
)

function showCategory(category: 'all' | 'flagged' | 'approvable' | 'errors') {
	return router.replace({
		query: { ...route.query, tab: category, project: undefined },
		hash: route.hash,
	})
}

function selectProject(projectId: string) {
	return router.replace({ query: { ...route.query, project: projectId }, hash: route.hash })
}
useFlagErrorNotification(runError, 'Could not load the run')
useFlagErrorNotification(activityError, 'Could not load scan activity')
useFlagErrorNotification(starterError, 'Could not load run starter')
useFlagErrorNotification(() => run.value?.error, 'Flag rule run failed')
</script>
