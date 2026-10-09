<template>
	<div
		v-if="matchingIssues.length && (!threadHistory || isStaff(auth.user))"
		:id="fieldAnchor"
		class="flex flex-col gap-2"
	>
		<div
			v-for="issue in matchingIssues"
			:key="issue.id"
			class="min-w-0 rounded-2xl border border-solid border-surface-5 bg-surface-2"
			:class="threadHistory ? 'p-2.5 py-3 pr-1.5' : 'p-4'"
		>
			<div class="flex flex-wrap items-start justify-between gap-1">
				<h2
					class="m-0 flex min-w-0 flex-1 basis-[150px] flex-wrap items-start text-contrast"
					:class="threadHistory ? 'gap-1.5' : '!mt-[3px] gap-2'"
				>
					<CheckCircleIcon
						v-if="isComplete(issue)"
						class="!mt-0.5 shrink-0 text-primary"
						:class="threadHistory ? 'size-4' : 'size-5'"
						aria-hidden="true"
					/>
					<TriangleAlertIcon
						v-else
						class="!mt-0.5 shrink-0 text-red"
						:class="threadHistory ? 'size-4' : 'size-5'"
						aria-hidden="true"
					/>
					<span
						class="min-w-0 flex-1 [overflow-wrap:anywhere]"
						:class="threadHistory ? 'text-sm font-medium' : 'text-base font-semibold'"
						>{{ issueTitle(issue) }}</span
					>
				</h2>
				<Button
					v-if="threadHistory || isComplete(issue)"
					class="ml-auto max-w-full shrink-0 !whitespace-normal"
					:class="threadHistory ? '-my-1.5 aspect-square' : ''"
					:circular="threadHistory"
					type="quiet"
					size="sm"
					:aria-expanded="isExpanded(issue)"
					:aria-controls="issueContentId(issue)"
					@click="toggle(issue.id)"
				>
					<FoldVerticalIcon v-if="isExpanded(issue)" aria-hidden="true" />
					<UnfoldVerticalIcon v-else aria-hidden="true" />
					<template v-if="!threadHistory">
						{{
							formatMessage(
								issue.verdict === 'resolved'
									? isExpanded(issue)
										? messages.hideResolved
										: messages.showResolved
									: isExpanded(issue)
										? messages.hideAddressed
										: messages.showAddressed,
							)
						}}
					</template>
				</Button>
				<TeleportOverflowMenu
					v-if="isStaff(auth.user)"
					:label="formatMessage(messages.issueOptions)"
					:options="moderatorOptions(issue)"
					:disabled="verifyMutation.isPending.value"
					class="shrink-0"
					:class="threadHistory ? '-my-1.5' : ''"
					type="quiet"
					size="sm"
				>
					<MoreHorizontalIcon aria-hidden="true" />
					<template #moderator-verified="{ option }">
						<component :is="option.icon" class="text-primary" aria-hidden="true" />
						{{ option.label }}
					</template>
				</TeleportOverflowMenu>
			</div>
			<div
				:id="issueContentId(issue)"
				class="project-issue-content"
				:class="{ open: isExpanded(issue), expanded: isFullyExpanded(issue) }"
				@transitionend.self="onContentTransitionEnd($event, issue)"
			>
				<div :inert="!isExpanded(issue)" class="min-w-0">
					<div class="flex flex-col gap-3 pt-2">
						<div
							v-if="issueMessage(issue)"
							class="markdown-body min-w-0 text-sm text-primary"
							v-html="renderString(issueMessage(issue))"
						/>
						<div
							v-if="
								!threadHistory &&
								issue.verdict !== 'resolved' &&
								((showProjectAreaLink && issueActions(issue).length) ||
									addressableFacets(issue).length)
							"
							class="flex w-full flex-wrap items-center justify-between gap-3"
						>
							<div
								v-if="showProjectAreaLink && issueActions(issue).length"
								class="flex flex-wrap gap-2"
							>
								<ButtonLink
									v-for="action in issueActions(issue).slice(0, 2)"
									:key="action.id"
									type="outlined"
									:to="action.to"
								>
									<CheckIcon v-if="action.complete === true" aria-hidden="true" />
									<CircleIcon
										v-else-if="action.complete === false"
										class="size-4"
										aria-hidden="true"
									/>
									{{ action.label }}
									<ChevronRightIcon
										v-if="action.complete === undefined"
										class="size-3"
										aria-hidden="true"
									/>
								</ButtonLink>
								<TeleportOverflowMenu
									v-if="issueActions(issue).length > 2"
									:label="formatMessage(messages.moreActions)"
									:options="overflowOptions(issue)"
									type="outlined"
									:icon-only="false"
									:tooltip="
										issueActions(issue)
											.slice(2)
											.map((action) => action.label)
											.join(', ')
									"
								>
									+{{ issueActions(issue).length - 2 }}
								</TeleportOverflowMenu>
							</div>
							<div
								v-if="addressableFacets(issue).length"
								class="ml-auto flex flex-wrap justify-end gap-2"
							>
								<ButtonLink
									v-if="replyFacets(issue).length"
									:to="replyLink(issue)"
									type="outlined"
									:disabled="!canAddress && !isStaff(auth.user)"
								>
									{{ formatMessage(messages.replyToAddress) }}
								</ButtonLink>
								<Tooltip
									v-else
									:disabled="facetsToAddress(issue).length > 0 || isAddressed(issue)"
									:text="formatMessage(messages.changeFacetFirst)"
								>
									<Button
										:loading="
											addressMutation.isPending.value &&
											addressMutation.variables.value?.issueId === issue.id
										"
										:disabled="
											addressMutation.isPending.value ||
											(!canAddress && !isStaff(auth.user)) ||
											!facetsToAddress(issue).length
										"
										@click="
											addressMutation.mutate({
												issueId: issue.id,
												facetIds: facetsToAddress(issue).map(({ id }) => id),
												threadId: thread?.id,
												projectId: project.id,
											})
										"
									>
										<CheckIcon class="size-4" aria-hidden="true" />
										{{
											formatMessage(
												addressableFacets(issue).every((facet) => facet.verdict !== 'open')
													? messages.addressed
													: addressableFacets(issue).every(
																(facet) =>
																	facet.what.type === 'acknowledge' &&
																	facet.what.value.mode === 'checkbox',
														  )
														? issueTargetLabels.acknowledge
														: messages.markAddressed,
											)
										}}
									</Button>
								</Tooltip>
							</div>
						</div>
					</div>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	CheckCircleIcon,
	CheckIcon,
	ChevronRightIcon,
	CircleIcon,
	FoldVerticalIcon,
	MoreHorizontalIcon,
	TriangleAlertIcon,
	UnfoldVerticalIcon,
	XCircleIcon,
} from '@modrinth/assets'
import { IssuePriority, reviewPanels } from '@modrinth/moderation/src/data/issues'
import {
	issueLocationRegistry,
	readIssueLocations,
} from '@modrinth/moderation/src/data/issues/component-builders/locations'
import { issueTargetLabels } from '@modrinth/moderation/src/data/issues/component-builders/targets'
import type {
	Issue,
	PanelNode,
} from '@modrinth/moderation/src/data/issues/component-builders/types'
import {
	Button,
	ButtonLink,
	type ButtonMenuOption,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	TeleportOverflowMenu,
	Tooltip,
	useVIntl,
} from '@modrinth/ui'
import { isStaff, renderString } from '@modrinth/utils'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { computed, reactive } from 'vue'

