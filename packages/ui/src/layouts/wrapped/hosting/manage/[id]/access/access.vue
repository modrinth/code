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
			<div class="flex shrink-0 items-center gap-2 flex-wrap md:flex-nowrap">
				<Combobox
					v-model="roleFilter"
					:options="roleFilterOptions"
					:display-value="selectedRoleFilterLabel"
					trigger-class="min-w-[225px] !h-10 !min-h-10 !py-0"
				>
					<template #prefix>
						<FilterIcon class="size-5 text-secondary" aria-hidden="true" />
					</template>
				</Combobox>
				<ButtonStyled color="brand">
					<button
						v-tooltip="manageUsersActionTooltip"
						class="!h-10 w-full md:w-fit"
						:disabled="!canManageUsers"
						@click="grantAccessModal?.show($event)"
					>
						<UserPlusIcon aria-hidden="true" />
						{{ formatMessage(messages.inviteFriends) }}
					</button>
				</ButtonStyled>
			</div>
		</div>

		<AccessTable
			:members="filteredMembers"
			:roles="roleOptions"
			:can-manage-users="canManageUsers"
			:permission-denied-message="permissionDeniedMessage"
			:user-profile-link="props.userProfileLink"
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
				v-model:sort-direction="auditLogSortDirection"
				v-model:timeframe-mode="auditLogTimeframeMode"
				v-model:timeframe-preset="auditLogTimeframePreset"
				v-model:timeframe-last-amount="auditLogTimeframeLastAmount"
				v-model:timeframe-last-unit="auditLogTimeframeLastUnit"
				v-model:timeframe-custom-start-date="auditLogTimeframeCustomStartDate"
				v-model:timeframe-custom-end-date="auditLogTimeframeCustomEndDate"
				:entries="auditEntries"
				:has-active-external-filters="hasActiveAuditLogFilters"
				:has-more="hasMoreActionLogEntries"
				:loading="isActionLogFiltering"
				:loading-more="isLoadingMoreActionLogEntries"
				:show-world-column="showAuditLogInstances"
				:suppress-row-transitions="isActionLogSortTransitioning"
				@load-more="loadMoreActionLogEntries"
			>
				<template #filters>
					<DropdownFilterBar
						v-model="auditLogFilters"
						:categories="auditLogFilterCategories"
						:add-label="formatMessage(messages.addFilter)"
						:clear-label="formatMessage(messages.clearFilters)"
						:empty-options-label="formatMessage(messages.emptyFilterOptions)"
						:empty-search-label="formatMessage(messages.emptyFilterSearch)"
						apply-immediately
						use-filter-icon
						checkbox-position="right"
					/>
				</template>
			</AuditLogTable>
		</div>

		<GrantAccessModal
			ref="grantAccessModal"
			:members="members"
			:friend-ids="friendIds"
			:search-users="searchInviteUsers"
			:can-grant="canManageUsers"
			:permission-denied-message="permissionDeniedMessage"
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
			:can-remove="canManageUsers"
			:permission-denied-message="permissionDeniedMessage"
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
import DropdownFilterBar from '#ui/components/base/DropdownFilterBar.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
import {
	AccessTable,
	apiPermissionsToAccessRole,
	AuditLogTable,
	GrantAccessModal,
	type GrantServerAccessPayload,
	RemoveAccessModal,
	type ServerAccessInviteSuggestion,
	type ServerAccessMember,
	type ServerAccessRole,
	type ServerAccessRoleOption,
	type ServerAccessUserProfileLink,
} from '#ui/components/servers/access'
import { useVIntl } from '#ui/composables/i18n'
import { useServerPermissions } from '#ui/composables/server-permissions'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'

import { useAccessAuditLog } from './audit-log'
import { accessMessages } from './messages'

type RoleFilter = ServerAccessRole | 'all'

const props = withDefaults(
	defineProps<{
		showAuditLogInstances?: boolean
		userProfileLink?: (username: string) => ServerAccessUserProfileLink
	}>(),
	{
		showAuditLogInstances: false,
	},
)
const showAuditLogInstances = computed(() => props.showAuditLogInstances)

