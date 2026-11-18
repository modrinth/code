<script setup lang="ts">
import {
	CheckIcon,
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
import {
	getProjectNagCategory,
	nagDefinitions,
	normalizeProjectNagKind,
	toProjectNag,
} from '@modrinth/moderation'
import {
	BasicMarkdownText,
	Button,
	defineMessages,
	IconButton,
	OverlayScrollArea,
	Tabs,
	getActiveDisclosures,
	injectProjectPageContext,
	injectModrinthClient,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { readIssueLocations } from '@modrinth/moderation/src/data/issues/component-builders/locations'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { onClickOutside, useEventListener } from '@vueuse/core'
import { computed, nextTick, onMounted, ref, watch } from 'vue'

import ChecklistAccordion from './checklist-accordion.vue'
import ProjectModerationContent from './project-moderation-content.vue'
import ProjectPublishingBanner from './project-publishing-banner.vue'
import { projectIssueSettingsPath } from '~/components/project-settings/settings-shell/destinations'
import { threadIssueField } from '~/components/ui/project-issue-card/issue-targets'
import {
	canResubmitProjectForReview,
	canSubmitProjectForReview,
} from '~/composables/link-network-validation/submission'
import { injectProjectSettingsNavigation } from '~/composables/project-settings-navigation'
import { projectQueryOptions } from '~/composables/queries/project'
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
const messages = defineMessages({
	requestedByModerator: {
		id: 'project.settings.review.requested-by-moderator',
		defaultMessage: 'Requested by moderator',
	},
	automaticCheck: {
		id: 'project.settings.review.automatic-check',
		defaultMessage: 'Automatic check',
	},
	completed: {
		id: 'project.settings.review.completed',
		defaultMessage: 'Completed',
	},
	writeSummary: {
		id: 'project.settings.review.completed.write-summary',
		defaultMessage: 'Write a summary',
	},
	chooseLicense: {
		id: 'project.settings.review.completed.choose-license',
		defaultMessage: 'Choose a license',
	},
	writeDescription: {
		id: 'project.settings.review.completed.write-description',
		defaultMessage: 'Write a description',
	},
})
const route = useRoute()
const settingsNavigation = injectProjectSettingsNavigation()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()
const addressingIssueId = ref<string>()
function issueHighlightField(issue: Labrinth.Threads.v3.ThreadIssue) {
	return (
		readIssueLocations(issue.why?.locations)[0]?.field ??
		issue.facets.map((facet) => threadIssueField(facet.what)).find((field) => field !== undefined)
	)
}
function settingsDestination(path: string, field?: string) {
	const base = `/${project.value.project_type}/${project.value.slug ?? project.value.id}`
	const destination = `${base}/${path}`
	if (path !== 'settings' || !field) return destination
	const anchor = field === 'license' ? 'license' : `project-settings-field-${field}`
	return `${destination}#${anchor}`
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
const { data: disclosures } = useQuery(
	computed(() => ({
		...projectQueryOptions.disclosures(projectV3.value.id, client),
		enabled: open.value && view.value === 'moderation' && !!projectV3.value.id,
	})),
)
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
	(thread.value?.issues ?? [])
		.filter((issue) => !isThreadIssueVerified(issue))
		.map((issue) => {
			const message = issue.why?.message ?? ''
			const heading = message.match(/^\s*#{1,6}\s+(.+)(?:\r?\n|$)/)
			return {
				source: issue,
				verdict: threadIssueVerdict(issue),
				title: heading?.[1]?.trim() || issue.why?.title || 'Moderator issue',
				description: (heading ? message.slice(heading[0].length) : message).trim(),
			}
		}),
)
const issueItems = computed(() =>
	issues.value
		.filter((issue) => issue.verdict === 'open')
		.map(({ source, title, description }) => {
			const field = issueHighlightField(source)
			return {
				id: source.id,
				source,
				to: field ? settingsDestination(projectIssueSettingsPath(field), field) : undefined,
				highlightField: field,
				title,
				description,
			}
		}),
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
const completedItems = computed(() => {
	const data = projectV3.value
	const items = issues.value
		.filter((issue) => issue.verdict !== 'open')
		.map(({ source, title }) => ({ id: `issue-${source.id}`, title }))
	if (data.summary.trim()) {
		items.push({ id: 'write-summary', title: formatMessage(messages.writeSummary) })
	}
	if (!projectValidation.value) return items

	const outstanding = new Set(
		projectValidation.value.nags.map(({ kind }) => normalizeProjectNagKind(kind)),
	)
	const isServer = data.minecraft_server != null
	const hasVersions = data.versions.length > 0
	const candidates = [
		['select-license', !isServer && !!data.license.id, messages.chooseLicense],
		[
			'add-description',
			!!data.description.trim() && !outstanding.has('description-too-short'),
			messages.writeDescription,
		],
		['upload-version', !isServer && hasVersions],
		['add-icon', !!data.icon_url],
		['feature-gallery-image', !isServer && data.gallery.some((image) => image.featured)],
		[isServer ? 'add-links-server' : 'add-links', Object.keys(data.link_urls).length > 0],
		['select-language', !!data.minecraft_server?.languages?.length],
		['select-tags', hasVersions && data.categories.length > 0],
		['check-disclosures', getActiveDisclosures(disclosures.value?.disclosures).length > 0],
	] as const

	for (const [id, complete, title] of candidates) {
		if (complete && !outstanding.has(id)) {
			items.push({ id, title: formatMessage(title ?? nagDefinitions[id].title) })
		}
	}
	return items
})
const remaining = computed(
	() => nags.value.filter((nag) => nag.severity === 'required').length + issueItems.value.length,
)
const checking = computed(
	() =>
		!projectValidation.value &&
		(projectValidationLoading.value || projectLinksNetworkValidationLoading.value),
)
const canSubmit = computed(
	() =>
		canSubmitProjectForReview(
			projectValidation.value,
			projectValidationLoading.value || projectLinksNetworkValidationLoading.value,
		) &&
		(!rejected.value || canResubmitProjectForReview(thread.value)),
)
const bannerState = computed(() => {
	const status = project.value.status
	if (rejected.value && (canSubmit.value || submitting.value)) return 'ready-to-resubmit'
	if (status === 'draft' && submitting.value) return 'ready'
	if (checking.value) return 'checking'
	if (!projectValidation.value) return 'unavailable'
	switch (status) {
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
	if (status === 'draft') return canSubmit.value ? 'ready' : 'checking'
	return 'default'
})
const triggerColor = computed(() => {
	if (['review', 'withheld'].includes(bannerState.value)) return 'blue'
	if (['approved', 'unlisted', 'private', 'ready', 'ready-to-resubmit'].includes(bannerState.value))
		return 'green'
	if (['todo', 'changes'].includes(bannerState.value)) return 'orange'
	return undefined
})
const views = [
	{ value: 'moderation', label: 'Checklist' },
	{ value: 'thread', label: 'Thread' },
]
const disabled = computed(() => submitting.value || !canSubmit.value)
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
function onItemClick(event: MouseEvent, path: string, field?: string) {
	if (event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return
	event.preventDefault()
	if (!pinned.value) open.value = false
	void settingsNavigation.reveal(path.replace(/#.*$/, ''), field)
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
			:submitting="submitting"
			@open="show"
			@submit="submit"
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
					:completed="completedItems.length"
					class="mb-1.5 px-5 py-3 pb-1.5"
				/>
				<OverlayScrollArea v-show="view === 'moderation'" scroll-shadows class="min-w-0 flex-1">
					<div class="min-w-0 p-3 px-2.5 [overflow-wrap:anywhere]">
						<ChecklistAccordion
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
										<p class="m-0 text-sm font-semibold text-contrast">
											{{ issue.title }}
										</p>
										<p class="mb-0 mt-0.5 text-xs text-orange">
											{{ formatMessage(messages.requestedByModerator) }}
										</p>
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
										v-slot="{ href, route: destination }"
										:to="issue.to"
										custom
									>
										<a
											:href="href"
											class="absolute inset-0 rounded-lg focus-visible:outline focus-visible:outline-brand"
											:aria-label="`View issue: ${issue.title}`"
											@click="onItemClick($event, destination.fullPath, issue.highlightField)"
										/>
									</NuxtLink>
								</div>
							</template>
							<div
								v-for="nag in group.items"
								:key="nag.id"
								class="group relative flex items-start gap-2 rounded-lg p-3 px-2.5 transition-colors hover:bg-surface-4"
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
										v-if="
											nag.category === 'validation' &&
											(nag.severity === 'required' || nag.severity === 'warning')
										"
										class="mb-0 mt-0.5 text-xs text-red"
									>
										{{ formatMessage(messages.automaticCheck) }}
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
									v-slot="{ href, route: destination }"
									:to="settingsDestination(nag.link.path, nag.highlightField)"
									custom
								>
									<a
										:href="href"
										class="absolute inset-0 rounded-lg text-transparent focus-visible:outline focus-visible:outline-brand"
										@click="onItemClick($event, destination.fullPath, nag.highlightField)"
										>{{
											typeof nag.link.title === 'string'
												? nag.link.title
												: formatMessage(nag.link.title)
										}}</a
									>
								</NuxtLink>
							</div>
						</ChecklistAccordion>
						<ChecklistAccordion
							v-if="completedItems.length"
							button-class="m-0 flex w-full cursor-pointer items-center gap-2 border-0 bg-transparent px-2.5 py-2 text-xs font-semibold uppercase tracking-wide text-secondary hover:text-primary"
						>
							<template #button="{ open: expanded }">
								{{ formatMessage(messages.completed) }}
								<span>{{ completedItems.length }}</span>
								<DropdownIcon
									aria-hidden="true"
									class="ml-auto size-4 transition-transform duration-300 motion-reduce:transition-none"
									:class="{ '-rotate-90': !expanded }"
								/>
							</template>
							<ul class="m-0 list-none p-0">
								<li
									v-for="item in completedItems"
									:key="item.id"
									class="flex items-center gap-3 px-2.5 py-2.5 text-sm font-semibold text-secondary"
								>
									<span
										class="flex size-[18px] shrink-0 items-center justify-center rounded-full bg-green text-[var(--color-accent-contrast)]"
										aria-hidden="true"
									>
										<CheckIcon class="size-3" />
									</span>
									{{ item.title }}
								</li>
							</ul>
						</ChecklistAccordion>
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
