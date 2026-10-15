<template>
	<div
		v-if="matchingIssues.length && (!threadHistory || isStaff(auth.user))"
		:id="fieldAnchor"
		class="flex flex-col gap-2"
	>
		<div
			v-for="issue in matchingIssues"
			:key="issue.id"
			class="min-w-0 rounded-2xl border border-solid"
			:class="[
				threadHistory ? 'pl-2.5 pr-1.5' : 'px-4 pb-1',
				threadHistory ? 'text-primary' : 'text-contrast',
				threadHistory
					? 'border-surface-5 bg-surface-2'
					: isComplete(issue)
						? 'border-brand-green bg-bg-green'
						: 'border-brand-orange bg-bg-orange',
			]"
			@click.self="toggle(issue)"
		>
			<div
				class="flex cursor-pointer flex-wrap items-start justify-between gap-1"
				:class="threadHistory ? 'py-3' : 'pb-3 pt-4'"
				role="button"
				tabindex="0"
				:aria-expanded="isExpanded(issue)"
				:aria-controls="issueContentId(issue)"
				@click="toggle(issue)"
				@keydown.enter.self.prevent="toggle(issue)"
				@keydown.space.self.prevent="toggle(issue)"
			>
				<h2
					class="m-0 flex min-w-0 flex-1 basis-[150px] flex-wrap items-start text-inherit"
					:class="threadHistory ? 'gap-1.5' : '!mt-[3px] gap-2'"
				>
					<CheckCircleIcon
						v-if="isComplete(issue)"
						class="!mt-0.5 shrink-0 text-brand-green"
						:class="threadHistory ? 'size-4' : 'size-5'"
						aria-hidden="true"
					/>
					<MessageSquareWarningIcon
						v-else
						class="!mt-0.5 shrink-0 text-brand-orange"
						:class="threadHistory ? 'size-4' : 'size-5'"
						aria-hidden="true"
					/>
					<span
						class="min-w-0 flex-1 [overflow-wrap:anywhere]"
						:class="threadHistory ? 'text-sm font-medium' : 'text-base font-semibold'"
						>{{ issueTitle(issue) }}</span
					>
				</h2>
				<DropdownIcon
					class="mb-0.5 ml-auto mr-1.5 size-5 shrink-0 self-center transition-transform duration-300"
					:class="{ 'rotate-180': isExpanded(issue) }"
					aria-hidden="true"
				/>
				<div v-if="isStaff(auth.user)" class="shrink-0 cursor-default" @click.stop>
					<TeleportOverflowMenu
						:label="formatMessage(messages.issueOptions)"
						:options="moderatorOptions(issue)"
						:disabled="verifyMutation.isPending.value"
						class="!text-inherit"
						size="sm"
						:class="threadHistory ? '!-my-1.5' : '!-my-1'"
						type="quiet"
					>
						<MoreHorizontalIcon aria-hidden="true" />
					</TeleportOverflowMenu>
				</div>
			</div>
			<div
				:id="issueContentId(issue)"
				class="project-issue-content"
				:class="{ open: isExpanded(issue), expanded: isFullyExpanded(issue) }"
				@transitionend.self="onContentTransitionEnd($event, issue)"
			>
				<div :inert="!isExpanded(issue)" class="min-w-0">
					<div class="flex flex-col gap-3 pb-3">
						<IssueMessage
							v-if="issueMessage(issue)"
							class="markdown-body min-w-0 text-sm text-inherit"
							:message="issueMessage(issue)"
						/>
						<div
							v-if="
								!threadHistory &&
								!isComplete(issue) &&
								((showProjectAreaLink && issueActions(issue).length) || showAddressActions(issue))
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
									type="colored"
									color="orange"
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
									type="colored"
									color="orange"
									:icon-only="false"
									:tooltip="
										issueActions(issue)
											.slice(2)
											.map((action) => action.label)
											.join(', ')
									"
								>
									+{{ issueActions(issue).length - 2 }}
									<template v-for="option in overflowOptions(issue)" :key="option.id" #[option.id]>
										<span class="flex-1">{{ option.label }}</span>
										<component :is="option.icon" aria-hidden="true" />
									</template>
								</TeleportOverflowMenu>
							</div>
							<div
								v-if="showAddressActions(issue)"
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
									:disabled="facetsToAddress(issue).length > 0 && !isFieldAddressed(issue)"
									:text="
										isFieldAddressed(issue)
											? formatMessage(messages.addressedProgressTooltip, {
													completed: issue.facets.filter((facet) => facet.verdict !== 'open')
														.length,
													total: issue.facets.length,
												})
											: formatMessage(messages.changeFacetFirst)
									"
								>
									<TagItem
										v-if="isFieldAddressed(issue)"
										class="!border-surface-4 !bg-surface-3 !px-2.5 !py-1 !text-sm !font-medium !text-primary"
									>
										{{ addressButtonLabel(issue) }}
									</TagItem>
									<Button
										v-else-if="!isComplete(issue)"
										:loading="
											addressMutation.isPending.value &&
											addressMutation.variables.value?.issueId === issue.id
										"
										:disabled="
											addressMutation.isPending.value ||
											(!canAddress && !isStaff(auth.user)) ||
											!facetsToAddress(issue).length ||
											(fieldAction !== undefined && !fieldAction.canSave)
										"
										:type="isComplete(issue) ? 'base' : 'colored'"
										color="orange"
										@click="
											addressMutation.mutate({
												issueId: issue.id,
												facets: facetsToAddress(issue),
												saveField: fieldAction?.save,
												threadId: thread?.id,
												projectId: project.id,
											})
										"
									>
										<CheckIcon class="size-4" aria-hidden="true" />
										{{ addressButtonLabel(issue) }}
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
import { type Labrinth, ModrinthServerError } from '@modrinth/api-client'
import {
	CheckCircleIcon,
	CheckIcon,
	ChevronRightIcon,
	CircleIcon,
	DropdownIcon,
	MessageSquareWarningIcon,
	MoreHorizontalIcon,
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
	type ButtonMenuLink,
	type ButtonMenuOption,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	TagItem,
	TeleportOverflowMenu,
	Tooltip,
	useVIntl,
} from '@modrinth/ui'
import { isStaff } from '@modrinth/utils'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, reactive, watch } from 'vue'