const INVITE_RESEND_COOLDOWN_SECONDS = 2 * 60

const { formatMessage } = useVIntl()
const client = injectModrinthClient()
const { serverId, serverFull } = injectModrinthServerContext()
const { addNotification } = injectNotificationManager()
const queryClient = useQueryClient()
const grantAccessModal = ref<InstanceType<typeof GrantAccessModal> | null>(null)
const removeMemberConfirmModal = ref<InstanceType<typeof RemoveAccessModal> | null>(null)
const pendingRemovalMember = ref<ServerAccessMember | null>(null)
const shouldCancelInvite = ref(false)
const reinviteCooldownUntilByUserId = ref<Record<string, number | undefined>>({})
const editorScopes = [
	'BASE_READ',
	'POWER_ACTIONS',
	'EXEC_COMMANDS',
	'FILES_WRITE',
	'SETUP',
	'BACKUPS',
	'ADVANCED',
] as const
const viewerScopes = ['BASE_READ', 'POWER_ACTIONS'] as const

const { canManageUsers, permissionDeniedMessage } = useServerPermissions()
const manageUsersActionTooltip = computed(() =>
	canManageUsers.value ? undefined : permissionDeniedMessage.value,
)

const messages = accessMessages

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
	staleTime: 30_000,
})

const friendsQueryKey = ['user', 'friends', 'v3']
const friendsQuery = useQuery({
	queryKey: friendsQueryKey,
	queryFn: () => client.labrinth.friends_v3.list(),
	staleTime: 30_000,
})
const friendIds = computed(() => getFriendRelationshipUserIds(friendsQuery.data.value ?? []))

const members = computed<ServerAccessMember[]>(() =>
	(serverUsersQuery.data.value ?? [])
		.map((serverUser) => {
			const userId = serverUser.user.id
			const username = serverUser.user.username || userId
			const role = apiPermissionsToAccessRole(serverUser.permissions)
			const nowReinviteAvailableAt = reinviteCooldownUntilByUserId.value[userId]
			const apiReinviteAvailableAt = getInviteResendAvailableAt(serverUser.last_invite_sent)
			const reinviteAvailableAt = [nowReinviteAvailableAt, apiReinviteAvailableAt].reduce(
				(candidate, current) =>
					candidate === undefined || (current !== undefined && current > candidate)
						? current
						: candidate,
			)

			return {
				id: `${serverId}-${userId}`,
				user: {
					id: userId,
					username,
					avatarUrl: serverUser.user.avatar_url || undefined,
				},
				role,
				joinedAt: serverUser.added_on ?? null,
				pending: !serverUser.added_on,
				inviteResendAvailableAt: reinviteAvailableAt
					? new Date(reinviteAvailableAt).toISOString()
					: undefined,
				isOwner: role === 'owner',
			}
		})
		.sort((a, b) => {
			const ownerSort = Number(b.isOwner) - Number(a.isOwner)
			return ownerSort === 0 ? a.user.username.localeCompare(b.user.username) : ownerSort
		}),
)

