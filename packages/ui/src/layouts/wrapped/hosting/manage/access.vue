<template>
	<div class="flex flex-col gap-4">
		<div class="flex flex-col gap-2 md:flex-row">
			<StyledInput
				v-model="memberSearch"
				:icon="SearchIcon"
				:placeholder="formatMessage(messages.searchUsersPlaceholder, { count: members.length })"
				wrapper-class="min-w-0 flex-1"
				input-class="!h-10"
				clearable
			/>
			<div class="flex shrink-0 gap-2 flex-wrap md:flex-nowrap">
				<Combobox
					v-model="roleFilter"
					:options="roleFilterOptions"
					:display-value="selectedRoleFilterLabel"
					trigger-class="min-w-[225px]"
				>
					<template #prefix>
						<FilterIcon class="size-5 text-secondary" aria-hidden="true" />
					</template>
				</Combobox>
				<ButtonStyled color="brand">
					<button class="!h-10 w-full md:w-fit" @click="grantAccessModal?.show($event)">
						<UserPlusIcon aria-hidden="true" />
						{{ formatMessage(messages.inviteFriends) }}
					</button>
				</ButtonStyled>
			</div>
		</div>

		<AccessTable
			:members="filteredMembers"
			:roles="roleOptions"
			@update-role="updateMemberRole"
			@resend-invite="resendInvite"
			@cancel-invite="requestCancelInvite"
			@remove-member="requestRemoveMember"
		/>

		<div class="flex flex-col gap-4">
			<span class="m-0 text-2xl font-semibold text-contrast">
				{{ formatMessage(messages.activityLogTitle) }}
			</span>
			<AuditLogTable
				v-model:query="auditQuery"
				v-model:filters="auditFilters"
				:entries="auditEntries"
				:users="members"
				:worlds="worldOptions"
			/>
		</div>

		<GrantAccessModal
			ref="grantAccessModal"
			:suggestions="inviteSuggestions"
			@grant="grantAccess"
		/>
		<RemoveAccessModal
			ref="removeMemberConfirmModal"
			:username="pendingRemovalMember?.user.username ?? ''"
			:should-cancel="shouldCancelInvite"
			@remove="confirmAccessRemoval"
		/>
	</div>
</template>

