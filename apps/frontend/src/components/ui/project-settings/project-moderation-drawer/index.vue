<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	CheckIcon,
	ChevronRightIcon,
	CircleAlertIcon,
	CircleDashedIcon,
	CircleIcon,
	DropdownIcon,
	ListTodoIcon,
	MessageSquareWarningIcon,
	MessagesSquareIcon,
	PinIcon,
	RightArrowIcon,
	SendIcon,
	XCircleIcon,
	XIcon,
} from '@modrinth/assets'
import {
	getProjectNagCategory,
	nagDefinitions,
	nagDestinations,
	normalizeProjectNagKind,
	toProjectNag,
} from '@modrinth/moderation'
import { readIssueLocations } from '@modrinth/moderation/src/data/issues/component-builders/locations'
import {
	Button,
	defineMessages,
	getActiveDisclosures,
	IconButton,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	OverlayScrollArea,
	Tabs,
	TagItem,
	useVIntl,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { onClickOutside, useEventListener } from '@vueuse/core'
import { computed, nextTick, onMounted, ref, watch } from 'vue'

import { projectIssueSettingsPath } from '~/components/project-settings/settings-shell/destinations'
import IssueMessage from '~/components/ui/project-issue-card/issue-message.vue'
import { threadIssueField } from '~/components/ui/project-issue-card/issue-targets'
import { injectProjectSettingsNavigation } from '~/composables/project-settings-navigation'
import { projectQueryOptions } from '~/composables/queries/project'
import { isThreadIssueFacetReadyToAddress } from '~/helpers/thread-issues'
import { injectProjectPublishing, isProjectChecklistVisible } from '~/providers/project-publishing'

import ChecklistAccordion from './checklist-accordion.vue'
import { moderationDrawerWidth } from './layout'
import { publishingActionMessages } from './messages'
import OpenDrawerButton from './open-drawer-button.vue'
import ProjectModerationContent from './project-moderation-content.vue'
import ProjectPublishingBanner from './project-publishing-banner.vue'

const {
	projectV2: project,
	projectV3,
	allMembers,
	refreshProjectValidation,
	projectValidation,
	thread,
	setProcessing,
} = injectProjectPageContext()
const {
	bannerDismissed,
	canSubmit,
	dismissBanner,
	issues: publishingIssues,
	nags: publishingNags,
	rejected,
	remaining,
	state: publishingState,
	submitting,
} = injectProjectPublishing()
const showChecklist = computed(
	() =>
		projectV3.value.status === 'withheld' ||
		isProjectChecklistVisible(publishingState.value, remaining.value),
)
const { formatMessage } = useVIntl()
const messages = defineMessages({
	drawerLabel: {
		id: 'project.settings.review.drawer-label',
		defaultMessage: 'Review and publishing',
	},
	viewsLabel: {
		id: 'project.settings.review.views-label',
		defaultMessage: 'Review views',
	},
	closePanel: {
		id: 'project.settings.review.close-panel',
		defaultMessage: 'Close moderation panel',
	},
	unpinPanel: {
		id: 'project.settings.review.unpin-panel',
		defaultMessage: 'Unpin panel',
	},
	pinPanel: {
		id: 'project.settings.review.pin-panel',
		defaultMessage: 'Pin panel open',
	},
	markAddressed: {
		id: 'project.settings.review.mark-addressed',
		defaultMessage: 'Mark addressed',
	},
	markAddressedFailed: {
		id: 'project.settings.review.mark-addressed-failed',
		defaultMessage: 'Could not mark issue addressed',
	},
	moderatorIssue: {
		id: 'project.settings.review.moderator-issue',
		defaultMessage: 'Moderator issue',
	},
	viewIssue: {
		id: 'project.settings.review.view-issue',
		defaultMessage: 'View issue: {title}',
	},
	checklist: {
		id: 'project.settings.review.checklist',
		defaultMessage: 'Checklist',
	},
	thread: {
		id: 'project.settings.review.thread',
		defaultMessage: 'Thread',
	},
	todo: {
		id: 'project.settings.review.todo',
		defaultMessage: 'TODO',
	},
	warnings: {
		id: 'project.settings.review.warnings',
		defaultMessage: 'Warnings',
	},
	suggestions: {
		id: 'project.settings.review.suggestions',
		defaultMessage: 'Suggestions',
	},
	completeRequired: {
		id: 'project.settings.review.complete-required',
		defaultMessage: 'Complete the required items to continue.',
	},
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
			title: formatMessage(messages.markAddressedFailed),
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
type DrawerView = 'moderation' | 'thread'
const open = ref(false)
const pinned = ref(false)
const selectedView = ref<DrawerView>('moderation')
const view = computed(() => (showChecklist.value ? selectedView.value : 'thread'))
let openedFrom: DrawerView = 'moderation'
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
const launcher = ref<InstanceType<typeof OpenDrawerButton>>()
const launcherElement = computed(() => launcher.value?.element)
watch([open, pinned], ([isOpen, isPinned]) => {
	emit('pinned', isOpen && isPinned)
})
const issues = computed(() =>
	publishingIssues.value.map(({ source, verdict }) => {
		const message = source.why?.message ?? ''
		const heading = message.match(/^\s*#{1,6}\s+(.+)(?:\r?\n|$)/)
		const field = issueHighlightField(source)
		return {
			id: source.id,
			source,
			verdict,
			to: field ? settingsDestination(projectIssueSettingsPath(field), field) : undefined,
			highlightField: field,
			title: heading?.[1]?.trim() || source.why?.title || formatMessage(messages.moderatorIssue),
			description: (heading ? message.slice(heading[0].length) : message).trim(),
		}
	}),
)
const issueItems = computed(() => issues.value.filter((issue) => issue.verdict === 'open'))
const nags = computed(() =>
	publishingNags.value.map((nag) => {
		const category = getProjectNagCategory(nag.kind)
		return {
			...toProjectNag(nag, project.value.project_type),
			severity: nag.severity,
			category,
			isRequiredValidation: nag.severity === 'required' && category === 'validation',
		}
	}),
)
const threadDestination = computed(() => settingsDestination('moderation'))
const completedItems = computed(() => {
	const data = projectV3.value
	const items = issues.value
		.filter((issue) => issue.verdict !== 'open')
		.map(({ id, title, to, highlightField }) => ({ id: `issue-${id}`, title, to, highlightField }))
	if (data.summary.trim()) {
		items.push({
			id: 'write-summary',
			title: formatMessage(messages.writeSummary),
			to: settingsDestination(nagDestinations.general.path, 'summary'),
			highlightField: 'summary',
		})
	}
	if (!projectValidation.value) return items

	const outstanding = new Set(
		projectValidation.value.nags.map(({ kind }) => normalizeProjectNagKind(kind)),
	)
	const isServer = data.minecraft_server != null
	const hasVersions = data.versions.length > 0
	const candidates = [
		['select-license', !isServer && !!data.license.id, messages.chooseLicense],
		['add-description', !!data.description.trim(), messages.writeDescription],
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
			const definition = nagDefinitions[id]
			items.push({
				id,
				title: formatMessage(title ?? definition.title),
				to: settingsDestination(
					nagDestinations[definition.destination].path,
					definition.highlightField,
				),
				highlightField: definition.highlightField,
			})
		}
	}
	return items
})
const views = computed(() =>
	[
		{ value: 'moderation', label: formatMessage(messages.checklist), icon: ListTodoIcon },
		{ value: 'thread', label: formatMessage(messages.thread), icon: MessagesSquareIcon },
	].filter((tab) => showChecklist.value || tab.value === 'thread'),
)
const groups = computed(() => [
	{
		id: 'todo',
		title: formatMessage(messages.todo),
		color: 'text-secondary',
		icon: CircleIcon,
		items: [
			...nags.value.filter((nag) => nag.severity === 'required' && nag.category === 'validation'),
			...nags.value.filter((nag) => nag.severity === 'required' && nag.category === 'setup-step'),
		],
	},
	{
		id: 'warnings',
		title: formatMessage(messages.warnings),
		color: 'text-orange',
		icon: CircleAlertIcon,
		items: nags.value.filter((nag) => nag.severity === 'warning'),
	},
	{
		id: 'suggestions',
		title: formatMessage(messages.suggestions),
		color: 'text-purple',
		icon: CircleIcon,
		items: nags.value.filter((nag) => nag.severity === 'suggestion'),
	},
])
function onItemClick(event: MouseEvent, path: string | undefined, field?: string) {
	if (
		!path ||
		event.button !== 0 ||
		event.metaKey ||
		event.ctrlKey ||
		event.shiftKey ||
		event.altKey
	)
		return
	event.preventDefault()
	if (path === threadDestination.value) {
		show('thread')
		return
	}
	if (!pinned.value) open.value = false
	void settingsNavigation.reveal(path.replace(/#.*$/, ''), field)
}

function show(target: DrawerView) {
	if (!open.value) openedFrom = target
	selectedView.value = target
	open.value = true
	nextTick(() => panel.value?.focus())
}

defineExpose({ show })

function openPublishingDetails() {
	switch (publishingState.value) {
		case 'review':
		case 'withheld':
		case 'approved':
		case 'unlisted':
		case 'private':
			show('thread')
			break
		default:
			show('moderation')
	}
}

function close() {
	const focused = panel.value?.contains(panel.value.ownerDocument.activeElement)
	open.value = false
	if (focused) nextTick(() => launcher.value?.focusTrigger(openedFrom))
}
watch(
	() => route.query.moderation,
	(value) => {
		if (value) {
			selectedView.value = value === 'thread' ? 'thread' : 'moderation'
			if (!open.value) openedFrom = selectedView.value
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
const modalSelector = '[data-modal-root], [aria-modal="true"]'
const popoverSelector = '[role="menu"], [data-popper-placement], .v-popper__inner'

onClickOutside(
	panel,
	() => {
		if (!pinned.value) open.value = false
	},
	{
		ignore: [launcherElement, anchor, modalSelector, popoverSelector],
	},
)
useEventListener('keydown', (event) => {
	if (
		event.key === 'Escape' &&
		open.value &&
		!event.defaultPrevented &&
		!panel.value?.ownerDocument.querySelector(modalSelector)
	) {
		close()
	}
})
</script>

<template>
	<div ref="anchor" class="empty:hidden">
		<ProjectPublishingBanner
			v-if="!bannerDismissed"
			:state="publishingState"
			:remaining="remaining"
			:expanded="open"
			:controls="settingsNavigation.moderationDrawerId"
			:submitting="submitting"
			@open="openPublishingDetails"
			@submit="setProcessing"
			@dismiss="dismissBanner"
		/>
	</div>
	<Teleport v-if="teleportTarget" :to="teleportTarget">
		<OpenDrawerButton
			v-if="!open"
			ref="launcher"
			class="fixed right-0 top-[47.5%] z-30 -translate-y-1/2"
			:state="publishingState"
			:remaining="remaining"
			:controls="settingsNavigation.moderationDrawerId"
			@open="show"
		/>
		<div
			class="fixed bottom-0 right-0 top-0 z-40 w-full"
			:class="open ? 'pointer-events-auto' : 'pointer-events-none'"
			:style="{ maxWidth: `${moderationDrawerWidth}px` }"
		>
			<Transition name="moderation-drawer" appear>
				<aside
					v-show="open"
					:id="settingsNavigation.moderationDrawerId"
					ref="panel"
					tabindex="-1"
					:aria-label="formatMessage(messages.drawerLabel)"
					class="absolute inset-0 flex flex-col overflow-hidden overscroll-y-contain border-0 border-l border-solid border-divider bg-surface-3 shadow-xl focus:outline-none"
				>
					<header
						class="flex shrink-0 items-center gap-2 border-0 border-b border-solid border-divider p-4"
					>
						<Tabs
							:value="view"
							:tabs="views"
							:aria-label="formatMessage(messages.viewsLabel)"
							@update:value="selectedView = $event === 'thread' ? 'thread' : 'moderation'"
						/>
						<div class="ml-auto flex shrink-0 items-center gap-2">
							<IconButton
								type="quiet"
								:label="formatMessage(pinned ? messages.unpinPanel : messages.pinPanel)"
								:aria-pressed="pinned"
								:class="{ '!bg-bg-green !text-green': pinned }"
								@click="pinned = !pinned"
							>
								<PinIcon />
							</IconButton>
							<IconButton type="quiet" :label="formatMessage(messages.closePanel)" @click="close">
								<XIcon />
							</IconButton>
						</div>
					</header>
					<ProjectPublishingBanner
						v-if="view === 'moderation'"
						compact
						:state="publishingState"
						:remaining="remaining"
						:completed="completedItems.length"
						class="mb-1.5 px-5 py-3 pb-1.5"
					/>
					<OverlayScrollArea
						v-show="view === 'moderation'"
						scroll-shadows
						class="min-w-0 flex-1 [&_[data-overlayscrollbars-viewport]]:overscroll-y-contain"
					>
						<div class="min-w-0 p-3 px-2.5 [overflow-wrap:anywhere]">
							<ChecklistAccordion
								v-for="group in groups.filter(
									(group) => group.items.length || (group.id === 'todo' && issueItems.length),
								)"
								:key="group.id"
								open-by-default
								class="mb-4 last:mb-0"
								button-class="group/checklist-heading m-0 flex w-full cursor-pointer items-center gap-1.5 border-0 bg-transparent px-2.5 py-2 text-xs font-semibold uppercase tracking-wide text-primary"
							>
								<template #button="{ open: expanded }">
									<span
										class="transition-[filter] duration-150 group-hover/checklist-heading:brightness-125 motion-reduce:transition-none"
										>{{ group.title }}</span
									>
									<TagItem
										as="span"
										class="relative bottom-px !bg-surface-4 !px-1.5 !py-px !text-xs"
									>
										{{ group.items.length + (group.id === 'todo' ? issueItems.length : 0) }}
									</TagItem>
									<DropdownIcon
										class="ml-auto size-5 transition-[transform,filter] duration-300 group-hover/checklist-heading:brightness-125 motion-reduce:transition-none"
										:class="{ 'rotate-180': expanded }"
									/>
								</template>
								<template v-if="group.id === 'todo'">
									<div
										v-for="issue in issueItems"
										:key="issue.id"
										class="group relative flex min-w-0 items-start gap-1.5 rounded-lg px-2.5 py-3 transition-colors"
										:class="{ 'hover:bg-surface-4': issue.to }"
									>
										<MessageSquareWarningIcon
											class="mt-0.5 size-5 shrink-0 text-orange"
											aria-hidden="true"
										/>
										<div class="min-w-0 flex-1">
											<p class="m-0 font-semibold text-contrast">
												{{ issue.title }}
											</p>
											<p class="mb-0 mt-0.5 text-sm text-orange">
												{{ formatMessage(messages.requestedByModerator) }}
											</p>
											<IssueMessage
												v-if="issue.description"
												:message="issue.description"
												class="mt-1 text-base text-secondary [&_a]:relative [&_a]:z-10 [&_button]:relative [&_button]:z-10 [&_p]:!mb-2 [&_ul]:!mb-2"
											/>
											<div v-if="!issue.to" class="mt-3 flex justify-end">
												<Button
													size="sm"
													:disabled="!!addressingIssueId || !addressableFacets(issue.source).length"
													:loading="addressingIssueId === issue.id"
													@click="markIssueAddressed(issue.source)"
													>{{ formatMessage(messages.markAddressed) }}</Button
												>
											</div>
										</div>
										<ChevronRightIcon
											v-if="issue.to"
											class="mt-0.5 size-5 shrink-0 text-secondary opacity-0 transition-opacity group-hover:opacity-100"
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
												:aria-label="formatMessage(messages.viewIssue, { title: issue.title })"
												@click="onItemClick($event, destination?.fullPath, issue.highlightField)"
											/>
										</NuxtLink>
									</div>
								</template>
								<div
									v-for="nag in group.items"
									:key="nag.id"
									class="group relative flex items-start gap-2 rounded-lg p-3 px-2.5 transition-colors hover:bg-surface-4"
									:class="nag.isRequiredValidation ? 'text-red' : group.color"
								>
									<CircleDashedIcon
										v-if="nag.severity === 'suggestion'"
										aria-hidden="true"
										class="mt-0.5 size-5 shrink-0 text-secondary"
									/>
									<component
										:is="nag.isRequiredValidation ? XCircleIcon : group.icon"
										v-else
										class="mt-0.5 size-5 shrink-0"
									/>
									<div class="min-w-0 flex-1">
										<p class="m-0 font-semibold text-contrast">
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
											class="mb-0 mt-0.5 text-sm"
										>
											{{ formatMessage(messages.automaticCheck) }}
										</p>
										<p
											v-if="typeof nag.description !== 'function'"
											class="mb-0 mt-1 text-sm leading-relaxed text-secondary"
										>
											{{
												typeof nag.description === 'string'
													? nag.description
													: formatMessage(nag.description, nag.values)
											}}
										</p>
									</div>
									<ChevronRightIcon
										class="mt-0.5 size-5 shrink-0 text-secondary opacity-0 transition-opacity group-hover:opacity-100"
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
											@click="onItemClick($event, destination?.fullPath, nag.highlightField)"
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
									<TagItem as="span" class="relative bottom-px !bg-surface-4 !py-px !text-xs">
										{{ completedItems.length }}
									</TagItem>
									<DropdownIcon
										aria-hidden="true"
										class="ml-auto size-5 transition-transform duration-300 motion-reduce:transition-none"
										:class="{ 'rotate-180': expanded }"
									/>
								</template>
								<ul class="m-0 list-none p-0">
									<li v-for="item in completedItems" :key="item.id">
										<NuxtLink
											v-slot="{ href, route: destination }"
											:to="item.to ?? threadDestination"
											custom
										>
											<a
												:href="href"
												class="no-click-animation group flex items-center gap-3 rounded-lg px-2.5 py-2.5 text-sm font-semibold text-secondary no-underline hover:bg-surface-4 focus-visible:bg-surface-4 focus-visible:outline focus-visible:outline-brand active:scale-95 motion-safe:transition-[background-color,scale]"
												@click="onItemClick($event, destination.fullPath, item.highlightField)"
											>
												<span
													class="flex size-[18px] shrink-0 items-center justify-center rounded-full bg-green text-[var(--color-accent-contrast)]"
													aria-hidden="true"
												>
													<CheckIcon class="size-3" />
												</span>
												<span class="min-w-0 flex-1">{{ item.title }}</span>
												<RightArrowIcon
													aria-hidden="true"
													class="size-5 shrink-0 opacity-0 group-hover:opacity-100 group-focus-visible:opacity-100"
												/>
											</a>
										</NuxtLink>
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
								!canSubmit && remaining ? formatMessage(messages.completeRequired) : undefined
							"
						>
							<Button
								:type="canSubmit ? 'colored' : 'base'"
								class="w-full"
								:disabled="!canSubmit"
								@click="setProcessing"
								><SendIcon />{{
									formatMessage(
										rejected ? publishingActionMessages.resubmit : publishingActionMessages.submit,
									)
								}}</Button
							>
						</div>
					</div>
				</aside>
			</Transition>
		</div>
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