import type { ProjectIssueFieldAction } from '~/composables/project-issue-field-action'
import {
	isThreadIssueFacetReadyToAddress,
	isThreadIssueVerified,
	threadIssueVerdict,
} from '~/helpers/thread-issues'

import FilledCheckIcon from './filled-check-icon.vue'
import IssueMessage from './issue-message.vue'
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
	defaultCollapsed?: boolean
	fieldAction?: ProjectIssueFieldAction
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
const disclosuresQuery = useQuery({
	queryKey: computed(
		() => ['project', 'disclosures', 'v3', projectContext?.projectV2.value.id] as const,
	),
	queryFn: () => client.labrinth.projects_v3.getDisclosures(project.value.id),
	enabled: computed(
		() =>
			!!projectContext &&
			(props.issues ?? thread.value?.issues ?? []).some((issue) =>
				issue.facets.some(({ what }) =>
					[
						'add_project_disclosures',
						'remove_project_disclosures',
						'modify_project_disclosure',
						'modify_project_disclosure_note',
					].includes(what.type),
				),
			),
	),
})
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const auth = useAuthState()
const canAddress = computed(() => !!currentMember.value?.accepted)
const expandedIssues = reactive(new Map<string, boolean>())
const finishedExpanding = reactive(new Set<string>())
const messages = defineMessages({
	issueOptions: {
		id: 'thread-issues.options',
		defaultMessage: 'Issue options',
	},
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
	markAddressed: {
		id: 'thread-issues.mark-addressed',
		defaultMessage: 'Mark as addressed',
	},
	markAddressedProgress: {
		id: 'thread-issues.mark-addressed-progress',
		defaultMessage: 'Mark as addressed {completed, number}/{total, number}',
	},
	addressedProgress: {
		id: 'thread-issues.addressed-progress',
		defaultMessage: '{completed, number} of {total, number} addressed',
	},
	addressedProgressTooltip: {
		id: 'thread-issues.addressed-progress-tooltip',
		defaultMessage: '{completed, number}/{total, number} parts have been addressed for this issue',
	},
	addressFailed: {
		id: 'thread-issues.address-failed',
		defaultMessage: 'Failed to mark issue as addressed',
	},
	addressSaveFailed: {
		id: 'thread-issues.address-save-failed',
		defaultMessage:
			'Your changes could not be saved. Fix the validation errors before marking this issue as addressed.',
	},
	addressRequirementsUnchanged: {
		id: 'thread-issues.address-requirements-unchanged',
		defaultMessage: 'Make the requested changes before marking this issue as addressed.',
	},
	addressed: {
		id: 'thread-issues.addressed',
		defaultMessage: 'Marked as addressed',
	},
	showAddressed: {
		id: 'thread-issues.show-addressed',
		defaultMessage: 'Show addressed',
	},
	hideAddressed: {
		id: 'thread-issues.hide-addressed',
		defaultMessage: 'Hide addressed',
	},
	resolved: { id: 'thread-issues.resolved', defaultMessage: 'Resolved' },
	showResolved: {
		id: 'thread-issues.show-resolved',
		defaultMessage: 'Show resolved',
	},
	hideResolved: {
		id: 'thread-issues.hide-resolved',
		defaultMessage: 'Hide resolved',
	},
	editName: {
		id: 'thread-issues.target.edit-name',
		defaultMessage: 'Edit name',
	},
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
		facets,
		saveField,
	}: {
		issueId: string
		facets: Labrinth.Threads.v3.ThreadIssueFacet[]
		saveField?: () => Promise<boolean>
		threadId?: string
		projectId: string
	}) => {
		if (saveField && !(await saveField())) {
			throw new Error(formatMessage(messages.addressSaveFailed))
		}
		const results = await Promise.allSettled(
			facets.map(({ id }) => client.labrinth.threads_v3.user_addressed(id)),
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
			text:
				error instanceof ModrinthServerError &&
				error.message ===
					'moderation issue requirements must be changed before addressing the issue'
					? formatMessage(messages.addressRequirementsUnchanged)
					: error instanceof Error
						? error.message
						: String(error),
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
				client.labrinth.threads_v3.editIssueFacet(id, {
					moderator_verified: verified,
				}),
			),
		)
		const failure = results.find((result) => result.status === 'rejected')
		if (failure?.status === 'rejected') throw failure.reason
	},
	onSettled: async (_, __, { threadId, projectId }) => {
		emit('update-thread')
		await Promise.all([
			queryClient.invalidateQueries({
				queryKey: threadId ? ['thread', threadId] : ['thread'],
			}),
			...(projectId
				? [
						queryClient.invalidateQueries({
							queryKey: ['project', 'v2', projectId],
						}),
						queryClient.invalidateQueries({
							queryKey: ['project', 'v3', projectId],
						}),
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
	const resolved = threadIssueVerdict(issue) === 'resolved' || isThreadIssueVerified(issue)
	return [
		{
			id: 'moderator-verified',
			label: formatMessage(resolved ? messages.markUnresolved : messages.markVerified),
			icon: resolved ? XCircleIcon : CheckCircleIcon,
			tone: 'orange',
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
	if (!props.target || hasItemSelector()) return undefined
	const targets = Array.isArray(props.target) ? props.target : [props.target]
	const fields = new Set(targets.map((type) => threadIssueField({ type })))
	const [field] = fields
	return field && fields.size === 1 ? `project-issue-field-${field}` : undefined
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
	if (
		(what.type === 'add_project_disclosures' || what.type === 'remove_project_disclosures') &&
		props.disclosureType
	)
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
		node.type === 'section' ? panelIssues(node.children) : node.issue ? [node.issue] : [],
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

watch(
	() => matchingIssues.value.map((issue) => [issue.id, isComplete(issue)] as const),
	(issues, previousIssues) => {
		if (props.threadHistory) return
		const previousStates = new Map(previousIssues)
		for (const [id, complete] of issues) {
			if (previousStates.get(id) !== complete) {
				expandedIssues.delete(id)
				finishedExpanding.delete(id)
			}
		}
	},
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

function showAddressActions(issue: ThreadIssue): boolean {
	return (
		(!props.showProjectAreaLink || !readIssueLocations(issueDetails(issue).locations).length) &&
		addressableFacets(issue).length > 0
	)
}

function addressableFacets(issue: ThreadIssue): Labrinth.Threads.v3.ThreadIssueFacet[] {
	return issue.facets.filter(
		(facet) =>
			(issue.facets.length > 1 || facet.verdict !== 'resolved') &&
			(facet.what.type === 'acknowledge' ||
				facet.what.type === 'mark_addressed' ||
				matchesTarget(facet.what)),
	)
}

function isFieldAddressed(issue: ThreadIssue): boolean {
	return addressableFacets(issue).every((facet) => facet.verdict !== 'open')
}

function addressButtonLabel(issue: ThreadIssue): string {
	const facets = addressableFacets(issue)
	if (issue.facets.length > 1) {
		return formatMessage(
			isFieldAddressed(issue) ? messages.addressedProgress : messages.markAddressedProgress,
			{
				completed: issue.facets.filter((facet) => facet.verdict !== 'open').length,
				total: issue.facets.length,
			},
		)
	}
	return formatMessage(
		isFieldAddressed(issue)
			? messages.addressed
			: facets.every(
						(facet) => facet.what.type === 'acknowledge' && facet.what.value.mode === 'checkbox',
				  )
				? issueTargetLabels.acknowledge
				: messages.markAddressed,
	)
}

function facetsToAddress(issue: ThreadIssue): Labrinth.Threads.v3.ThreadIssueFacet[] {
	return addressableFacets(issue).filter(
		(facet) =>
			facet.verdict === 'open' &&
			isFacetReadyToAddress(
				facet,
				props.fieldAction ? { ...projectV3.value, ...props.fieldAction.draft } : projectV3.value,
			),
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

function isFacetReadyToAddress(
	facet: Labrinth.Threads.v3.ThreadIssueFacet,
	current: Labrinth.Projects.v3.Project,
): boolean {
	if (facet.what.type === 'modify_links' && props.platform) {
		const platforms = Array.isArray(props.platform) ? props.platform : [props.platform]
		facet = {
			...facet,
			what: {
				...facet.what,
				value: {
					links: Object.fromEntries(
						Object.entries(facet.what.value.links).filter(([platform]) =>
							platforms.includes(platform),
						),
					),
				},
			},
		}
	}
	return isThreadIssueFacetReadyToAddress(
		facet,
		current,
		allMembers.value,
		props.fieldAction?.disclosures ??
			(disclosuresQuery.isSuccess.value ? disclosuresQuery.data.value?.disclosures : undefined),
	)
}

function overflowOptions(issue: ThreadIssue): ButtonMenuLink[] {
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
	return threadIssueVerdict(issue) === 'addressed'
}

function isComplete(issue: ThreadIssue): boolean {
	return isAddressed(issue) || threadIssueVerdict(issue) === 'resolved'
}

function issueContentId(issue: ThreadIssue): string {
	return `${props.threadHistory ? 'thread' : 'project'}-issue-${issue.id}`
}

function isExpanded(issue: ThreadIssue): boolean {
	return (
		expandedIssues.get(issue.id) ??
		(!props.threadHistory && !props.defaultCollapsed && !isComplete(issue))
	)
}

function isFullyExpanded(issue: ThreadIssue): boolean {
	return isExpanded(issue) && (!expandedIssues.has(issue.id) || finishedExpanding.has(issue.id))
}

function onContentTransitionEnd(event: TransitionEvent, issue: ThreadIssue) {
	if (event.propertyName === 'grid-template-rows' && isExpanded(issue)) {
		finishedExpanding.add(issue.id)
	}
}

function toggle(issue: ThreadIssue) {
	finishedExpanding.delete(issue.id)
	expandedIssues.set(issue.id, !isExpanded(issue))
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
.markdown-body :deep(p) {
	margin-bottom: 12px;
}
</style>
