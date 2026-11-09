<script setup lang="ts">
import {
	ChevronRightIcon,
	CircleDashedIcon,
	CircleIcon,
	DropdownIcon,
	MessagesSquareIcon,
	PinIcon,
	SendIcon,
	SpinnerIcon,
	TriangleAlertIcon,
	XCircleIcon,
	XIcon,
} from '@modrinth/assets'
import type { Labrinth } from '@modrinth/api-client'
import { getProjectNagCategory, normalizeProjectNagKind, toProjectNag } from '@modrinth/moderation'
import {
	Accordion,
	BasicMarkdownText,
	ProjectPublishingBanner,
	Button,
	IconButton,
	OverlayScrollArea,
	Tabs,
	injectProjectPageContext,
	injectModrinthClient,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import {
	issueLocationRegistry,
	readIssueLocations,
} from '@modrinth/moderation/src/data/issues/component-builders/locations'
import { useQueryClient } from '@tanstack/vue-query'
import { onClickOutside, useEventListener } from '@vueuse/core'
import { computed, nextTick, onMounted, ref, watch } from 'vue'

import ProjectModerationContent from './ProjectModerationContent.vue'
import { threadIssueField } from '~/components/ui/project-issue-card/issue-targets'
import {
	canResubmitProjectForReview,
	canSubmitProjectForReview,
} from '~/composables/link-network-validation/submission'
import { isRejected } from '~/helpers/projects'
import {
	isThreadIssueFacetReadyToAddress,
	isThreadIssueVerified,
	threadIssueVerdict,
} from '~/helpers/thread-issues'

const {
	projectV2: project,
	projectV3,
	allMembers,
	refreshProjectValidation,
	projectValidation,
	projectValidationLoading,
	projectLinksNetworkValidationLoading,
	thread,
	setProcessing,
} = injectProjectPageContext()
const { formatMessage } = useVIntl()
const route = useRoute()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()
const addressingIssueId = ref<string>()
function issueDestination(issue: Labrinth.Threads.v3.ThreadIssue) {
	const field =
		readIssueLocations(issue.why?.locations)[0]?.field ??
		issue.facets.map((facet) => threadIssueField(facet.what)).find((field) => field !== undefined)
	if (!field) return undefined
	const area = issueLocationRegistry[field].area
	const base = `/${project.value.project_type}/${project.value.slug ?? project.value.id}/settings`
	return area ? `${base}/${area}` : `${base}#project-issue-field-${field}`
}
function addressableFacets(issue: Labrinth.Threads.v3.ThreadIssue) {
	return issue.facets.filter(
		(facet) =>
			facet.verdict === 'open' &&
			isThreadIssueFacetReadyToAddress(facet, projectV3.value, allMembers.value),
	)
}
async function markIssueAddressed(issue: Labrinth.Threads.v3.ThreadIssue) {
	if (addressingIssueId.value) return
	const facets = addressableFacets(issue)
	if (!facets.length) return
	const threadId = thread.value?.id
	addressingIssueId.value = issue.id
	try {
		const results = await Promise.allSettled(
			facets.map((facet) => client.labrinth.threads_v3.user_addressed(facet.id)),
		)
		const failure = results.find((result) => result.status === 'rejected')
		if (failure?.status === 'rejected') throw failure.reason
	} catch (error) {
		addNotification({
			title: 'Could not mark issue addressed',
			text: error instanceof Error ? error.message : String(error),
			type: 'error',
		})
	} finally {
		try {
			await Promise.all([
				queryClient.invalidateQueries({ queryKey: ['thread', threadId] }),
				refreshProjectValidation(),
			])
		} finally {
			addressingIssueId.value = undefined
		}
	}
}
const emit = defineEmits<{ pinned: [value: boolean] }>()
const open = ref(false)
const pinned = ref(false)
const view = ref<'moderation' | 'thread'>('moderation')
const anchor = ref<HTMLElement>()
const teleportTarget = ref<HTMLElement>()
onMounted(() => {
	teleportTarget.value = anchor.value?.ownerDocument.body
})
const panel = ref<HTMLElement>()
const trigger = ref<InstanceType<typeof Button>>()
watch([open, pinned], () => emit('pinned', open.value && pinned.value))
const submitting = ref(false)
const rejected = computed(() => isRejected(project.value))
const issues = computed(() =>
	(thread.value?.issues ?? []).filter((issue) => !isThreadIssueVerified(issue)),
)
const issueItems = computed(() =>
	issues.value
		.filter((issue) => threadIssueVerdict(issue) === 'open')
		.map((issue) => {
			const message = issue.why?.message ?? ''
			const heading = message.match(/^\s*#{1,6}\s+(.+)(?:\r?\n|$)/)
			const body = heading ? message.slice(heading[0].length) : message
			return {
				id: issue.id,
				source: issue,
				to: issueDestination(issue),
				title: heading?.[1]?.trim() || issue.why?.title || 'Moderator issue',
				description: body.trim(),
			}
		}),
)
const addressedIssueCount = computed(
	() => issues.value.filter((issue) => threadIssueVerdict(issue) !== 'open').length,
)
const nags = computed(() =>
	(projectValidation.value?.nags ?? [])
		.filter(
			(nag) =>
				normalizeProjectNagKind(nag.kind) &&
				normalizeProjectNagKind(nag.kind) !== 'moderator-feedback',
		)
		.map((nag) => ({
			...toProjectNag(nag, project.value.project_type),
			severity: nag.severity,
			category: getProjectNagCategory(nag.kind),
		})),
)
const remaining = computed(
	() =>
		nags.value.filter((nag) => nag.severity === 'required').length +
		issues.value.filter((issue) => threadIssueVerdict(issue) === 'open').length,
)
const checking = computed(
	() =>
		!projectValidation.value &&
		(projectValidationLoading.value || projectLinksNetworkValidationLoading.value),
)
const bannerState = computed(() => {
	if (checking.value) return 'checking'
	if (!projectValidation.value) return 'unavailable'
	switch (project.value.status) {
		case 'processing':
			return 'review'
		case 'approved':
		case 'archived':
			return 'approved'
		case 'unlisted':
			return 'unlisted'
		case 'private':
			return 'private'
		case 'withheld':
			return 'withheld'
		case 'rejected':
			return 'changes'
	}
	if (remaining.value) return 'todo'
	return project.value.status === 'draft' ? 'ready' : 'default'
})
const triggerColor = computed(() => {
	if (['review', 'withheld'].includes(bannerState.value)) return 'blue'
	if (['approved', 'unlisted', 'private'].includes(bannerState.value)) return 'green'
	if (['todo', 'changes'].includes(bannerState.value)) return 'orange'
	return undefined
})
const views = [
	{ value: 'moderation', label: 'Checklist' },
	{ value: 'thread', label: 'Thread' },
]
const disabled = computed(
	() =>
		submitting.value ||
		!canSubmitProjectForReview(
			projectValidation.value,
			projectValidationLoading.value || projectLinksNetworkValidationLoading.value,
		) ||
		(rejected.value && !canResubmitProjectForReview(thread.value)),
)
const groups = computed(() => [
	{
		title: 'TODO',
		color: 'text-secondary',
		icon: CircleIcon,
		items: [
			...nags.value.filter((nag) => nag.severity === 'required' && nag.category === 'validation'),
			...nags.value.filter((nag) => nag.severity === 'required' && nag.category === 'setup-step'),
		],
	},
	{
		title: 'Warnings',
		color: 'text-orange',
		icon: TriangleAlertIcon,
		items: nags.value.filter((nag) => nag.severity === 'warning'),
	},
	{
		title: 'Suggestions',
		color: 'text-purple',
		icon: CircleIcon,
		items: nags.value.filter((nag) => nag.severity === 'suggestion'),
	},
])
function onItemClick() {
	if (!pinned.value) close()
}

function show() {
	open.value = true
	nextTick(() => panel.value?.focus())
}

function close() {
	open.value = false
	nextTick(() => trigger.value?.element?.focus())
}
watch(
	() => route.query.moderation,
	(value) => {
		if (value) {
			view.value = value === 'thread' ? 'thread' : 'moderation'
			open.value = true
		}
	},
	{ immediate: true },
)
watch(
	() => route.path,
	() => {
		if (!pinned.value) open.value = false
	},
)
onClickOutside(
	panel,
	() => {
		if (!pinned.value) open.value = false
	},
	{ ignore: [trigger, anchor] },
)
useEventListener('keydown', (event) => {
	if (event.key === 'Escape' && open.value) close()
})
async function submit() {
	if (disabled.value) return
	submitting.value = true
	try {
		await setProcessing()
	} finally {
		submitting.value = false
	}
}
</script>

<template>
	<div ref="anchor">
		<ProjectPublishingBanner
			:state="bannerState"
			:remaining="remaining"
			:expanded="open"
			controls="project-moderation-drawer"
			@open="show"
		/>
	</div>
	<Teleport v-if="teleportTarget" :to="teleportTarget">
		<Button
			v-if="!open"
			ref="trigger"
			:type="triggerColor ? 'colored' : 'base'"
			:color="triggerColor"
			class="!fixed right-0 top-[46%] z-30 !min-h-14 !min-w-20 !rounded-r-none !px-4"
			size="lg"
			aria-label="Open review checklist"
			aria-controls="project-moderation-drawer"
			:aria-expanded="open"
			@click="show"
		>
			<SpinnerIcon v-if="checking" class="animate-spin" aria-hidden="true" />
			<CircleIcon v-else-if="remaining" /><MessagesSquareIcon v-else />
			<span class="bg-surface-1/10 rounded-md text-base">{{ remaining }}</span>
		</Button>
		<Transition name="moderation-drawer" appear>
			<aside
				v-show="open"
				id="project-moderation-drawer"
				ref="panel"
				tabindex="-1"
				aria-label="Review and publishing"
				class="fixed bottom-0 right-0 top-0 z-40 flex w-full max-w-[400px] flex-col border-0 border-l border-solid border-divider bg-surface-3 shadow-xl"
			>
				<header
					class="flex shrink-0 items-center gap-2 border-0 border-b border-solid border-divider p-4"
				>
					<Tabs
						:value="view"
						:tabs="views"
						aria-label="Review views"
						@update:value="view = $event === 'thread' ? 'thread' : 'moderation'"
					/>
					<div class="ml-auto flex shrink-0 items-center gap-2">
						<IconButton
							type="quiet"
							:label="pinned ? 'Unpin panel' : 'Pin panel open'"
							:aria-pressed="pinned"
							:class="{ '!bg-bg-green !text-green': pinned }"
							@click="pinned = !pinned"
						>
							<PinIcon />
						</IconButton>
						<IconButton type="quiet" label="Close moderation panel" @click="close">
							<XIcon />
						</IconButton>
					</div>
				</header>
				<ProjectPublishingBanner
					v-if="view === 'moderation'"
					compact
					:state="bannerState"
					:remaining="remaining"
					class="px-5 py-3"
				/>
				<OverlayScrollArea
					v-if="view === 'moderation'"
					class="min-w-0 flex-1"
				>
					<div class="min-w-0 p-3 px-2.5 [overflow-wrap:anywhere]">
						<div v-if="issues.length" class="mb-6 px-2.5">
							<p class="mb-2 mt-0 text-xs text-secondary">
								{{ addressedIssueCount }} of {{ issues.length }} issues addressed
							</p>
							<div
								role="progressbar"
								aria-label="Moderator issues addressed"
								:aria-valuenow="addressedIssueCount"
								:aria-valuemax="issues.length"
								:aria-valuemin="0"
								class="h-1.5 overflow-hidden rounded-full bg-surface-5"
							>
								<div
									class="h-full rounded-full bg-orange"
									:style="{ width: `${(addressedIssueCount / issues.length) * 100}%` }"
								/>
							</div>
						</div>
						<Accordion
							v-for="group in groups.filter(
								(group) => group.items.length || (group.title === 'TODO' && issueItems.length),
							)"
							:key="group.title"
							open-by-default
							class="mb-4 last:mb-0"
							button-class="group/checklist-heading m-0 flex w-full cursor-pointer items-center gap-2 border-0 bg-transparent px-2.5 py-2 text-xs font-semibold uppercase tracking-wide text-primary"
						>
							<template #button="{ open: expanded }">
								<span
									class="transition-[filter] duration-150 group-hover/checklist-heading:brightness-125 motion-reduce:transition-none"
									>{{ group.title }}</span
								>
								<span class="rounded-md bg-surface-2 px-1.5 py-0.5">{{
									group.items.length + (group.title === 'TODO' ? issueItems.length : 0)
								}}</span
								><DropdownIcon
									class="ml-auto size-4 transition-[transform,filter] duration-300 group-hover/checklist-heading:brightness-125 motion-reduce:transition-none"
									:class="{ '-rotate-90': !expanded }"
								/>
							</template>
							<template v-if="group.title === 'TODO'">
								<div
									v-for="issue in issueItems"
									:key="issue.id"
									class="group relative flex min-w-0 items-start gap-1.5 rounded-lg px-2.5 py-3"
									:class="{ 'hover:bg-surface-2': issue.to }"
								>
									<TriangleAlertIcon
										class="mt-0.5 size-4 shrink-0 text-orange"
										aria-hidden="true"
									/>
									<div class="min-w-0 flex-1">
										<p class="m-0 text-sm font-semibold text-contrast">{{ issue.title }}</p>
										<BasicMarkdownText
											v-if="issue.description"
											:text="issue.description"
											class="mt-1 text-xs leading-relaxed text-secondary"
										/>
										<div v-if="!issue.to" class="mt-3 flex justify-end">
											<Button
												size="sm"
												:disabled="!!addressingIssueId || !addressableFacets(issue.source).length"
												:loading="addressingIssueId === issue.id"
												@click="markIssueAddressed(issue.source)"
												>Mark addressed</Button
											>
										</div>
									</div>
									<ChevronRightIcon
										v-if="issue.to"
										class="mt-0.5 size-4 shrink-0 text-secondary opacity-0 transition-opacity group-hover:opacity-100"
									/>
									<NuxtLink
										v-if="issue.to"
										:to="issue.to"
										class="absolute inset-0 rounded-lg focus-visible:outline focus-visible:outline-brand"
										:aria-label="`View issue: ${issue.title}`"
										@click="onItemClick"
									/>
								</div>
							</template>
							<div
								v-for="nag in group.items"
								:key="nag.id"
								class="group relative flex items-start gap-1.5 rounded-lg p-3 px-2.5 transition-colors hover:bg-surface-2"
							>
								<CircleDashedIcon
									v-if="nag.severity === 'suggestion'"
									aria-hidden="true"
									class="mt-0.5 size-4 shrink-0 text-secondary"
								/>
								<component
									v-else
									:is="
										nag.severity === 'required' && nag.category === 'validation'
											? XCircleIcon
											: group.icon
									"
									class="mt-0.5 size-4 shrink-0"
									:class="
										nag.severity === 'required' && nag.category === 'validation'
											? 'text-red'
											: group.color
									"
								/>
								<div class="min-w-0 flex-1">
									<p class="m-0 text-sm font-semibold text-contrast">
										{{
											typeof nag.title === 'string'
												? nag.title
												: formatMessage(nag.title, nag.values)
										}}
									</p>
									<p
										v-if="typeof nag.description !== 'function'"
										class="mb-0 mt-1 text-xs leading-relaxed text-secondary"
									>
										{{
											typeof nag.description === 'string'
												? nag.description
												: formatMessage(nag.description, nag.values)
										}}
									</p>
								</div>
								<ChevronRightIcon
									class="mt-0.5 size-4 shrink-0 text-secondary opacity-0 transition-opacity group-hover:opacity-100"
								/>
								<NuxtLink
									v-if="nag.link"
									:to="`/${project.project_type}/${project.slug ?? project.id}/${nag.link.path}`"
									class="absolute inset-0 rounded-lg text-transparent focus-visible:outline focus-visible:outline-brand"
									@click="onItemClick"
									>{{
										typeof nag.link.title === 'string'
											? nag.link.title
											: formatMessage(nag.link.title)
									}}</NuxtLink
								>
							</div>
						</Accordion>
					</div>
				</OverlayScrollArea>
				<div
					v-show="view === 'thread' && open"
					class="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden"
				>
					<Suspense>
						<KeepAlive>
							<ProjectModerationContent v-if="view === 'thread' && open" view="thread" />
						</KeepAlive>
					</Suspense>
				</div>
				<div
					v-if="view === 'moderation' && (project.status === 'draft' || rejected)"
					class="border-0 border-t border-solid border-divider p-4"
				>
					<div
						v-tooltip="
							disabled && remaining ? 'Complete the required items to continue.' : undefined
						"
					>
						<Button class="w-full" :disabled="disabled" @click="submit"
							><SendIcon />{{ rejected ? 'Resubmit for review' : 'Submit for review' }}</Button
						>
					</div>
				</div>
			</aside>
		</Transition>
	</Teleport>
</template>

<style scoped>
.moderation-drawer-enter-active {
	transition: transform 240ms cubic-bezier(0.16, 1, 0.3, 1);
}

.moderation-drawer-leave-active {
	transition: transform 180ms ease-in;
	pointer-events: none;
}

.moderation-drawer-enter-from,
.moderation-drawer-leave-to {
	transform: translateX(100%);
}

@media (prefers-reduced-motion: reduce) {
	.moderation-drawer-enter-active,
	.moderation-drawer-leave-active {
		transition: none;
	}
}
</style>
