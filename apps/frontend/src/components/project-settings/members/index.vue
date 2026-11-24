<script setup lang="ts">
import { UserPlusIcon, UserXIcon } from '@modrinth/assets'
import {
	Avatar,
	Button,
	commonProjectSettingsMessages,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	IntlFormatted,
	SettingsOptionCard,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { useDocumentVisibility, useIntervalFn } from '@vueuse/core'
import { computed, nextTick, onMounted, useTemplateRef, watch } from 'vue'

import ProjectIssueCard from '~/components/ui/project-issue-card/index.vue'
import { projectQueryOptions } from '~/composables/queries/project'
import { removeSelfFromTeam } from '~/helpers/teams.js'
import { projectCopyType } from '~/utils/project-messages'

import SettingsSection from '../settings-section.vue'
import InviteDialog from './invite-modal.vue'
import MemberEditor from './member-editor/index.vue'
import MemberTable from './member-table.vue'
import {
	hasPermission,
	projectAccessGroups,
	projectAccessPresets,
	projectPermissions,
} from './permissions'
import ProjectRevenueSplit from './project-revenue-split.vue'
import RemoveMemberModal from './remove-member-modal.vue'
import {
	isValidPayoutWeight,
	revenueMembersWithOverrides,
	revenueShare,
	revenueTotal,
	revenueWeightUpdates,
} from './revenue'
import type { MemberWeight } from './revenue-split-editor/allocation'
import type { MemberDraft } from './types'
import { useMemberErrorNotification } from './use-member-error-notification'

const {
	projectV2: project,
	projectV3,
	organization,
	allMembers,
	currentMember,
	invalidate,
} = injectProjectPageContext()
const client = injectModrinthClient()
const { labrinth } = client
const queryClient = useQueryClient()
const { formatMessage, locale } = useVIntl()
const { addNotification } = injectNotificationManager()
const notifyError = useMemberErrorNotification()
const loading = useLoading()
const auth = await useAuth()
const currentUserId = computed(() => auth.value.user?.id)

const messages = defineMessages({
	membersHelp: {
		id: 'project.settings.members.access-help',
		defaultMessage:
			'Members have access to this {contentType, select, mod {mod} modpack {modpack} resourcepack {resource pack} shader {shader} plugin {plugin} datapack {data pack} server {server} other {project}} even when private, and can modify the project page if assigned permissions by the owner.',
	},
	addMember: {
		id: 'project.settings.members.add-member',
		defaultMessage: 'Add member',
	},
	managedBy: {
		id: 'project.settings.members.organization-defaults-location',
		defaultMessage:
			'Organization member defaults are set in <organization>{name} settings</organization>.',
	},
	leave: {
		id: 'project.settings.members.leave.title',
		defaultMessage: 'Leave project',
	},
	leaveDescription: {
		id: 'project.settings.members.leave.description',
		defaultMessage: 'Remove yourself as a member of this project.',
	},
	revenue: {
		id: 'project.settings.members.revenue.title',
		defaultMessage: 'Revenue split',
	},
	revenueChanged: {
		id: 'project.settings.members.revenue.changed-while-editing',
		defaultMessage:
			'The revenue split changed while you were editing. Reset your edits to see the latest split.',
	},
	revenueSaved: {
		id: 'project.settings.members.revenue.saved',
		defaultMessage: 'Revenue split updated',
	},
	revenueSaveStopped: {
		id: 'project.settings.members.revenue.save-stopped',
		defaultMessage:
			'{count, plural, =0 {The save stopped before any requested changes could be confirmed.} other {The save stopped. The latest split matches your requested weights for {members}.}}',
	},
	revenueRefreshFailed: {
		id: 'project.settings.members.revenue.refresh-failed',
		defaultMessage:
			'The save stopped and the latest split could not be loaded. Reload the page to check which changes took effect.',
	},
	removed: {
		id: 'project.settings.members.removed.title',
		defaultMessage: 'Member removed',
	},
	removedDescription: {
		id: 'project.settings.members.removed.description',
		defaultMessage: "Your project's member has been removed.",
	},
	removalChanged: {
		id: 'project.settings.members.removal-changed',
		defaultMessage: 'Membership has changed. Check the member list before trying again.',
	},
	updated: {
		id: 'project.settings.members.updated.title',
		defaultMessage: 'Member(s) updated',
	},
	updatedDescription: {
		id: 'project.settings.members.updated.description',
		defaultMessage: "Your project's member(s) has been updated.",
	},
})

const allProjectPermissions = Object.values(projectPermissions).reduce((mask, bit) => mask | bit, 0)
const isServerProject = computed(() => projectV3.value?.minecraft_server != null)
const permissions = computed(() => currentMember.value?.permissions ?? null)
const canManageInvites = computed(() =>
	hasPermission(permissions.value, projectPermissions.manageInvites),
)
const inviteDialog = useTemplateRef('inviteDialog')
const memberEditor = useTemplateRef('memberEditor')
const removeMemberModal = useTemplateRef('removeMemberModal')
const revenueSplit = useTemplateRef('revenueSplit')

const organizationMembers = computed(() => organization.value?.members ?? [])
const teamByUserId = computed(
	() => new Map(allMembers.value.map((member) => [member.user.id, member])),
)

function compareSelfFirst(a: MemberDraft, b: MemberDraft) {
	const userId = currentUserId.value
	return Number(b.user.id === userId) - Number(a.user.id === userId)
}

function projectOrganizationMembers(
	members: MemberDraft[],
	projectMembersById: ReadonlyMap<string, MemberDraft>,
): MemberDraft[] {
	return members.map((orgMember) => {
		const member = projectMembersById.get(orgMember.user.id)
		return {
			...(member ?? orgMember),
			is_owner: orgMember.is_owner,
			override: !!member,
			oldOverride: !!member,
			organizationDefaults: {
				role: orgMember.role,
				permissions: orgMember.permissions,
				payouts_split: orgMember.payouts_split,
			},
		}
	})
}

const members = computed(() => {
	const orgMembers = projectOrganizationMembers(organizationMembers.value, teamByUserId.value)
	const orgUserIds = new Set(orgMembers.map(({ user }) => user.id))
	const projectMembers = allMembers.value.filter(({ user }) => !orgUserIds.has(user.id))
	return [...projectMembers.sort(compareSelfFirst), ...orgMembers.sort(compareSelfFirst)]
})
const hasPendingMembers = computed(() => members.value.some((member) => !member.accepted))
function refreshMembers() {
	const projectId = project.value.id
	void queryClient.invalidateQueries(
		{
			queryKey: projectQueryOptions.members(projectId, client).queryKey,
			exact: true,
		},
		{ cancelRefetch: false },
	)
	void queryClient.invalidateQueries(
		{
			queryKey: projectQueryOptions.organization(projectId, client).queryKey,
			exact: true,
		},
		{ cancelRefetch: false },
	)
}

const visibility = useDocumentVisibility()
onMounted(refreshMembers)
watch(visibility, (state) => {
	if (state === 'visible') refreshMembers()
})

const { pause: pauseMemberPolling, resume: resumeMemberPolling } = useIntervalFn(
	refreshMembers,
	30_000,
	{ immediate: false },
)
watch(
	hasPendingMembers,
	(pending) => {
		if (import.meta.client && pending) {
			resumeMemberPolling()
		} else {
			pauseMemberPolling()
		}
	},
	{ immediate: true },
)

const accessOptions = computed(() => ({
	presets: projectAccessPresets(isServerProject.value).map((preset) => ({
		...preset,
		label: formatMessage(preset.label),
		description: formatMessage(preset.description),
	})),
	groups: projectAccessGroups(isServerProject.value).map((group) => ({
		...group,
		label: formatMessage(group.label),
		options: group.options.map((option) => ({
			...option,
			label: formatMessage(option.label),
		})),
	})),
}))

const revenueMembers = computed(() => members.value.filter((member) => member.accepted))
const showRevenue = computed(
	() =>
		!isServerProject.value &&
		(!project.value.organization || !!organization.value) &&
		revenueMembers.value.every((member) => member.payouts_split != null),
)
const totalWeight = computed(() => (showRevenue.value ? revenueTotal(revenueMembers.value) : 0))
const memberShare = (member: MemberDraft) =>
	revenueShare(member, totalWeight.value, revenueMembers.value.length)

async function saveMemberDetails(team: string, member: MemberDraft, original: MemberDraft) {
	const userId = member.user.id
	if (member.override && !original.oldOverride) {
		await labrinth.teams_v2.addMember(team, {
			user_id: userId,
			role: member.role,
			payouts_split: member.payouts_split ?? undefined,
			permissions: member.is_owner ? allProjectPermissions : (member.permissions ?? undefined),
		})
		return
	}
	if (member.override === false && original.oldOverride) {
		await labrinth.teams_v2.removeMember(team, userId)
		return
	}

	const role = member.role !== original.role ? member.role : undefined
	const permissions =
		!member.is_owner && member.permissions !== original.permissions
			? (member.permissions ?? undefined)
			: undefined
	const payoutWeight =
		member.payouts_split !== original.payouts_split
			? (member.payouts_split ?? undefined)
			: undefined
	if (role !== undefined || permissions !== undefined || payoutWeight !== undefined) {
		await labrinth.teams_v2.editMember(team, userId, {
			role,
			permissions,
			payouts_split: payoutWeight,
		})
	}
}

const { mutateAsync: saveMember, isPending: saving } = useMutation({
	mutationFn: ({ member, original }: { member: MemberDraft; original: MemberDraft }) =>
		saveMemberDetails(project.value.team, member, original),
	onSuccess: async () => {
		await invalidate()
		addNotification({
			title: formatMessage(messages.updated),
			text: formatMessage(messages.updatedDescription),
			type: 'success',
		})
	},
	onError: notifyError,
	onSettled: () => {
		loading.value = false
	},
})

async function fetchRevenueMembers(projectId: string) {
	const [members, projectOrganization] = await Promise.all([
		queryClient.fetchQuery({
			...projectQueryOptions.members(projectId, client),
			staleTime: 0,
		}),
		queryClient.fetchQuery({
			...projectQueryOptions.organization(projectId, client),
			staleTime: 0,
		}),
	])
	const membersById = new Map<string, MemberDraft>(
		members.map((member) => [member.user.id, member]),
	)
	for (const member of projectOrganizationMembers(
		projectOrganization?.members ?? [],
		membersById,
	)) {
		membersById.set(member.user.id, member)
	}
	return new Map([...membersById].filter(([, member]) => member.accepted))
}

const { mutateAsync: saveSplit, isPending: savingRevenue } = useMutation({
	retry: false,
	mutationFn: async ({
		projectId,
		teamId,
		allocation,
		original,
		overrides,
	}: {
		projectId: string
		teamId: string
		allocation: MemberWeight[]
		original: MemberDraft[]
		overrides: ReadonlyMap<string, boolean>
	}) => {
		const current = await fetchRevenueMembers(projectId)
		if (
			current.size !== original.length ||
			original.some((member) => {
				const latest = current.get(member.user.id)
				return (
					!latest ||
					member.payouts_split == null ||
					latest.payouts_split !== member.payouts_split ||
					latest.override !== member.override ||
					(overrides.has(member.user.id) &&
						(!canToggleRevenueOverride(latest) ||
							latest.role !== member.role ||
							latest.permissions !== member.permissions ||
							latest.organizationDefaults?.payouts_split !==
								member.organizationDefaults?.payouts_split))
				)
			})
		) {
			if (project.value.team === teamId) await invalidate()
			throw new Error(formatMessage(messages.revenueChanged))
		}

		const effectiveMembers = revenueMembersWithOverrides(original, overrides)
		const weights = new Map(
			revenueWeightUpdates(effectiveMembers, allocation).map(({ userId, weight }) => [
				userId,
				weight,
			]),
		)
		if ([...weights.values()].some((weight) => !isValidPayoutWeight(weight))) return

		const updates = original.flatMap((before, index) => {
			const member = effectiveMembers[index]
			const weight = weights.get(before.user.id)
			if (!member || (weight === undefined && !overrides.has(before.user.id))) return []
			return [{ before, after: { ...member, payouts_split: weight ?? member.payouts_split } }]
		})
		try {
			// TODO: Replace these sequential requests with a bulk PATCH that saves all weights atomically.
			for (const { before, after } of updates) {
				await saveMemberDetails(teamId, after, before)
			}
		} catch (error) {
			const latest = await fetchRevenueMembers(projectId).catch(() => null)
			if (!latest || [...latest.values()].some((member) => member.payouts_split == null)) {
				throw new Error(formatMessage(messages.revenueRefreshFailed), {
					cause: error,
				})
			}

			await nextTick()
			if (project.value.team === teamId) revenueSplit.value?.reset()

			const savedMembers = updates.flatMap(({ after }) => {
				const member = latest.get(after.user.id)
				return member &&
					member.override === after.override &&
					member.payouts_split === after.payouts_split
					? [member.user.username]
					: []
			})
			throw new Error(
				formatMessage(messages.revenueSaveStopped, {
					count: savedMembers.length,
					members: new Intl.ListFormat(locale.value).format(savedMembers),
				}),
				{ cause: error },
			)
		}
	},
	onSuccess: async () => {
		await invalidate()
		addNotification({
			title: formatMessage(messages.revenueSaved),
			type: 'success',
		})
	},
	onError: notifyError,
	onSettled: () => {
		loading.value = false
	},
})

const { mutateAsync: removeMember, isPending: removing } = useMutation({
	mutationFn: async ({
		projectId,
		teamId,
		member,
	}: {
		projectId: string
		teamId: string
		member: MemberDraft
	}) => {
		const roster = await queryClient.fetchQuery({
			...projectQueryOptions.members(projectId, client),
			staleTime: 0,
		})
		const latest = roster.find(({ user }) => user.id === member.user.id)
		if (!latest || latest.is_owner || latest.accepted !== member.accepted) return false

		await labrinth.teams_v2.removeMember(teamId, member.user.id)
		return true
	},
	onSuccess: async (removed, { teamId }) => {
		if (!removed) {
			addNotification({ title: formatMessage(messages.removalChanged), type: 'warning' })
			return
		}
		if (project.value.team === teamId) await invalidate()
		addNotification({
			title: formatMessage(messages.removed),
			text: formatMessage(messages.removedDescription),
			type: 'success',
		})
	},
	onError: notifyError,
	onSettled: () => {
		loading.value = false
	},
})

const busy = computed(() => saving.value || removing.value || savingRevenue.value)
const canEditMembers = computed(() =>
	hasPermission(permissions.value, projectPermissions.editMember),
)
const canRemoveMembers = computed(() =>
	hasPermission(permissions.value, projectPermissions.removeMember),
)

function canCreateProjectOverride(member: MemberDraft) {
	const requiredPermissions = member.is_owner ? allProjectPermissions : member.permissions
	return (
		canManageInvites.value &&
		member.payouts_split != null &&
		requiredPermissions != null &&
		hasPermission(permissions.value, requiredPermissions)
	)
}

function canRemoveProjectOverride(member: MemberDraft) {
	if (!member.accepted) return canManageInvites.value
	return canRemoveMembers.value || member.user.id === currentUserId.value
}

function canChangeProjectOverride(member: MemberDraft) {
	return member.oldOverride ? canRemoveProjectOverride(member) : canCreateProjectOverride(member)
}

function canRestoreRevenueInheritance(member: MemberDraft) {
	const defaults = member.organizationDefaults
	return (
		defaults &&
		canRemoveProjectOverride(member) &&
		member.role === defaults.role &&
		(member.is_owner || member.permissions === defaults.permissions)
	)
}

function canToggleRevenueOverride(member: MemberDraft) {
	if (!member.organizationDefaults) return false
	return member.override ? !!canRestoreRevenueInheritance(member) : canCreateProjectOverride(member)
}

async function save(member: MemberDraft, original: MemberDraft) {
	if (busy.value || !canEditMembers.value) return
	if (member.override === false && !original.oldOverride) return
	if (member.override !== original.override && !canChangeProjectOverride(original)) return

	loading.value = true
	try {
		await saveMember({ member, original })
	} catch {
		return
	}

	await nextTick()
	memberEditor.value?.hide()
}

async function saveRevenue(
	allocation: MemberWeight[],
	original: MemberDraft[],
	overrides: ReadonlyMap<string, boolean>,
) {
	if (busy.value || !canEditMembers.value || !showRevenue.value) return
	const { id: projectId, team: teamId } = project.value
	loading.value = true
	try {
		await saveSplit({ projectId, teamId, allocation, original, overrides })
	} catch {
		return
	}
	await nextTick()
	if (project.value.team === teamId) revenueSplit.value?.reset()
}

async function remove(member: MemberDraft) {
	if (busy.value || !canRemoveMembers.value || member.is_owner) return
	const { id: projectId, team: teamId } = project.value
	const dialog = removeMemberModal.value
	loading.value = true
	try {
		await removeMember({ projectId, teamId, member })
	} catch {
		return
	}
	await nextTick()
	dialog?.hide()
}

async function leaveProject() {
	await removeSelfFromTeam(project.value.team)
	navigateTo('/dashboard/projects')
}

const canLeaveProject = computed(() =>
	members.value.some(
		(member) =>
			member.user.id === currentUserId.value &&
			member.accepted &&
			!member.is_owner &&
			member.override !== false,
	),
)
</script>

<template>
	<div data-project-field="members" data-project-field-scroll-only>
		<ProjectIssueCard location="members" class="mb-4" />
		<InviteDialog
			ref="inviteDialog"
			:team-id="project.team"
			:team-members-by-user-id="teamByUserId"
			:on-invited="invalidate"
			:can-invite="canManageInvites"
			:grantable="permissions ?? 0"
			:access-options="accessOptions"
			:title="formatMessage(messages.addMember)"
		/>
		<MemberEditor
			ref="memberEditor"
			:permissions="permissions"
			:access-options="accessOptions"
			:is-server-project="isServerProject"
			:show-revenue-split="showRevenue"
			:can-change-override="canChangeProjectOverride"
			:saving="saving"
			@save="save"
		/>
		<RemoveMemberModal
			:key="project.team"
			ref="removeMemberModal"
			:team-name="project.title"
			:has-revenue="!isServerProject"
			:busy="busy || !canRemoveMembers"
			:removing="removing"
			@confirm="remove"
		/>
		<div class="flex flex-col gap-8">
			<SettingsSection
				:title="formatMessage(commonProjectSettingsMessages.members)"
				:subtitle="formatMessage(messages.membersHelp, { contentType: projectCopyType(projectV3) })"
			>
				<template v-if="canManageInvites" #actions>
					<Button
						type="colored"
						color="brand"
						:aria-busy="inviteDialog?.isOpening || undefined"
						@click="inviteDialog?.show()"
					>
						<UserPlusIcon /> {{ formatMessage(messages.addMember) }}
					</Button>
				</template>
				<MemberTable
					:members="members"
					:user-id="currentUserId"
					:permissions="permissions"
					:is-server-project="isServerProject"
					:show-revenue="showRevenue"
					:revenue-share="memberShare"
					:busy="busy"
					@edit="(member, event) => memberEditor?.show(member, event)"
					@remove="(member, event) => removeMemberModal?.show(member, event)"
				/>
				<p v-if="organization" class="mb-0 mt-4 text-secondary">
					<IntlFormatted :message-id="messages.managedBy" :values="{ name: organization.name }">
						<template #organization="{ children }">
							<NuxtLink
								:to="`/organization/${organization.slug}/settings/members`"
								target="_blank"
								rel="noopener noreferrer"
								class="inline-flex items-center gap-1 align-middle"
							>
								<Avatar
									:src="organization.icon_url"
									:tint-by="organization.id"
									size="1.5rem"
									no-shadow
								/>
								<span class="underline"><component :is="() => children" /></span>
							</NuxtLink>
						</template>
					</IntlFormatted>
				</p>
			</SettingsSection>
			<SettingsSection
				v-if="showRevenue && revenueMembers.length"
				:title="formatMessage(messages.revenue)"
			>
				<ProjectRevenueSplit
					:key="project.team"
					ref="revenueSplit"
					:members="revenueMembers"
					:can-toggle-override="canToggleRevenueOverride"
					:can-edit="canEditMembers"
					:busy="busy"
					:saving="savingRevenue"
					@save="saveRevenue"
				/>
			</SettingsSection>
			<SettingsOptionCard v-if="canLeaveProject" :title="formatMessage(messages.leave)">
				<p>{{ formatMessage(messages.leaveDescription) }}</p>
				<template #actions>
					<Button type="colored" color="red" @click="leaveProject">
						<UserXIcon /> {{ formatMessage(messages.leave) }}
					</Button>
				</template>
			</SettingsOptionCard>
		</div>
	</div>
</template>
