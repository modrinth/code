<template>
	<div v-if="matchingIssues.length" class="flex flex-col gap-2">
		<div
			v-for="issue in matchingIssues"
			:key="issue.id"
			class="min-w-0 rounded-2xl border border-solid border-surface-5 bg-surface-2 p-4"
		>
			<div class="flex items-center justify-between gap-3">
				<h2 class="m-0 flex min-w-0 items-center gap-2 text-base font-semibold text-contrast">
					<CheckCircleIcon
						v-if="isAddressed(issue)"
						class="size-5 shrink-0 text-primary"
						aria-hidden="true"
					/>
					<TriangleAlertIcon v-else class="size-5 shrink-0 text-red" aria-hidden="true" />
					<span class="min-w-0 break-words">{{ issueTitle(issue) }}</span>
				</h2>
				<Button
					v-if="isAddressed(issue)"
					type="quiet"
					size="sm"
					:aria-expanded="isExpanded(issue)"
					:aria-controls="`project-issue-${issue.id}`"
					@click="toggle(issue.id)"
				>
					<FoldVerticalIcon v-if="isExpanded(issue)" aria-hidden="true" />
					<UnfoldVerticalIcon v-else aria-hidden="true" />
					{{ formatMessage(isExpanded(issue) ? messages.hideAddressed : messages.showAddressed) }}
				</Button>
			</div>
			<div
				:id="`project-issue-${issue.id}`"
				class="project-issue-content"
				:class="{ open: isExpanded(issue), expanded: isFullyExpanded(issue) }"
				@transitionend.self="onContentTransitionEnd($event, issue)"
			>
				<div :inert="!isExpanded(issue)">
					<div class="flex flex-col gap-3 pt-3">
						<div
							v-if="issueMessage(issue)"
							class="markdown-body min-w-0 text-sm text-primary"
							v-html="renderString(issueMessage(issue))"
						/>
						<div class="flex w-full flex-wrap items-center justify-between gap-3">
							<div
								v-if="showProjectAreaLink && actionFacets(issue).length"
								class="flex flex-wrap gap-2"
							>
								<ButtonLink
									v-for="facet in actionFacets(issue).slice(0, 2)"
									:key="facet.id"
									type="outlined"
									:to="settingsLink(facet.what)"
								>
									<CheckIcon v-if="isFacetComplete(facet)" aria-hidden="true" />
									<CircleIcon v-else class="size-4" aria-hidden="true" />
									{{ targetButtonLabel(facet.what) }}
								</ButtonLink>
								<TeleportOverflowMenu
									v-if="actionFacets(issue).length > 2"
									:label="formatMessage(messages.moreActions)"
									:options="overflowOptions(issue)"
									type="outlined"
									:icon-only="false"
									:tooltip="
										actionFacets(issue)
											.slice(2)
											.map((facet) => targetButtonLabel(facet.what))
											.join(', ')
									"
								>
									+{{ actionFacets(issue).length - 2 }}
								</TeleportOverflowMenu>
							</div>
							<Tooltip
								v-if="!showProjectAreaLink && issue.verdict !== 'resolved'"
								:disabled="allActionsComplete(issue) || isAddressed(issue)"
								:text="formatMessage(messages.completeActionsFirst)"
								class="ml-auto"
							>
								<Button
									:loading="
										addressMutation.isPending.value && addressMutation.variables.value === issue.id
									"
									:disabled="
										isAddressed(issue) ||
										!allActionsComplete(issue) ||
										(!canAddress && !isStaff(auth.user)) ||
										addressMutation.isPending.value
									"
									@click="addressMutation.mutate(issue.id)"
								>
									<CheckIcon class="size-4" aria-hidden="true" />
									{{
										formatMessage(isAddressed(issue) ? messages.addressed : messages.markAddressed)
									}}
								</Button>
							</Tooltip>
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
	CircleIcon,
	FoldVerticalIcon,
	TriangleAlertIcon,
	UnfoldVerticalIcon,
} from '@modrinth/assets'
import { IssuePriority, reviewPanels } from '@modrinth/moderation/src/data/issues'
import { issueTargetLabels } from '@modrinth/moderation/src/data/issues/component-builders/targets'
import type { Issue, PanelNode } from '@modrinth/moderation/src/data/issues/component-builders/types'
import {
	Button,
	ButtonLink,
	type ButtonMenuOption,
	commonMessages,
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

import FilledCheckIcon from './filled-check-icon.vue'
import { threadIssueSettingsArea } from './issue-targets'

type ThreadIssue = Labrinth.Threads.v3.ThreadIssue
type Target = Labrinth.Threads.v3.ThreadIssueTarget
const issueHeadingPattern = /^##[ \t]+([^\r\n]*)(?:\r?\n)?/m

const props = defineProps<{
	target?: Target['type'] | Target['type'][]
	issues?: ThreadIssue[]
	showProjectAreaLink?: boolean
	platform?: string
	versionId?: string
	imageId?: number
	userId?: string
	teamId?: string
	disclosureType?: string
}>()

const {
	thread,
	currentMember,
	projectV2: project,
	projectV3,
	allMembers,
	refreshProjectValidation,
} = injectProjectPageContext()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const auth = useAuthState()
const canAddress = computed(() => !!currentMember.value?.accepted)
const expandedAddressed = reactive(new Set<string>())
const finishedExpanding = reactive(new Set<string>())
const messages = defineMessages({
	markAddressed: { id: 'thread-issues.mark-addressed', defaultMessage: 'Mark as addressed' },
	addressed: { id: 'thread-issues.addressed', defaultMessage: 'Marked as addressed' },
	showAddressed: { id: 'thread-issues.show-addressed', defaultMessage: 'Show addressed' },
	hideAddressed: { id: 'thread-issues.hide-addressed', defaultMessage: 'Hide addressed' },
	editName: { id: 'thread-issues.target.edit-name', defaultMessage: 'Edit name' },
	editUrl: { id: 'thread-issues.target.edit-url', defaultMessage: 'Edit URL' },
	editSummary: { id: 'thread-issues.target.edit-summary', defaultMessage: 'Edit summary' },
	moreActions: { id: 'thread-issues.more-actions', defaultMessage: 'More required actions' },
	completeActionsFirst: {
		id: 'thread-issues.complete-actions-first',
		defaultMessage:
			'You must complete all required actions from the issue to mark it as addressed.',
	},
})

const addressMutation = useMutation({
	mutationFn: async (id: string) => {
		await client.labrinth.threads_v3.user_addressed(id)
	},
	onSuccess: async () => {
		await Promise.all([
			thread.value?.id
				? queryClient.invalidateQueries({ queryKey: ['thread', thread.value.id] })
				: Promise.resolve(),
			refreshProjectValidation(),
		])
	},
	onError: (error) =>
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: error instanceof Error ? error.message : String(error),
			type: 'error',
		}),
})