<script setup lang="ts">
import type { Archon, Labrinth } from '@modrinth/api-client'
import { FilterIcon, SearchIcon, UserPlusIcon } from '@modrinth/assets'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import Combobox, { type ComboboxOption } from '#ui/components/base/Combobox.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
import {
	AccessTable,
	AuditLogTable,
	GrantAccessModal,
	RemoveAccessModal,
	type GrantServerAccessPayload,
	type ServerAccessInviteSuggestion,
	type ServerAccessMember,
	type ServerAccessRole,
	type ServerAccessRoleOption,
	type ServerAccessUser,
	type ServerAuditLogEntry,
	type ServerAuditLogFilters,
} from '#ui/components/servers/access'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import {
	injectAuth,
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'

type RoleFilter = ServerAccessRole | 'all'

const { formatMessage } = useVIntl()
const auth = injectAuth()
const client = injectModrinthClient()
const { serverId, server } = injectModrinthServerContext()
const { addNotification } = injectNotificationManager()
const queryClient = useQueryClient()
const grantAccessModal = ref<InstanceType<typeof GrantAccessModal> | null>(null)
const removeMemberConfirmModal = ref<InstanceType<typeof RemoveAccessModal> | null>(null)
const pendingRemovalMember = ref<ServerAccessMember | null>(null)
const shouldCancelInvite = ref(false)

const messages = defineMessages({
	searchUsersPlaceholder: {
		id: 'servers.access-page.search-users-placeholder',
		defaultMessage: 'Search {count} {count, plural, one {user} other {users}}...',
	},
	inviteFriends: {
		id: 'servers.access-page.invite-friends',
		defaultMessage: 'Add user',
	},
	activityLogTitle: {
		id: 'servers.access-page.activity-log-title',
		defaultMessage: 'Activity log',
	},
	allRoles: {
		id: 'servers.access-page.role-filter.all',
		defaultMessage: 'All',
	},
	selectedRoleFilter: {
		id: 'servers.access-page.role-filter.selected',
		defaultMessage: 'Role: {role}',
	},
	ownerRole: {
		id: 'servers.access-page.role.owner',
		defaultMessage: 'Owner',
	},
	ownerDescription: {
		id: 'servers.access-page.role.owner-description',
		defaultMessage: 'Full access including billing, members, and destructive actions.',
	},
	editorRole: {
		id: 'servers.access-page.role.editor',
		defaultMessage: 'Editor',
	},
	editorDescription: {
		id: 'servers.access-page.role.editor-description',
		defaultMessage: 'Manage server content, files, backups, and other settings.',
	},
	viewerRole: {
		id: 'servers.access-page.role.viewer',
		defaultMessage: 'Limited',
	},
	viewerDescription: {
		id: 'servers.access-page.role.viewer-description',
		defaultMessage: 'Start, stop, restart, and view the server without making changes.',
	},
	inviteSentTitle: {
		id: 'servers.access-page.notification.invite-sent.title',
		defaultMessage: 'Invite sent',
	},
	inviteSentText: {
		id: 'servers.access-page.notification.invite-sent.text',
		defaultMessage: 'Invited {target} as {role}.',
	},
	inviteResentTitle: {
		id: 'servers.access-page.notification.invite-resent.title',
		defaultMessage: 'Invite resent',
	},
	inviteResentText: {
		id: 'servers.access-page.notification.invite-resent.text',
		defaultMessage: 'Sent another invite to {target}.',
	},
	inviteCancelledTitle: {
		id: 'servers.access-page.notification.invite-cancelled.title',
		defaultMessage: 'Invite cancelled',
	},
	inviteCancelledText: {
		id: 'servers.access-page.notification.invite-cancelled.text',
		defaultMessage: 'Cancelled the invite for {target}.',
	},
	memberRemovedTitle: {
		id: 'servers.access-page.notification.member-removed.title',
		defaultMessage: 'Access removed',
	},
	memberRemovedText: {
		id: 'servers.access-page.notification.member-removed.text',
		defaultMessage: 'Removed {target} from this server.',
	},
	roleUpdatedTitle: {
		id: 'servers.access-page.notification.role-updated.title',
		defaultMessage: 'Role updated',
	},
	roleUpdatedText: {
		id: 'servers.access-page.notification.role-updated.text',
		defaultMessage: 'Changed {target} to {role}.',
	},
	loadFailedTitle: {
		id: 'servers.access-page.notification.load-failed.title',
		defaultMessage: 'Access could not be loaded',
	},
	loadFailedText: {
		id: 'servers.access-page.notification.load-failed.text',
		defaultMessage: 'Refresh the page to try again.',
	},
	userLookupFailedTitle: {
		id: 'servers.access-page.notification.user-lookup-failed.title',
		defaultMessage: 'User could not be found',
	},
	userLookupFailedText: {
		id: 'servers.access-page.notification.user-lookup-failed.text',
		defaultMessage: 'Could not find {target}. Check the username and try again.',
	},
	inviteFailedTitle: {
		id: 'servers.access-page.notification.invite-failed.title',
		defaultMessage: 'User could not be added',
	},
	inviteResendUnavailableTitle: {
		id: 'servers.access-page.notification.invite-resend-unavailable.title',
		defaultMessage: 'Invite cannot be resent',
	},
	inviteResendUnavailableText: {
		id: 'servers.access-page.notification.invite-resend-unavailable.text',
		defaultMessage: 'Invites are accepted automatically for this server.',
	},
	removeFailedTitle: {
		id: 'servers.access-page.notification.remove-failed.title',
		defaultMessage: 'Access could not be removed',
	},
	roleUpdateFailedTitle: {
		id: 'servers.access-page.notification.role-update-failed.title',
		defaultMessage: 'Role could not be updated',
	},
})

const roleOptions = computed<ServerAccessRoleOption[]>(() => [
	{
		value: 'owner',
		label: formatMessage(messages.ownerRole),
		description: formatMessage(messages.ownerDescription),
	},
	{
		value: 'editor',
		label: formatMessage(messages.editorRole),
		description: formatMessage(messages.editorDescription),
	},
	{
		value: 'viewer',
		label: formatMessage(messages.viewerRole),
		description: formatMessage(messages.viewerDescription),
	},
])

const roleFilterOptions = computed<ComboboxOption<RoleFilter>[]>(() => [
	{ value: 'all', label: formatMessage(messages.allRoles) },
	...roleOptions.value.map((role) => ({
		value: role.value,
		label: role.label,
	})),
])

const selectedRoleFilterLabel = computed(() =>
	formatMessage(messages.selectedRoleFilter, {
		role:
			roleFilterOptions.value.find((option) => option.value === roleFilter.value)?.label ??
			formatMessage(messages.allRoles),
	}),
)

const serverUsersQueryKey = ['servers', 'users', 'v1', serverId]
const serverUsersQuery = useQuery({
	queryKey: serverUsersQueryKey,
	queryFn: () => client.archon.server_users_v1.list(serverId),
})

const serverFullQuery = useQuery({
	queryKey: ['servers', 'v1', 'detail', serverId],
	queryFn: () => client.archon.servers_v1.get(serverId),
})

const userIds = computed(() => [
	...new Set((serverUsersQuery.data.value ?? []).map((user) => user.user_id)),
])

const userProfilesQuery = useQuery({
	queryKey: computed(() => ['labrinth', 'users', 'v2', userIds.value]),
	queryFn: () => client.labrinth.users_v2.getMultiple(userIds.value),
	enabled: computed(() => userIds.value.length > 0),
})

const userProfiles = computed(() => {
	const profiles = new Map<string, Labrinth.Users.v2.User>()
	for (const user of userProfilesQuery.data.value ?? []) {
		profiles.set(user.id, user)
	}
	return profiles
})

const members = computed<ServerAccessMember[]>(() =>
	(serverUsersQuery.data.value ?? [])
		.map((serverUser) => {
			const profile = userProfiles.value.get(serverUser.user_id)
			const username = profile?.username ?? serverUser.user_id

			return {
				id: `${serverId}-${serverUser.user_id}`,
				user: {
					id: serverUser.user_id,
					username,
					avatarUrl: profile?.avatar_url,
				},
				role: apiRoleToAccessRole(serverUser.role),
				joinedAt: serverUser.added_on,
				isOwner: serverUser.role === 'Owner' || serverUser.user_id === server.value?.owner_id,
			}
		})
		.sort((a, b) => {
			const ownerSort = Number(b.isOwner) - Number(a.isOwner)
			return ownerSort === 0 ? a.user.username.localeCompare(b.user.username) : ownerSort
		}),
)

const worldOptions = computed(
	() =>
		serverFullQuery.data.value?.worlds.map((world) => ({ id: world.id, name: world.name })) ?? [],
)

const auditEntries = ref<ServerAuditLogEntry[]>([])
const inviteSuggestions: ServerAccessInviteSuggestion[] = []
const hasShownLoadError = ref(false)

const memberSearch = ref('')
const roleFilter = ref<RoleFilter>('all')
const auditQuery = ref('')
const auditFilters = ref<ServerAuditLogFilters>({
	userId: null,
	worldId: null,
	actionType: null,
})

const filteredMembers = computed(() => {
	const normalizedSearch = memberSearch.value.trim().toLowerCase()
	return members.value.filter((member) => {
		if (roleFilter.value !== 'all' && member.role !== roleFilter.value) return false
		if (!normalizedSearch) return true

		const roleLabel = formatRole(member.role)
		const pendingLabel = member.pending ? 'pending' : ''
		return [member.user.username, roleLabel, pendingLabel].some((value) =>
			value.toLowerCase().includes(normalizedSearch),
		)
	})
})

function formatRole(role: ServerAccessRole) {
	return roleOptions.value.find((option) => option.value === role)?.label ?? role
}

const currentActor = computed<ServerAccessUser | null>(() => {
	if (auth.user.value) return labrinthUserToAccessUser(auth.user.value)
	return members.value.find((member) => member.isOwner)?.user ?? null
})

watch(
	() => [serverUsersQuery.error.value, userProfilesQuery.error.value],
	([serverUsersError, userProfilesError]) => {
		if (hasShownLoadError.value || (!serverUsersError && !userProfilesError)) return

		hasShownLoadError.value = true
		addNotification({
			type: 'error',
			title: formatMessage(messages.loadFailedTitle),
			text:
				formatErrorMessage(serverUsersError ?? userProfilesError) ??
				formatMessage(messages.loadFailedText),
		})
	},
)

function labrinthUserToAccessUser(user: Labrinth.Users.v2.User): ServerAccessUser {
	return {
		id: user.id,
		username: user.username,
		avatarUrl: user.avatar_url,
	}
}

function apiRoleToAccessRole(role: Archon.ServerUsers.v1.ServerUserRole): ServerAccessRole {
	switch (role) {
		case 'Owner':
			return 'owner'
		case 'Editor':
			return 'editor'
		case 'Viewer':
		case 'Unknown':
			return 'viewer'
	}
}

function accessRoleToApiRole(
	role: Exclude<ServerAccessRole, 'owner'>,
): Archon.ServerUsers.v1.AssignableServerUserRole {
	switch (role) {
		case 'editor':
			return 'Editor'
		case 'viewer':
			return 'Viewer'
	}
}

function formatErrorMessage(error: unknown): string | undefined {
	return error instanceof Error ? error.message : undefined
}

async function invalidateServerUsers() {
	await queryClient.invalidateQueries({ queryKey: serverUsersQueryKey })
}

function findMemberByTarget(target: string) {
	const normalizedTarget = target.trim().toLowerCase()
	return members.value.find(
		(member) =>
			member.user.username.toLowerCase() === normalizedTarget ||
			member.user.id.toLowerCase() === normalizedTarget,
	)
}

function pushAuditEntry(entry: Omit<ServerAuditLogEntry, 'id' | 'timestamp' | 'actor'>) {
	const actor = currentActor.value
	if (!actor) return

	auditEntries.value = [
		{
			...entry,
			actor,
			id: `audit-${Date.now()}-${Math.random().toString(36).slice(2)}`,
			timestamp: new Date().toISOString(),
		},
		...auditEntries.value,
	]
}

async function updateMemberRole(member: ServerAccessMember, role: ServerAccessRole) {
	if (member.isOwner || member.role === role || role === 'owner') return

	try {
		await client.archon.server_users_v1.update(serverId, member.user.id, accessRoleToApiRole(role))
		await invalidateServerUsers()
		pushAuditEntry({
			world: null,
			action: { type: 'role_changed', target: member.user.username, role },
		})
		addNotification({
			type: 'success',
			title: formatMessage(messages.roleUpdatedTitle),
			text: formatMessage(messages.roleUpdatedText, {
				target: member.user.username,
				role: formatRole(role),
			}),
		})
	} catch (error) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.roleUpdateFailedTitle),
			text: formatErrorMessage(error),
		})
	}
}