import { isThreadIssueFacetReadyToAddress, isThreadIssueVerified } from '~/helpers/thread-issues'

import FilledCheckIcon from './filled-check-icon.vue'
import { threadIssueField, threadIssueSettingsArea } from './issue-targets'

type ThreadIssue = Labrinth.Threads.v3.ThreadIssue
type Target = Labrinth.Threads.v3.ThreadIssueTarget
const issueHeadingPattern = /^##[ \t]+([^\r\n]*)(?:\r?\n)?/m

const props = defineProps<{
	target?: Target['type'] | Target['type'][]
	location?: Labrinth.Threads.v3.ThreadIssueLocation['field']
	issues?: ThreadIssue[]
	showProjectAreaLink?: boolean
	platform?: string | string[]
	versionId?: string
	imageId?: number
	userId?: string
	teamId?: string
	disclosureType?: string
	threadHistory?: boolean
}>()

const emit = defineEmits<{
	'update-thread': []
}>()

const projectContext = props.threadHistory ? null : injectProjectPageContext()
const thread = computed(() => projectContext?.thread.value)
const currentMember = computed(() => projectContext?.currentMember.value)
const project = computed(() => requireProjectContext().projectV2.value)
const projectV3 = computed(() => requireProjectContext().projectV3.value)
const allMembers = computed(() => requireProjectContext().allMembers.value)

