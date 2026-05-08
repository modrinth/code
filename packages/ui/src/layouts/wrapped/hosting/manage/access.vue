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
			:resolve-user="resolveInviteUser"
			@grant="grantAccess"
		/>
		<RemoveAccessModal
			ref="removeMemberConfirmModal"
			:username="pendingRemovalMember?.user.username ?? ''"
			:avatar-url="pendingRemovalMember?.user.avatarUrl"
			:role="pendingRemovalMember?.role"
			:joined-at="pendingRemovalMember?.joinedAt"
			:pending="pendingRemovalMember?.pending"
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
	type GrantServerAccessPayload,
	RemoveAccessModal,
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
const { serverId } = injectModrinthServerContext()
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
		defaultMessage: 'Manage instance content, files, backups, and other settings.',
	},
	viewerRole: {
		id: 'servers.access-page.role.viewer',
		defaultMessage: 'Limited',
	},
	viewerDescription: {
		id: 'servers.access-page.role.viewer-description',
		defaultMessage: 'Start, stop, and view the server without making changes.',
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

const members = computed<ServerAccessMember[]>(() =>
	(serverUsersQuery.data.value ?? [])
		.map((serverUser) => {
			const username = serverUser.user.username
			const role = apiPermissionsToAccessRole(serverUser.permissions)

			return {
				id: `${serverId}-${username}`,
				user: {
					id: username,
					username,
					avatarUrl: serverUser.user.avatar_url || undefined,
				},
				role,
				joinedAt: serverUser.added_on ?? null,
				pending: !serverUser.added_on && role !== 'owner',
				isOwner: role === 'owner',
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
	() => serverUsersQuery.error.value,
	(serverUsersError) => {
		if (hasShownLoadError.value || !serverUsersError) return

		hasShownLoadError.value = true
		addNotification({
			type: 'error',
			title: formatMessage(messages.loadFailedTitle),
			text: formatErrorMessage(serverUsersError) ?? formatMessage(messages.loadFailedText),
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

function apiPermissionsToAccessRole(
	permissions: Archon.ServerUsers.v1.UserScope,
): ServerAccessRole {
	const scopes = apiPermissionsToScopes(permissions)
	if (scopes.has('SERVER_ADMIN') || scopes.has('MANAGE_USERS')) return 'owner'
	if (
		scopes.has('FILES_WRITE') ||
		scopes.has('SETUP') ||
		scopes.has('BACKUPS') ||
		scopes.has('ADVANCED') ||
		scopes.has('RESET_SERVER')
	) {
		return 'editor'
	}
	return 'viewer'
}

function apiPermissionsToScopes(permissions: Archon.ServerUsers.v1.UserScope) {
	return new Set(
		permissions
			.split('|')
			.map((scope) => scope.trim())
			.filter(Boolean),
	)
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

async function resolveInviteUser(target: string): Promise<ServerAccessInviteSuggestion | null> {
	const user = await client.labrinth.users_v2.get(target)
	return {
		id: user.id,
		username: user.username,
		avatarUrl: user.avatar_url || undefined,
	}
}

async function resolveMemberUserId(member: ServerAccessMember): Promise<string> {
	try {
		const user = await client.labrinth.users_v2.get(member.user.username)
		return user.id
	} catch {
		return member.user.id
	}
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
		const userId = await resolveMemberUserId(member)
		await client.archon.server_users_v1.update(serverId, userId, accessRoleToApiRole(role))
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
		const userId = await resolveMemberUserId(member)
		await client.archon.server_users_v1.delete(serverId, userId)
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

	const resolvedMember = findMemberByTarget(user.id) ?? findMemberByTarget(user.username)
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