function matchesTarget(what: Target): boolean {
	const targets = Array.isArray(props.target) ? props.target : [props.target]
	if (!targets.includes(what.type)) return false
	if (what.type === 'modify_links' && props.platform)
		return Object.hasOwn(what.value.links, props.platform)
	if (what.type === 'modify_server_address' && props.platform)
		return what.value.platform === props.platform
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
	Object.values(reviewPanels).flatMap((panel) =>
		panelIssues(panel.children).map(
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
				issue.verdict !== 'resolved' &&
				(props.issues !== undefined || issue.facets.some(({ what }) => matchesTarget(what))),
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
				: formatMessage(issueTargetLabels[issue.facets[0]?.what.type ?? 'acknowledge']))
	)
}

function issueMessage(issue: ThreadIssue): string {
	const message = issueDetails(issue).message
	return typeof message === 'string' ? message.replace(issueHeadingPattern, '').trimStart() : ''
}

function actionFacets(issue: ThreadIssue): ThreadIssue['facets'] {
	return issue.facets
		.filter(({ what }) => what.type !== 'acknowledge')
		.sort(
			(a, b) =>
				Number(isFacetComplete(a)) - Number(isFacetComplete(b)) ||
				targetButtonLabel(a.what).localeCompare(targetButtonLabel(b.what)),
		)
}

function isFacetComplete(facet: ThreadIssue['facets'][number]): boolean {
	if (facet.verdict !== 'open') return true
	const current = projectV3.value
	const target = facet.what
	if (!current) return facet.verdict !== 'open'
	switch (target.type) {
		case 'modify_title':
			return current.name !== target.value.original
		case 'modify_slug':
			return (current.slug ?? '') !== target.value.original
		case 'modify_summary':
			return current.summary !== target.value.original
		case 'modify_description':
			return current.description !== target.value.original
		case 'modify_license':
			return (
				current.license.id !== target.value.license.original ||
				(current.license.url ?? '') !== target.value.url.original
			)
		case 'modify_icon':
			return (current.icon_url ?? null) !== target.value.original_url
		case 'modify_links':
			return (
				Object.keys(target.value.links).length > 0 &&
				Object.entries(target.value.links).every(
					([platform, { original }]) => (current.link_urls[platform]?.url ?? '') !== original,
				)
			)
		case 'add_gallery_images':
			return current.gallery.length > target.value.original_count
		case 'remove_tags':
			return target.value.tags.every(
				(tag) => !current.categories.includes(tag) && !current.additional_categories.includes(tag),
			)
		case 'remove_gallery_images':
			return target.value.image_ids.every((id) => !current.gallery.some((image) => image.id === id))
		case 'modify_gallery_image': {
			const image = current.gallery.find(({ id }) => id === target.value.image_id)
			if (!image) return false
			return (
				(target.value.name !== undefined &&
					(image.name ?? '') !== (target.value.name?.original ?? '')) ||
				(target.value.description !== undefined &&
					(image.description ?? '') !== (target.value.description?.original ?? ''))
			)
		}
		case 'modify_team_member_role': {
			const member = allMembers.value.find(
				({ user, team_id }) => user.id === target.value.user_id && team_id === target.value.team_id,
			)
			return member ? member.role !== target.value.role.original : false
		}
		case 'modify_server_languages':
			return (
				JSON.stringify([...(current.minecraft_server?.languages ?? [])].sort()) !==
				JSON.stringify([...target.value.original].sort())
			)
		case 'modify_server_address':
			return (
				((target.value.platform === 'minecraft_java'
					? current.minecraft_java_server?.address
					: current.minecraft_bedrock_server?.address) ?? '') !== target.value.address.original
			)
		default:
			return facet.verdict !== 'open'
	}
}

function allActionsComplete(issue: ThreadIssue): boolean {
	return issue.facets.every((facet) => facet.what.type === 'acknowledge' || isFacetComplete(facet))
}

function overflowOptions(issue: ThreadIssue): ButtonMenuOption[] {
	return actionFacets(issue)
		.slice(2)
		.map((facet) => ({
			id: facet.id,
			type: 'link',
			label: targetButtonLabel(facet.what),
			icon: isFacetComplete(facet) ? FilledCheckIcon : CircleIcon,
			to: settingsLink(facet.what),
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
	return issue.user_addressed
}

function isExpanded(issue: ThreadIssue): boolean {
	return !isAddressed(issue) || expandedAddressed.has(issue.id)
}

function isFullyExpanded(issue: ThreadIssue): boolean {
	return !isAddressed(issue) || finishedExpanding.has(issue.id)
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
</style>