function requireProjectContext() {
	if (!projectContext) throw new Error('Project issue actions require project-page context')
	return projectContext
}

const client = injectModrinthClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const auth = useAuthState()
const canAddress = computed(() => !!currentMember.value?.accepted)
const expandedAddressed = reactive(new Set<string>())
const finishedExpanding = reactive(new Set<string>())
const messages = defineMessages({
	issueOptions: { id: 'thread-issues.options', defaultMessage: 'Issue options' },
	markVerified: {
		id: 'thread-issues.mark-resolved',
		defaultMessage: 'Mark as resolved',
	},
	markUnresolved: {
		id: 'thread-issues.mark-unresolved',
		defaultMessage: 'Mark as unresolved',
	},
	verifyFailed: {
		id: 'thread-issues.resolve-failed',
		defaultMessage: 'Failed to mark issue as resolved',
	},
	unresolveFailed: {
		id: 'thread-issues.unresolve-failed',
		defaultMessage: 'Failed to mark issue as unresolved',
	},
	showIssue: { id: 'thread-issues.show-issue', defaultMessage: 'Show issue' },
	hideIssue: { id: 'thread-issues.hide-issue', defaultMessage: 'Hide issue' },
	markAddressed: { id: 'thread-issues.mark-addressed', defaultMessage: 'Mark as addressed' },
	addressFailed: {
		id: 'thread-issues.address-failed',
		defaultMessage: 'Failed to mark issue as addressed',
	},
	addressed: { id: 'thread-issues.addressed', defaultMessage: 'Marked as addressed' },
	showAddressed: { id: 'thread-issues.show-addressed', defaultMessage: 'Show addressed' },
	hideAddressed: { id: 'thread-issues.hide-addressed', defaultMessage: 'Hide addressed' },
	resolved: { id: 'thread-issues.resolved', defaultMessage: 'Resolved' },
	showResolved: { id: 'thread-issues.show-resolved', defaultMessage: 'Show resolved' },
	hideResolved: { id: 'thread-issues.hide-resolved', defaultMessage: 'Hide resolved' },
	editName: { id: 'thread-issues.target.edit-name', defaultMessage: 'Edit name' },
	editUrl: { id: 'thread-issues.target.edit-url', defaultMessage: 'Edit URL' },
	editSummary: {
		id: 'thread-issues.target.edit-summary',
		defaultMessage: 'Edit summary',
	},
	moreActions: {
		id: 'thread-issues.more-actions',
		defaultMessage: 'More required actions',
	},
	changeFacetFirst: {
		id: 'thread-issues.change-facet-first',
		defaultMessage: 'Please make the requested changes before marking it as addressed.',
	},
	replyToAddress: {
		id: 'thread-issues.reply-to-address',
		defaultMessage: 'Reply to address',
	},
})

const addressMutation = useMutation({
	mutationFn: async ({
		facetIds,
	}: {
		issueId: string
		facetIds: string[]
		threadId?: string
		projectId: string
	}) => {
		const results = await Promise.allSettled(
			facetIds.map((id) => client.labrinth.threads_v3.user_addressed(id)),
		)
		const failure = results.find((result) => result.status === 'rejected')
		if (failure?.status === 'rejected') throw failure.reason
	},
	onSettled: async (_, __, { threadId, projectId }) => {
		await Promise.all([
			threadId
				? queryClient.invalidateQueries({ queryKey: ['thread', threadId] })
				: Promise.resolve(),
			queryClient.invalidateQueries({ queryKey: ['project', 'v2', projectId] }),
			queryClient.invalidateQueries({ queryKey: ['project', 'v3', projectId] }),
			project.value.id === projectId
				? requireProjectContext().refreshProjectValidation()
				: Promise.resolve(),
		])
	},
	onError: (error) =>
		addNotification({
			title: formatMessage(messages.addressFailed),
			text: error instanceof Error ? error.message : String(error),
			type: 'error',
		}),
})

