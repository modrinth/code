<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { TransferIcon, UserPlusIcon, UserXIcon } from '@modrinth/assets'
import {
	Button,
	Checkbox,
	commonProjectSettingsMessages,
	defineMessages,
	IconButton,
	injectModrinthClient,
	injectNotificationManager,
	SettingsOptionCard,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { computed, nextTick, shallowRef, useTemplateRef } from 'vue'

import InviteDialog from '~/components/project-settings/members/invite-modal.vue'
import MemberEditor from '~/components/project-settings/members/member-editor/index.vue'
import ProjectAccessPicker from '~/components/project-settings/members/member-editor/project-access-picker.vue'
import MemberTable from '~/components/project-settings/members/member-table.vue'
import {
	hasPermission,
	permissionMessages,
	permissionsHiddenMessage,
	projectAccessGroups,
	projectAccessPresets,
	projectPermissions,
} from '~/components/project-settings/members/permissions'
import RevenueSplit from '~/components/project-settings/members/project-revenue-split.vue'
import RemoveMemberModal from '~/components/project-settings/members/remove-member-modal.vue'
import {
	isValidPayoutWeight,
	revenueWeightUpdates,
} from '~/components/project-settings/members/revenue'
import type { MemberWeight } from '~/components/project-settings/members/revenue-split-editor/allocation'
import type { MemberDraft } from '~/components/project-settings/members/types'
import { useMemberErrorNotification } from '~/components/project-settings/members/use-member-error-notification'
import SettingsSection from '~/components/project-settings/settings-section.vue'
import ConfirmTransferOrgModal from '~/components/ui/ConfirmTransferOrgModal.vue'
import { projectQueryOptions } from '~/composables/queries/project'
import { injectOrganizationContext } from '~/providers/organization-context'

const messages = defineMessages({
	addMember: {
		id: 'organization.settings.members.add-member',
		defaultMessage: 'Add member',
	},
	organizationAccess: {
		id: 'organization.settings.members.organization-access',
		defaultMessage: 'Organization access',
	},
	defaultProjectAccess: {
		id: 'organization.settings.members.default-project-access',
		defaultMessage: 'Default project access',
	},
	noOrganizationAccess: {
		id: 'organization.settings.members.no-organization-access',
		defaultMessage: 'No organization permissions',
	},
	customAccess: {
		id: 'organization.settings.members.custom-access',
		defaultMessage: 'Custom',
	},
	addProject: {
		id: 'organization.settings.members.permission.add-project',
		defaultMessage: 'Add projects',
	},
	removeProject: {
		id: 'organization.settings.members.permission.remove-project',
		defaultMessage: 'Remove projects',
	},
	deleteOrganization: {
		id: 'organization.settings.members.permission.delete-organization',
		defaultMessage: 'Delete organization',
	},
	editDefaultAccess: {
		id: 'organization.settings.members.permission.edit-default-access',
		defaultMessage: 'Edit default project access',
	},
	restrictedAccess: {
		id: 'organization.settings.members.editor.restricted-access',
		defaultMessage:
			'This member has permissions you can’t grant. You can edit their other details.',
	},
	transfer: {
		id: 'organization.settings.members.transfer',
		defaultMessage: 'Transfer ownership to {username}',
	},
	transferred: {
		id: 'organization.settings.members.transferred',
		defaultMessage: 'Ownership transferred',
	},
	invited: {
		id: 'organization.settings.members.invited',
		defaultMessage: 'Member invited',
	},
	updated: {
		id: 'organization.settings.members.updated',
		defaultMessage: 'Member updated',
	},
	removed: {
		id: 'organization.settings.members.removed',
		defaultMessage: 'Member removed',
	},
	inviteCanceled: {
		id: 'organization.settings.members.invite-canceled',
		defaultMessage: 'Invite canceled',
	},
	revenue: {
		id: 'organization.settings.members.revenue.title',
		defaultMessage: 'Revenue split',
	},
	revenueDescription: {
		id: 'organization.settings.members.revenue.description',
		defaultMessage:
			'These shares set the default payout weights for organization members. Each project’s split can differ if it has project members or overrides.',
	},
	revenueChanged: {
		id: 'organization.settings.members.revenue.changed-while-editing',
		defaultMessage:
			'The revenue split changed while you were editing. Reset your edits to see the latest split.',
	},
	revenueSaved: {
		id: 'organization.settings.members.revenue.saved',
		defaultMessage: 'Revenue split updated',
	},
	revenueSaveStopped: {
		id: 'organization.settings.members.revenue.save-stopped',
		defaultMessage:
			'The save stopped before all changes could be applied. The editor shows the latest split.',
	},
	revenueRefreshFailed: {
		id: 'organization.settings.members.revenue.refresh-failed',
		defaultMessage:
			'The latest split could not be loaded. Reload the page to check which changes took effect.',
	},
	leave: {
		id: 'organization.settings.members.leave',
		defaultMessage: 'Leave organization',
	},
	leaveDescription: {
		id: 'organization.settings.members.leave-description',
		defaultMessage: 'Remove yourself as a member of this organization.',
	},
})

const organizationPermissions = {
	editDetails: 1 << 0,
	manageInvites: 1 << 1,
	removeMember: 1 << 2,
	editMember: 1 << 3,
	addProject: 1 << 4,
	removeProject: 1 << 5,
	deleteOrganization: 1 << 6,
	editDefaultAccess: 1 << 7,
}

const permissionOptions = {
	editDetails: permissionMessages.editDetails,
	manageInvites: permissionMessages.manageInvites,
	removeMember: permissionMessages.removeMember,
	editMember: permissionMessages.editMember,
	addProject: messages.addProject,
	removeProject: messages.removeProject,
	deleteOrganization: messages.deleteOrganization,
	editDefaultAccess: messages.editDefaultAccess,
}

const organizationAccessOptions = (
	Object.keys(organizationPermissions) as (keyof typeof organizationPermissions)[]
).map((key) => ({
	bit: organizationPermissions[key],
	label: permissionOptions[key],
}))

const { organization, projects, refresh, currentMember } = injectOrganizationContext()
const client = injectModrinthClient()
const { labrinth } = client
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const notifyError = useMemberErrorNotification()
const auth = await useAuth()
const currentUserId = computed<string | undefined>(() => auth.value.user?.id)
const inviteDialog = useTemplateRef('inviteDialog')
const memberEditor = useTemplateRef('memberEditor')
const revenueSplit = useTemplateRef('revenueSplit')
const removeModal = useTemplateRef('removeModal')
const transferModal = useTemplateRef('transferModal')
const transferTargetId = shallowRef<string>()

const members = computed<MemberDraft[]>(() =>
	(organization.value?.members ?? [])
		.map((member) => ({
			...member,
			organization_permissions:
				member.permissions == null ? null : (member.organization_permissions ?? 0),
		}))
		.sort(
			(a, b) =>
				Number(b.user.id === currentUserId.value) - Number(a.user.id === currentUserId.value),
		),
)
const revenueMembers = computed(() => members.value.filter((member) => member.accepted))
const showRevenue = computed(
	() =>
		revenueMembers.value.length > 0 &&
		revenueMembers.value.every((member) => member.payouts_split != null),
)
const viewer = computed(() =>
	members.value.find((member) => member.user.id === currentUserId.value),
)
const owner = computed(() => members.value.find((member) => member.is_owner))
const transferTarget = computed(() =>
	members.value.find((member) => member.user.id === transferTargetId.value),
)
const allOrganizationPermissions = Object.values(organizationPermissions).reduce(
	(mask, bit) => mask | bit,
	0,
)
const organizationMask = computed(() =>
	auth.value.user?.role === 'admin' || viewer.value?.is_owner
		? allOrganizationPermissions
		: currentMember.value?.accepted
			? currentMember.value.organization_permissions
			: null,
)
const canInvite = computed(() =>
	hasPermission(organizationMask.value, organizationPermissions.manageInvites),
)
const canEdit = computed(() =>
	hasPermission(organizationMask.value, organizationPermissions.editMember),
)
const canEditDefaults = computed(
	() =>
		canEdit.value &&
		hasPermission(organizationMask.value, organizationPermissions.editDefaultAccess),
)
const projectMask = computed(() =>
	auth.value.user?.role === 'admin' || viewer.value?.is_owner
		? Object.values(projectPermissions).reduce((mask, bit) => mask | bit, 0)
		: (currentMember.value?.permissions ?? 0),
)
const canLeave = computed(() => viewer.value?.accepted && !viewer.value.is_owner)
const accessOptions = computed(() => ({
	presets: projectAccessPresets(false).map((preset) => ({
		...preset,
		label: formatMessage(preset.label),
		description: formatMessage(preset.description),
	})),
	groups: projectAccessGroups(false).map((group) => ({
		...group,
		label: formatMessage(group.label),
		options: group.options.map((option) => ({
			...option,
			label: formatMessage(option.label),
		})),
	})),
}))

function accessLabel(member: MemberDraft) {
	if (member.organization_permissions == null) return formatMessage(permissionsHiddenMessage)
	return formatMessage(
		member.organization_permissions ? messages.customAccess : messages.noOrganizationAccess,
	)
}

function canRemove(member: MemberDraft) {
	const latest = members.value.find(({ user }) => user.id === member.user.id)
	return (
		!!latest &&
		!latest.is_owner &&
		latest.user.id !== currentUserId.value &&
		latest.accepted === member.accepted &&
		hasPermission(
			organizationMask.value,
			latest.accepted
				? organizationPermissions.removeMember
				: organizationPermissions.manageInvites,
		)
	)
}

function canTransfer(member: MemberDraft) {
	return !!viewer.value?.is_owner && member.accepted && !member.is_owner
}

function hasRestrictedOrganizationAccess(member: MemberDraft) {
	return !hasPermission(organizationMask.value, member.organization_permissions ?? 0)
}

function notify(message: (typeof messages)[keyof typeof messages]) {
	addNotification({ title: formatMessage(message), type: 'success' })
}

async function onInvited() {
	await refresh()
	notify(messages.invited)
}

const { mutateAsync: saveMember, isPending: saving } = useMutation({
	mutationFn: ({
		teamId,
		userId,
		changes,
	}: {
		teamId: string
		userId: string
		changes: Labrinth.Teams.v2.EditTeamMemberRequest
	}) => labrinth.teams_v2.editMember(teamId, userId, changes),
	onSuccess: async () => {
		await refresh()
		notify(messages.updated)
	},
	onError: notifyError,
})

const { mutateAsync: removeMember, isPending: removing } = useMutation({
	mutationFn: ({ teamId, member }: { teamId: string; member: MemberDraft }) =>
		labrinth.teams_v2.removeMember(teamId, member.user.id),
	onSuccess: async (_, { member }) => {
		await refresh()
		notify(member.accepted ? messages.removed : messages.inviteCanceled)
	},
	onError: notifyError,
})

const { mutate: transferOwnership, isPending: transferring } = useMutation({
	mutationFn: ({ teamId, userId }: { teamId: string; userId: string }) =>
		labrinth.teams_v2.transferOwnership(teamId, { user_id: userId }),
	onSuccess: async () => {
		await refresh()
		notify(messages.transferred)
	},
	onError: notifyError,
})

const { mutate: leaveOrganization, isPending: leaving } = useMutation({
	mutationFn: ({ teamId, userId }: { teamId: string; userId: string; organizationId: string }) =>
		labrinth.teams_v2.removeMember(teamId, userId),
	onSuccess: async (_, { organizationId }) => {
		await navigateTo(`/organization/${organizationId}`)
		await refresh()
	},
	onError: notifyError,
})

function updateOrganizationCache(latest: Labrinth.Organizations.v3.Organization) {
	queryClient.setQueriesData<Labrinth.Organizations.v3.Organization>(
		{
			queryKey: ['organization'],
			predicate: (query) =>
				query.queryKey.length === 2 &&
				(query.state.data as Labrinth.Organizations.v3.Organization | undefined)?.id === latest.id,
		},
		latest,
	)
}

const { mutate: saveSplit, isPending: savingRevenue } = useMutation({
	retry: false,
	mutationFn: async ({
		teamId,
		organizationId,
		projectIds,
		updates,
		original,
	}: {
		teamId: string
		organizationId: string
		projectIds: string[]
		updates: ReturnType<typeof revenueWeightUpdates>
		original: MemberDraft[]
	}) => {
		const latest = await labrinth.organizations_v3.get(organizationId)
		const accepted = latest.members.filter((member) => member.accepted)
		const weights = new Map(accepted.map((member) => [member.user.id, member.payouts_split]))
		if (
			latest.team_id !== teamId ||
			weights.size !== original.length ||
			original.some(
				(member) =>
					member.payouts_split == null || weights.get(member.user.id) !== member.payouts_split,
			)
		) {
			updateOrganizationCache(latest)
			throw new Error(formatMessage(messages.revenueChanged))
		}

		let saveError: unknown
		try {
			for (const { userId, weight } of updates) {
				await labrinth.teams_v2.editMember(teamId, userId, { payouts_split: weight })
			}
		} catch (error) {
			saveError = error
		} finally {
			await Promise.all(
				projectIds.map((projectId) =>
					queryClient.invalidateQueries({
						queryKey: projectQueryOptions.organization(projectId, client).queryKey,
						exact: true,
					}),
				),
			)
		}

		try {
			updateOrganizationCache(await labrinth.organizations_v3.get(organizationId))
		} catch (error) {
			throw new Error(formatMessage(messages.revenueRefreshFailed), { cause: error })
		}

		await nextTick()
		if (organization.value?.team_id === teamId) revenueSplit.value?.reset()
		if (saveError) {
			throw new Error(formatMessage(messages.revenueSaveStopped), { cause: saveError })
		}
	},
	onSuccess: () => notify(messages.revenueSaved),
	onError: notifyError,
})

const busy = computed(
	() =>
		saving.value || savingRevenue.value || removing.value || transferring.value || leaving.value,
)

function saveRevenue(allocation: MemberWeight[], original: MemberDraft[]) {
	const org = organization.value
	if (!org || busy.value || !canEdit.value || !showRevenue.value) return
	const updates = revenueWeightUpdates(original, allocation)
	if (updates.some(({ weight }) => !isValidPayoutWeight(weight))) return
	saveSplit({
		teamId: org.team_id,
		organizationId: org.id,
		projectIds: projects.value?.map(({ id }) => id) ?? [],
		updates,
		original,
	})
}

async function save(member: MemberDraft, original: MemberDraft) {
	const org = organization.value
	const latest = members.value.find(({ user }) => user.id === member.user.id)
	if (!org || !latest || !canEdit.value || busy.value) return

	const changes: Labrinth.Teams.v2.EditTeamMemberRequest = {}
	if (member.role !== original.role) changes.role = member.role
	if (!latest.is_owner) {
		if (
			member.organization_permissions != null &&
			member.organization_permissions !== original.organization_permissions
		) {
			if (hasRestrictedOrganizationAccess(latest) || hasRestrictedOrganizationAccess(member)) return
			changes.organization_permissions = member.organization_permissions
		}
		if (member.permissions != null && member.permissions !== original.permissions) {
			const changedBits = (latest.permissions ?? 0) ^ member.permissions
			if (!canEditDefaults.value || !hasPermission(projectMask.value, changedBits)) return
			changes.permissions = member.permissions
		}
	}
	if (!Object.keys(changes).length) {
		memberEditor.value?.hide()
		return
	}
	const dialog = memberEditor.value
	try {
		await saveMember({ teamId: org.team_id, userId: member.user.id, changes })
	} catch {
		return
	}
	await nextTick()
	dialog?.hide()
}

async function remove(member: MemberDraft) {
	const org = organization.value
	if (!org || busy.value || !canRemove(member)) return

	const dialog = removeModal.value
	try {
		await removeMember({ teamId: org.team_id, member })
	} catch {
		return
	}
	await nextTick()
	dialog?.hide()
}

async function showTransfer(member: MemberDraft, event: MouseEvent) {
	if (busy.value || !canTransfer(member)) return
	transferTargetId.value = member.user.id
	await nextTick()
	transferModal.value?.show(event)
}

function transfer() {
	const member = transferTarget.value
	if (organization.value && member && !busy.value && canTransfer(member)) {
		transferOwnership({ teamId: organization.value.team_id, userId: member.user.id })
	}
}

function leave() {
	const org = organization.value
	if (org && currentUserId.value && canLeave.value && !busy.value) {
		leaveOrganization({ teamId: org.team_id, userId: currentUserId.value, organizationId: org.id })
	}
}
</script>

<template>
	<div v-if="organization" class="flex flex-col gap-8">
		<InviteDialog
			:key="organization.team_id"
			ref="inviteDialog"
			organization
			:team-id="organization.team_id"
			:can-invite="canInvite && !busy"
			:title="formatMessage(messages.addMember)"
			:on-invited="onInvited"
		/>
		<MemberEditor
			:key="organization.id"
			ref="memberEditor"
			:organization="{ canEdit }"
			:saving="saving"
			@save="save"
		>
			<template #access="{ draft, canEdit: canEditFields }">
				<fieldset class="m-0 flex min-w-0 flex-col gap-2 border-0 p-0">
					<legend class="mb-2 p-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.organizationAccess) }}
					</legend>
					<p v-if="draft.organization_permissions == null" class="m-0 text-secondary">
						{{ formatMessage(permissionsHiddenMessage) }}
					</p>
					<template v-else>
						<div
							class="grid gap-3 rounded-xl border border-solid border-surface-5 p-4 sm:grid-cols-2"
						>
							<Checkbox
								v-for="{ bit, label } in organizationAccessOptions"
								:key="bit"
								:label="formatMessage(label)"
								:model-value="hasPermission(draft.organization_permissions, bit)"
								:disabled="
									!canEditFields ||
									hasRestrictedOrganizationAccess(draft) ||
									!hasPermission(organizationMask, bit)
								"
								@update:model-value="
									draft.organization_permissions = $event
										? (draft.organization_permissions ?? 0) | bit
										: (draft.organization_permissions ?? 0) & ~bit
								"
							/>
						</div>
						<p
							v-if="canEditFields && hasRestrictedOrganizationAccess(draft)"
							class="m-0 text-sm text-secondary"
						>
							{{ formatMessage(messages.restrictedAccess) }}
						</p>
					</template>
				</fieldset>
				<ProjectAccessPicker
					v-if="draft.permissions != null"
					v-model="draft.permissions"
					:title="formatMessage(messages.defaultProjectAccess)"
					:presets="accessOptions.presets"
					:groups="accessOptions.groups"
					:grantable="projectMask"
					:disabled="!canEditFields || !canEditDefaults"
				/>
				<div v-else class="flex flex-col gap-2">
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.defaultProjectAccess) }}
					</h3>
					<p class="m-0 text-secondary">{{ formatMessage(permissionsHiddenMessage) }}</p>
				</div>
			</template>
		</MemberEditor>
		<RemoveMemberModal
			:key="organization.id"
			ref="removeModal"
			organization
			:team-name="organization.name"
			:busy="busy"
			:removing="removing"
			:can-remove="canRemove"
			@confirm="remove"
		/>
		<ConfirmTransferOrgModal
			v-if="owner && transferTarget"
			ref="transferModal"
			:organization="organization"
			:current-owner="{
				...owner.user,
				avatar_url: owner.user.avatar_url ?? null,
				role: owner.role,
			}"
			:transfer-to="{
				...transferTarget.user,
				avatar_url: transferTarget.user.avatar_url ?? null,
				role: transferTarget.role,
			}"
			:on-confirm="transfer"
		/>
		<SettingsSection :title="formatMessage(commonProjectSettingsMessages.members)">
			<template v-if="canInvite" #actions>
				<Button
					type="colored"
					color="brand"
					:disabled="busy"
					:aria-busy="inviteDialog?.isOpening || undefined"
					@click="inviteDialog?.show()"
				>
					<UserPlusIcon /> {{ formatMessage(messages.addMember) }}
				</Button>
			</template>
			<MemberTable
				:members="members"
				:user-id="currentUserId"
				:organization-roster="{ accessLabel, canRemove }"
				:busy="busy"
				@edit="(member, event) => memberEditor?.show(member, event)"
				@remove="(member, event) => removeModal?.show(member, event)"
			>
				<template #member-actions="{ member }">
					<IconButton
						v-if="canTransfer(member)"
						v-tooltip="formatMessage(messages.transfer, { username: member.user.username })"
						:label="formatMessage(messages.transfer, { username: member.user.username })"
						type="quiet"
						:disabled="busy"
						@click="showTransfer(member, $event)"
					>
						<TransferIcon />
					</IconButton>
				</template>
			</MemberTable>
		</SettingsSection>
		<SettingsSection v-if="showRevenue" :title="formatMessage(messages.revenue)">
			<p class="mb-4 mt-0 text-secondary">{{ formatMessage(messages.revenueDescription) }}</p>
			<RevenueSplit
				:key="organization.team_id"
				ref="revenueSplit"
				:members="revenueMembers"
				:can-edit="canEdit"
				:busy="busy"
				:saving="savingRevenue"
				@save="saveRevenue"
			/>
		</SettingsSection>
		<SettingsOptionCard v-if="canLeave" :title="formatMessage(messages.leave)">
			<p>{{ formatMessage(messages.leaveDescription) }}</p>
			<template #actions>
				<Button type="colored" color="red" :disabled="busy" @click="leave">
					<UserXIcon /> {{ formatMessage(messages.leave) }}
				</Button>
			</template>
		</SettingsOptionCard>
	</div>
</template>