function resendInvite(member: ServerAccessMember) {
	addNotification({
		type: 'error',
		title: formatMessage(messages.inviteResendUnavailableTitle),
		text: formatMessage(messages.inviteResendUnavailableText, { target: member.user.username }),
	})
}

async function cancelInvite(member: ServerAccessMember) {
	await removeMemberAccess(member, true)
}

function requestRemoveMember(member: ServerAccessMember) {
	pendingRemovalMember.value = member
	shouldCancelInvite.value = false
	removeMemberConfirmModal.value?.show()
}

function requestCancelInvite(member: ServerAccessMember) {
	pendingRemovalMember.value = member
	shouldCancelInvite.value = true
	removeMemberConfirmModal.value?.show()
}

async function confirmAccessRemoval() {
	const member = pendingRemovalMember.value
	const shouldCancel = shouldCancelInvite.value
	pendingRemovalMember.value = null
	shouldCancelInvite.value = false
	if (!member) return

	if (shouldCancel) {
		await cancelInvite(member)
		return
	}

	await removeMember(member)
}

async function removeMember(member: ServerAccessMember) {
	await removeMemberAccess(member, false)
}

async function removeMemberAccess(member: ServerAccessMember, shouldCancel: boolean) {
	try {
		await client.archon.server_users_v1.delete(serverId, member.user.id)
		await invalidateServerUsers()
		pushAuditEntry({
			world: null,
			action: { type: 'member_removed', target: member.user.username },
		})
		addNotification({
			type: 'success',
			title: formatMessage(
				shouldCancel ? messages.inviteCancelledTitle : messages.memberRemovedTitle,
			),
			text: formatMessage(
				shouldCancel ? messages.inviteCancelledText : messages.memberRemovedText,
				{
					target: member.user.username,
				},
			),
		})
	} catch (error) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.removeFailedTitle),
			text: formatErrorMessage(error),
		})
	}
}