const memberSearch = ref('')
const roleFilter = ref<RoleFilter>('all')
const {
	auditEntries,
	auditLogFilterCategories,
	auditLogFilters,
	auditLogSortDirection,
	auditLogTimeframeCustomEndDate,
	auditLogTimeframeCustomStartDate,
	auditLogTimeframeLastAmount,
	auditLogTimeframeLastUnit,
	auditLogTimeframeMode,
	auditLogTimeframePreset,
	hasActiveAuditLogFilters,
	hasMoreActionLogEntries,
	invalidateActionLog,
	isActionLogFiltering,
	isActionLogSortTransitioning,
	isLoadingMoreActionLogEntries,
	loadMoreActionLogEntries,
} = useAccessAuditLog({
	client,
	serverId,
	serverFull,
	showAuditLogInstances,
	addNotification,
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

const hasShownLoadError = ref(false)

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

function accessRoleToApiPermissions(role: Exclude<ServerAccessRole, 'owner'>) {
	switch (role) {
		case 'editor':
			return serializeUserScope(editorScopes)
		case 'viewer':
			return serializeUserScope(viewerScopes)
	}
}

function serializeUserScope(scopes: readonly string[]): Archon.ServerUsers.v1.UserScope {
	return scopes.join(' | ')
}

function formatErrorMessage(error: unknown): string | undefined {
	return error instanceof Error ? error.message : undefined
}

function isSuppressedFriendRequestError(error: unknown) {
	return getErrorMessageParts(error).some((message) => {
		const normalizedMessage = message.toLowerCase()
		return (
			normalizedMessage.includes('you are already friends with this user') ||
			normalizedMessage.includes('you cannot add yourself as a friend') ||
			normalizedMessage.includes('you cannot accept your own friend request')
		)
	})
}

function getErrorMessageParts(error: unknown): string[] {
	const errorMessages: string[] = []

	if (error instanceof Error) {
		errorMessages.push(error.message)
	}

	if (!error || typeof error !== 'object') return errorMessages

	const record = error as Record<string, unknown>
	pushErrorDescription(errorMessages, record.responseData)
	pushErrorDescription(errorMessages, record.v1Error)

	return errorMessages
}

function pushErrorDescription(errorMessages: string[], value: unknown) {
	if (!value || typeof value !== 'object') return

	const record = value as Record<string, unknown>
	if (typeof record.description === 'string') {
		errorMessages.push(record.description)
	}
}

async function invalidateServerUsers() {
	await queryClient.invalidateQueries({ queryKey: serverUsersQueryKey })
}

function setCachedMemberRole(member: ServerAccessMember, role: Exclude<ServerAccessRole, 'owner'>) {
	const normalizedUserId = member.user.id.toLowerCase()
	const normalizedUsername = member.user.username.toLowerCase()

	queryClient.setQueryData<Archon.ServerUsers.v1.ServerUser[]>(serverUsersQueryKey, (serverUsers) =>
		serverUsers?.map((serverUser) => {
			const isTargetUser =
				serverUser.user.id.toLowerCase() === normalizedUserId ||
				serverUser.user.username.toLowerCase() === normalizedUsername

			return isTargetUser
				? { ...serverUser, permissions: accessRoleToApiPermissions(role) }
				: serverUser
		}),
	)
}

function findMemberByTarget(target: string) {
	const normalizedTarget = target.trim().toLowerCase()
	return members.value.find(
		(member) =>
			member.user.username.toLowerCase() === normalizedTarget ||
			member.user.id.toLowerCase() === normalizedTarget,
	)
}

async function searchInviteUsers(query: string): Promise<ServerAccessInviteSuggestion[]> {
	const users = await client.labrinth.users_v3.search(query)
	return users.map((user) => ({
		id: user.id,
		username: user.username,
		avatarUrl: user.avatar_url || undefined,
	}))
}

function resolveMemberUserId(member: ServerAccessMember): string {
	return member.user.id
}

function getInviteResendAvailableAt(lastInviteSent: string | null | undefined): number | undefined {
	if (!lastInviteSent) return undefined
	const lastInviteSentAt = new Date(lastInviteSent).getTime()
	if (Number.isNaN(lastInviteSentAt)) return undefined
	return lastInviteSentAt + INVITE_RESEND_COOLDOWN_SECONDS * 1000
}

function setReinviteCooldown(member: ServerAccessMember, cooldownSeconds: number | null) {
	if (!cooldownSeconds) {
		reinviteCooldownUntilByUserId.value[member.user.id] = undefined
		return
	}

	reinviteCooldownUntilByUserId.value[member.user.id] = Date.now() + cooldownSeconds * 1000
}

async function updateMemberRole(member: ServerAccessMember, role: ServerAccessRole) {
	if (!canManageUsers.value || member.isOwner || member.role === role || role === 'owner') return
	const previousRole = member.role
	if (previousRole === 'owner') return

	await queryClient.cancelQueries({ queryKey: serverUsersQueryKey })
	setCachedMemberRole(member, role)

	try {
		const userId = await resolveMemberUserId(member)
		await client.archon.server_users_v1.update(serverId, userId, accessRoleToApiRole(role))
	} catch (error) {
		setCachedMemberRole(member, previousRole)
		addNotification({
			type: 'error',
			title: formatMessage(messages.roleUpdateFailedTitle),
			text: formatErrorMessage(error),
		})
		return
	}

	await invalidateServerUsers()
	await invalidateActionLog()
}

async function resendInvite(member: ServerAccessMember) {
	if (!canManageUsers.value || !member.pending || member.role === 'owner') return

	try {
		const result = await client.archon.server_users_v1.reinvite(serverId, member.user.id)
		setReinviteCooldown(member, result.cooldown_seconds)

		if (!result.sent) return

		await invalidateServerUsers()
		await invalidateActionLog()
		addNotification({
			type: 'success',
			title: formatMessage(messages.inviteResentTitle),
			text: formatMessage(messages.inviteResentText, {
				target: member.user.username,
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

async function cancelInvite(member: ServerAccessMember) {
	await removeMemberAccess(member, true)
}

function requestRemoveMember(member: ServerAccessMember) {
	if (!canManageUsers.value) return
	pendingRemovalMember.value = member
	shouldCancelInvite.value = false
	removeMemberConfirmModal.value?.show()
}

function requestCancelInvite(member: ServerAccessMember) {
	if (!canManageUsers.value) return
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
	if (!canManageUsers.value) return

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
	if (!canManageUsers.value) return

	try {
		const userId = await resolveMemberUserId(member)
		await client.archon.server_users_v1.delete(serverId, userId)
		await invalidateServerUsers()
		await invalidateActionLog()
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
	if (!canManageUsers.value) return

	const target = payload.target.trim()
	if (!target) return

	const user = payload.user
	const existingMember =
		findMemberByTarget(user.id) ?? findMemberByTarget(user.username) ?? findMemberByTarget(target)
	if (existingMember) {
		await updateMemberRole(existingMember, payload.role)
		if (payload.addAsFriend) {
			await sendFriendRequest(user.id)
		}
		return
	}

	try {
		await client.archon.server_users_v1.add(serverId, {
			user_id: user.id,
			role: accessRoleToApiRole(payload.role),
		})
		await invalidateServerUsers()
		await invalidateActionLog()
		addNotification({
			type: 'success',
			title: formatMessage(messages.inviteSentTitle),
			text: formatMessage(messages.inviteSentText, {
				target: user.username,
				role: formatRole(payload.role),
			}),
		})
		if (payload.addAsFriend) await sendFriendRequest(user.id)
	} catch (error) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.inviteFailedTitle),
			text: formatErrorMessage(error),
		})
	}
}

async function sendFriendRequest(userIdOrUsername: string) {
	const friends = await queryClient.ensureQueryData({
		queryKey: friendsQueryKey,
		queryFn: () => client.labrinth.friends_v3.list(),
	})

	if (hasFriendRelationship(friends, userIdOrUsername)) return

	try {
		await client.labrinth.friends_v3.add(userIdOrUsername)
		await queryClient.invalidateQueries({ queryKey: friendsQueryKey })
	} catch (error) {
		if (isSuppressedFriendRequestError(error)) return

		addNotification({
			type: 'error',
			title: formatMessage(messages.friendRequestFailedTitle),
			text: formatErrorMessage(error),
		})
	}
}

function hasFriendRelationship(friends: Labrinth.Friends.v3.UserFriend[], userId: string) {
	return friends.some((friend) => friend.id === userId || friend.friend_id === userId)
}

function getFriendRelationshipUserIds(friends: Labrinth.Friends.v3.UserFriend[]) {
	return [...new Set(friends.flatMap((friend) => [friend.id, friend.friend_id]))]
}
</script>