const verifyMutation = useMutation({
	mutationFn: async ({
		facetIds,
		verified,
	}: {
		facetIds: string[]
		verified: boolean
		threadId?: string
		projectId?: string
	}) => {
		const results = await Promise.allSettled(
			facetIds.map((id) =>
				client.labrinth.threads_v3.editIssueFacet(id, { moderator_verified: verified }),
			),
		)
		const failure = results.find((result) => result.status === 'rejected')
		if (failure?.status === 'rejected') throw failure.reason
	},
	onSettled: async (_, __, { threadId, projectId }) => {
		emit('update-thread')
		await Promise.all([
			queryClient.invalidateQueries({ queryKey: threadId ? ['thread', threadId] : ['thread'] }),
			...(projectId
				? [
						queryClient.invalidateQueries({ queryKey: ['project', 'v2', projectId] }),
						queryClient.invalidateQueries({ queryKey: ['project', 'v3', projectId] }),
						projectContext?.projectV2.value.id === projectId
							? projectContext.refreshProjectValidation()
							: Promise.resolve(),
					]
				: []),
		])
	},
	onError: (error, { verified }) =>
		addNotification({
			title: formatMessage(verified ? messages.verifyFailed : messages.unresolveFailed),
			text: error instanceof Error ? error.message : String(error),
			type: 'error',
		}),
})

function moderatorOptions(issue: ThreadIssue): ButtonMenuOption[] {
	const resolved = issue.verdict === 'resolved' || isThreadIssueVerified(issue)
	return [
		{
			id: 'moderator-verified',
			label: formatMessage(resolved ? messages.markUnresolved : messages.markVerified),
			icon: resolved ? XCircleIcon : CheckCircleIcon,
			disabled: !issue.facets.length || verifyMutation.isPending.value,
			action: () => {
				if (!isStaff(auth.value.user) || verifyMutation.isPending.value) return
				verifyMutation.mutate({
					facetIds: issue.facets
						.filter((facet) => resolved || !facet.moderator_verified)
						.map(({ id }) => id),
					verified: !resolved,
					threadId: thread.value?.id,
					projectId: projectContext?.projectV2.value.id,
				})
			},
		},
	]
}

const fieldAnchor = computed(() => {
	if (typeof props.target !== 'string' || hasItemSelector()) return undefined
	const field = threadIssueField({ type: props.target })
	return field ? `project-issue-field-${field}` : undefined
})

function hasItemSelector(): boolean {
	return (
		props.platform !== undefined ||
		props.versionId !== undefined ||
		props.imageId !== undefined ||
		props.userId !== undefined ||
		props.teamId !== undefined ||
		props.disclosureType !== undefined
	)
}

function matchesLocation(issue: ThreadIssue): boolean {
	const locations = readIssueLocations(issueDetails(issue).locations)
	if (props.location) return locations.some(({ field }) => field === props.location)
	if (hasItemSelector()) return false
	const targets = Array.isArray(props.target) ? props.target : props.target ? [props.target] : []
	return locations.some(({ field }) =>
		targets.some(
			(type) => threadIssueField({ type }) === field && issueLocationRegistry[field].inline,
		),
	)
}

function matchesTarget(what: Target): boolean {
	const targets = Array.isArray(props.target) ? props.target : [props.target]
	if (!targets.includes(what.type)) return false
	const platforms = Array.isArray(props.platform) ? props.platform : [props.platform]
	if (what.type === 'modify_links' && props.platform)
		return platforms.some((platform) => platform && Object.hasOwn(what.value.links, platform))
	if (what.type === 'modify_server_address' && props.platform)
		return platforms.includes(what.value.platform)
	if (what.type === 'version' && props.versionId) return what.value.version_id === props.versionId
	if (what.type === 'modify_gallery_image' && props.imageId !== undefined)
		return what.value.image_id === props.imageId
	if (what.type === 'modify_team_member_role')
		return (
			(!props.userId || what.value.user_id === props.userId) &&
			(!props.teamId || what.value.team_id === props.teamId)
		)
	if (what.type === 'remove_project_disclosures' && props.disclosureType)
		return what.value.disclosure_types.includes(props.disclosureType)
	if (
		(what.type === 'modify_project_disclosure' || what.type === 'modify_project_disclosure_note') &&
		props.disclosureType
	)
		return what.value.disclosure_type === props.disclosureType
	return true
}

function panelIssues(nodes: readonly PanelNode[]): Issue[] {
	return nodes.flatMap((node) =>
		node.type === 'section' ? panelIssues(node.children) : [node.issue],
	)
}

const issuePriorities = new Map(
	Object.values(reviewPanels)
		.flatMap((registration) =>
			Array.isArray(registration) ? registration.map(({ panel }) => panel) : [registration],
		)
		.flatMap((panel) =>
			('type' in panel ? [] : panelIssues(panel.children)).map(
				(issue) => [issue.id, issue.priority ?? IssuePriority.Default] as const,
			),
		),
)