async function grantAccess(payload: GrantServerAccessPayload) {
	const target = payload.target.trim()
	if (!target) return

	const existingMember = findMemberByTarget(target)
	if (existingMember) {
		await updateMemberRole(existingMember, payload.role)
		return
	}

	let user: Labrinth.Users.v2.User
	try {
		user = await client.labrinth.users_v2.get(target)
	} catch (error) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.userLookupFailedTitle),
			text: formatErrorMessage(error) ?? formatMessage(messages.userLookupFailedText, { target }),
		})
		return
	}

	const resolvedMember = findMemberByTarget(user.id)
	if (resolvedMember) {
		await updateMemberRole(resolvedMember, payload.role)
		return
	}

	try {
		await client.archon.server_users_v1.add(serverId, {
			user_id: user.id,
			role: accessRoleToApiRole(payload.role),
		})
		await invalidateServerUsers()
		pushAuditEntry({
			world: null,
			action: { type: 'member_invited', target: user.username, role: payload.role },
		})
		addNotification({
			type: 'success',
			title: formatMessage(messages.inviteSentTitle),
			text: formatMessage(messages.inviteSentText, {
				target: user.username,
				role: formatRole(payload.role),
			}),
		})
	} catch (error) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.inviteFailedTitle),
			text: formatErrorMessage(error),
		})
	}
}
</script>