function issuePriority(issue: ThreadIssue) {
	const id = issueDetails(issue).issue_id
	return typeof id === 'string'
		? (issuePriorities.get(id) ?? IssuePriority.Default)
		: IssuePriority.Default
}

const matchingIssues = computed(() =>
	(props.issues ?? thread.value?.issues ?? [])
		.filter(
			(issue) =>
				(props.threadHistory || !isThreadIssueVerified(issue)) &&
				(props.issues !== undefined ||
					matchesLocation(issue) ||
					issue.facets.some(({ what }) => matchesTarget(what))),
		)
		.sort((a, b) => issuePriority(a) - issuePriority(b)),
)

function issueDetails(issue: ThreadIssue): Record<string, unknown> {
	return issue.why && typeof issue.why === 'object' && !Array.isArray(issue.why)
		? (issue.why as Record<string, unknown>)
		: {}
}

function issueTitle(issue: ThreadIssue): string {
	const why = issueDetails(issue)
	const heading =
		typeof why.message === 'string' ? why.message.match(issueHeadingPattern)?.[1].trim() : ''
	return (
		heading ||
		(typeof why.title === 'string'
			? why.title
			: typeof why.issue_id === 'string'
				? why.issue_id.replaceAll('-', ' ')
				: formatMessage(issueTargetLabels[issue.facets[0]?.what.type ?? 'mark_addressed']))
	)
}

function issueMessage(issue: ThreadIssue): string {
	const message = issueDetails(issue).message
	return typeof message === 'string' ? message.replace(issueHeadingPattern, '').trimStart() : ''
}

interface IssueAction {
	id: string
	to: string
	label: string
	complete?: boolean
}

function issueActions(issue: ThreadIssue): IssueAction[] {
	const locations = readIssueLocations(issueDetails(issue).locations)
	const facets = visibleFacets(issue).filter(
		({ what }) => what.type !== 'acknowledge' && what.type !== 'mark_addressed',
	)
	const facetFields = new Set(facets.map(({ what }) => threadIssueField(what)))
	const actions: IssueAction[] = facets.map((facet) => {
		const label = locations.find(({ field }) => field === threadIssueField(facet.what))?.label
		return {
			id: facet.id,
			to: settingsLink(facet.what),
			label: label ? formatMessage(label) : targetButtonLabel(facet.what),
			complete: facet.verdict !== 'open',
		}
	})
	for (const location of locations) {
		if (facetFields.has(location.field)) continue
		const { area, label } = issueLocationRegistry[location.field]
		const base = `/${project.value.project_type}/${project.value.id}/settings`
		actions.push({
			id: `location-${location.field}`,
			to: area ? `${base}/${area}` : `${base}#project-issue-field-${location.field}`,
			label: formatMessage(location.label ?? label),
		})
	}
	return actions.sort(
		(a, b) =>
			Number(a.complete === true) - Number(b.complete === true) || a.label.localeCompare(b.label),
	)
}

function visibleFacets(issue: ThreadIssue): Labrinth.Threads.v3.ThreadIssueFacet[] {
	if (props.issues !== undefined || props.showProjectAreaLink) return issue.facets
	return issue.facets.filter(
		({ what }) =>
			matchesTarget(what) ||
			((what.type === 'acknowledge' || what.type === 'mark_addressed') && matchesLocation(issue)) ||
			(props.location !== undefined && threadIssueField(what) === props.location),
	)
}

function addressableFacets(issue: ThreadIssue): Labrinth.Threads.v3.ThreadIssueFacet[] {
	return issue.facets.filter(
		(facet) =>
			facet.verdict !== 'resolved' &&
			(facet.what.type === 'acknowledge' ||
				facet.what.type === 'mark_addressed' ||
				matchesTarget(facet.what)),
	)
}

function facetsToAddress(issue: ThreadIssue): Labrinth.Threads.v3.ThreadIssueFacet[] {
	return addressableFacets(issue).filter(
		(facet) =>
			facet.verdict === 'open' && (isFacetReadyToAddress(facet) || isStaff(auth.value.user)),
	)
}

function replyFacets(issue: ThreadIssue): Labrinth.Threads.v3.ThreadIssueFacet[] {
	return issue.facets.filter(
		(facet) =>
			facet.verdict === 'open' &&
			facet.what.type === 'acknowledge' &&
			facet.what.value.mode === 'reply',
	)
}

function replyLink(issue: ThreadIssue): string {
	const query = new URLSearchParams()
	for (const facet of replyFacets(issue)) query.append('reply_to_facet', facet.id)
	return `/${project.value.project_type}/${project.value.slug ?? project.value.id}/moderation?${query}#messages`
}

function isFacetReadyToAddress(facet: Labrinth.Threads.v3.ThreadIssueFacet): boolean {
	return isThreadIssueFacetReadyToAddress(facet, projectV3.value, allMembers.value)
}

function overflowOptions(issue: ThreadIssue): ButtonMenuOption[] {
	return issueActions(issue)
		.slice(2)
		.map((action) => ({
			id: action.id,
			type: 'link',
			label: action.label,
			icon:
				action.complete === undefined
					? ChevronRightIcon
					: action.complete
						? FilledCheckIcon
						: CircleIcon,
			to: action.to,
		}))
}

function settingsLink(target: Target): string {
	const base = `/${project.value.project_type}/${project.value.id}/settings`
	const area = threadIssueSettingsArea(target)
	return area ? `${base}/${area}` : base
}

function targetButtonLabel(target: Target): string {
	switch (target.type) {
		case 'modify_title':
			return formatMessage(messages.editName)
		case 'modify_slug':
			return formatMessage(messages.editUrl)
		case 'modify_summary':
			return formatMessage(messages.editSummary)
		default:
			return formatMessage(issueTargetLabels[target.type])
	}
}

function isAddressed(issue: ThreadIssue): boolean {
	return issue.verdict === 'addressed'
}

function isComplete(issue: ThreadIssue): boolean {
	return isAddressed(issue) || issue.verdict === 'resolved'
}

function issueContentId(issue: ThreadIssue): string {
	return `${props.threadHistory ? 'thread' : 'project'}-issue-${issue.id}`
}

function isExpanded(issue: ThreadIssue): boolean {
	return (!props.threadHistory && !isComplete(issue)) || expandedAddressed.has(issue.id)
}

function isFullyExpanded(issue: ThreadIssue): boolean {
	return (!props.threadHistory && !isComplete(issue)) || finishedExpanding.has(issue.id)
}

function onContentTransitionEnd(event: TransitionEvent, issue: ThreadIssue) {
	if (event.propertyName === 'grid-template-rows' && isExpanded(issue)) {
		finishedExpanding.add(issue.id)
	}
}

function toggle(id: string) {
	finishedExpanding.delete(id)
	if (expandedAddressed.has(id)) expandedAddressed.delete(id)
	else expandedAddressed.add(id)
}
</script>

<style scoped>
.project-issue-content {
	display: grid;
	grid-template-rows: 0fr;
	transition: grid-template-rows 300ms ease-in-out;
}

.project-issue-content.open {
	grid-template-rows: 1fr;
}

.project-issue-content > div {
	overflow: hidden;
}

.project-issue-content.expanded > div {
	overflow: visible;
}

@media (prefers-reduced-motion: reduce) {
	.project-issue-content {
		transition: none;
	}

	.project-issue-content.open > div {
		overflow: visible;
	}
}
.markdown-body :deep(.review-card-image-targets) {
	display: flex;
	flex-direction: column;
	gap: 1rem;
}

.markdown-body :deep(.review-card-image-target) {
	display: flex;
	flex-direction: column;
	align-items: flex-start;
	gap: 0.5rem;
}

.markdown-body :deep(.review-card-image-heading) {
	font-weight: 600;
}

.markdown-body :deep(.review-card-image-list) {
	display: flex;
	flex-wrap: wrap;
	gap: 0.75rem;
	margin: 0;
	padding: 0;
	list-style: none;
}

.markdown-body :deep(.review-card-image-entry) {
	width: 140px;
	max-width: 100%;
	margin: 0;
	padding: 0;
	list-style: none;
}

.markdown-body :deep(.review-card-image-entry > a) {
	display: block;
}

.markdown-body :deep(.review-card-image-entry .review-card-gallery-image) {
	display: block;
	box-sizing: border-box;
	width: 100%;
	max-width: 140px;
	height: 112px;
	padding: 0.5rem;
	border-radius: 0.5rem;
	background: var(--surface-3);
	object-fit: contain;
}

.markdown-body :deep(.review-card-image-caption) {
	display: block;
	margin-top: 0.375rem;
	overflow-wrap: anywhere;
	font-size: 0.875em;
}

.markdown-body :deep(.review-card-image-target .review-card-image-description) {
	margin: 0;
}
</style>
