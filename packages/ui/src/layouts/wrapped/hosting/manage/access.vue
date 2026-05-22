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
					/>
				</template>
			</AuditLogTable>
		</div>

		<GrantAccessModal
			ref="grantAccessModal"
			:members="members"
			:resolve-user="resolveInviteUser"
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
import { Archon, type Labrinth } from '@modrinth/api-client'
import { FilterIcon, SearchIcon, UserPlusIcon } from '@modrinth/assets'
import { useInfiniteQuery, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, onBeforeUnmount, ref, watch } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import Combobox, { type ComboboxOption } from '#ui/components/base/Combobox.vue'
import DropdownFilterBar, {
	type DropdownFilterBarCategory,
	type DropdownFilterBarOption,
} from '#ui/components/base/DropdownFilterBar.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
import type {
	TimeFrameLastUnit,
	TimeFrameMode,
	TimeFramePreset,
} from '#ui/components/base/TimeFramePicker.vue'
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
	type ServerAuditLogEntry,
} from '#ui/components/servers/access'
import { parseAuditEvent } from '#ui/components/servers/access/events'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { useServerPermissions } from '#ui/composables/server-permissions'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'

type RoleFilter = ServerAccessRole | 'all'
type AuditLogFilterKey = 'users' | 'worlds' | 'actions'

const ACTION_LOG_PAGE_SIZE = 200
const ACTION_LOG_FILTER_OVERLAY_MS = 750
const SUPPORT_ACTION_LOG_USER_FILTER = 'support'
const SERVER_SCOPED_ACTION_LOG_WORLD_FILTER = '__server_scoped__'

const { formatMessage } = useVIntl()
const client = injectModrinthClient()
const { serverId, serverFull } = injectModrinthServerContext()
const { addNotification } = injectNotificationManager()
const queryClient = useQueryClient()
const grantAccessModal = ref<InstanceType<typeof GrantAccessModal> | null>(null)
const removeMemberConfirmModal = ref<InstanceType<typeof RemoveAccessModal> | null>(null)
const pendingRemovalMember = ref<ServerAccessMember | null>(null)
const shouldCancelInvite = ref(false)
const UserScope = Archon.ServerUsers.v1.UserScope
const editorScopes = [
	UserScope.BASE_READ,
	UserScope.POWER_ACTIONS,
	UserScope.FILES_WRITE,
	UserScope.SETUP,
	UserScope.BACKUPS,
	UserScope.ADVANCED,
]
const viewerScopes = [UserScope.BASE_READ, UserScope.POWER_ACTIONS]

const { canManageUsers, permissionDeniedMessage } = useServerPermissions()
const manageUsersActionTooltip = computed(() =>
	canManageUsers.value ? undefined : permissionDeniedMessage.value,
)

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
	addFilter: {
		id: 'servers.access-page.activity-log-filter.add',
		defaultMessage: 'Add filter',
	},
	clearFilters: {
		id: 'servers.access-page.activity-log-filter.clear',
		defaultMessage: 'Clear filters',
	},
	emptyFilterOptions: {
		id: 'servers.access-page.activity-log-filter.empty-options',
		defaultMessage: 'No options available.',
	},
	emptyFilterSearch: {
		id: 'servers.access-page.activity-log-filter.empty-search',
		defaultMessage: 'No options found.',
	},
	userFilter: {
		id: 'servers.access-page.activity-log-filter.users',
		defaultMessage: 'Actors',
	},
	supportActor: {
		id: 'servers.access-page.activity-log-filter.support-actor',
		defaultMessage: 'Support',
	},
	instanceFilter: {
		id: 'servers.access-page.activity-log-filter.instances',
		defaultMessage: 'Instances',
	},
	serverScopedInstance: {
		id: 'servers.access-page.activity-log-filter.server-scoped-instance',
		defaultMessage: 'Server',
	},
	actionTypeFilter: {
		id: 'servers.access-page.activity-log-filter.action-types',
		defaultMessage: 'Action types',
	},
	actionTypeFilterSearch: {
		id: 'servers.access-page.activity-log-filter.action-types-search',
		defaultMessage: 'Search action types...',
	},
	server_created: {
		id: 'servers.access-page.activity-log-filter.action.server-created',
		defaultMessage: 'Server created',
	},
	changed_server_name: {
		id: 'servers.access-page.activity-log-filter.action.changed-server-name',
		defaultMessage: 'Server name changed',
	},
	changed_server_subdomain: {
		id: 'servers.access-page.activity-log-filter.action.changed-server-subdomain',
		defaultMessage: 'Server subdomain changed',
	},
	server_reallocated: {
		id: 'servers.access-page.activity-log-filter.action.server-reallocated',
		defaultMessage: 'Server reallocated',
	},
	server_plan_changed: {
		id: 'servers.access-page.activity-log-filter.action.server-plan-changed',
		defaultMessage: 'Plan changed',
	},
	user_invited: {
		id: 'servers.access-page.activity-log-filter.action.user-invited',
		defaultMessage: 'User invited',
	},
	user_invite_revoked: {
		id: 'servers.access-page.activity-log-filter.action.user-invite-revoked',
		defaultMessage: 'User invite revoked',
	},
	user_permission_modified: {
		id: 'servers.access-page.activity-log-filter.action.user-permission-modified',
		defaultMessage: 'User permissions changed',
	},
	user_removed: {
		id: 'servers.access-page.activity-log-filter.action.user-removed',
		defaultMessage: 'User removed',
	},
	addon_added: {
		id: 'servers.access-page.activity-log-filter.action.addon-added',
		defaultMessage: 'Content added',
	},
	addon_uploaded: {
		id: 'servers.access-page.activity-log-filter.action.addon-uploaded',
		defaultMessage: 'Content uploaded',
	},
	addon_disabled: {
		id: 'servers.access-page.activity-log-filter.action.addon-disabled',
		defaultMessage: 'Content disabled',
	},
	addon_enabled: {
		id: 'servers.access-page.activity-log-filter.action.addon-enabled',
		defaultMessage: 'Content enabled',
	},
	addon_deleted: {
		id: 'servers.access-page.activity-log-filter.action.addon-deleted',
		defaultMessage: 'Content deleted',
	},
	addon_updated: {
		id: 'servers.access-page.activity-log-filter.action.addon-updated',
		defaultMessage: 'Content updated',
	},
	modpack_changed: {
		id: 'servers.access-page.activity-log-filter.action.modpack-changed',
		defaultMessage: 'Modpack changed',
	},
	modpack_unlinked: {
		id: 'servers.access-page.activity-log-filter.action.modpack-unlinked',
		defaultMessage: 'Modpack unlinked',
	},
	server_repaired: {
		id: 'servers.access-page.activity-log-filter.action.server-repaired',
		defaultMessage: 'Server repaired',
	},
	server_reset: {
		id: 'servers.access-page.activity-log-filter.action.server-reset',
		defaultMessage: 'Server reset',
	},
	server_started: {
		id: 'servers.access-page.activity-log-filter.action.server-started',
		defaultMessage: 'Server started',
	},
	server_stopped: {
		id: 'servers.access-page.activity-log-filter.action.server-stopped',
		defaultMessage: 'Server stopped',
	},
	server_restarted: {
		id: 'servers.access-page.activity-log-filter.action.server-restarted',
		defaultMessage: 'Server restarted',
	},
	server_killed: {
		id: 'servers.access-page.activity-log-filter.action.server-killed',
		defaultMessage: 'Server killed',
	},
	port_allocation_added: {
		id: 'servers.access-page.activity-log-filter.action.port-allocation-added',
		defaultMessage: 'Port allocation added',
	},
	port_allocation_removed: {
		id: 'servers.access-page.activity-log-filter.action.port-allocation-removed',
		defaultMessage: 'Port allocation removed',
	},
	loader_version_edited: {
		id: 'servers.access-page.activity-log-filter.action.loader-version-edited',
		defaultMessage: 'Loader version changed',
	},
	game_version_edited: {
		id: 'servers.access-page.activity-log-filter.action.game-version-edited',
		defaultMessage: 'Minecraft version changed',
	},
	server_properties_modified: {
		id: 'servers.access-page.activity-log-filter.action.server-properties-modified',
		defaultMessage: 'Server properties modified',
	},
	file_uploaded: {
		id: 'servers.access-page.activity-log-filter.action.file-uploaded',
		defaultMessage: 'File uploaded',
	},
	file_deleted: {
		id: 'servers.access-page.activity-log-filter.action.file-deleted',
		defaultMessage: 'File deleted',
	},
	file_renamed: {
		id: 'servers.access-page.activity-log-filter.action.file-renamed',
		defaultMessage: 'File renamed',
	},
	file_edited: {
		id: 'servers.access-page.activity-log-filter.action.file-edited',
		defaultMessage: 'File edited',
	},
	sftp_login: {
		id: 'servers.access-page.activity-log-filter.action.sftp-login',
		defaultMessage: 'SFTP login',
	},
	console_command_executed: {
		id: 'servers.access-page.activity-log-filter.action.console-command-executed',
		defaultMessage: 'Console command run',
	},
	console_cleared: {
		id: 'servers.access-page.activity-log-filter.action.console-cleared',
		defaultMessage: 'Console cleared',
	},
	backup_created: {
		id: 'servers.access-page.activity-log-filter.action.backup-created',
		defaultMessage: 'Backup created',
	},
	backup_renamed: {
		id: 'servers.access-page.activity-log-filter.action.backup-renamed',
		defaultMessage: 'Backup renamed',
	},
	backup_restored: {
		id: 'servers.access-page.activity-log-filter.action.backup-restored',
		defaultMessage: 'Backup restored',
	},
	backup_deleted: {
		id: 'servers.access-page.activity-log-filter.action.backup-deleted',
		defaultMessage: 'Backup deleted',
	},
	startup_command_modified: {
		id: 'servers.access-page.activity-log-filter.action.startup-command-modified',
		defaultMessage: 'Startup command changed',
	},
	java_runtime_modified: {
		id: 'servers.access-page.activity-log-filter.action.java-runtime-modified',
		defaultMessage: 'Java runtime changed',
	},
	java_version_modified: {
		id: 'servers.access-page.activity-log-filter.action.java-version-modified',
		defaultMessage: 'Java version changed',
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
		defaultMessage: 'Invite could not be sent',
	},
	friendRequestFailedTitle: {
		id: 'servers.access-page.notification.friend-request-failed.title',
		defaultMessage: 'Friend request could not be sent',
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

const actionLogActionNames = [
	'server_created',
	'changed_server_name',
	'changed_server_subdomain',
	'server_reallocated',
	'server_plan_changed',
	'user_invited',
	'user_invite_revoked',
	'user_permission_modified',
	'user_removed',
	'addon_added',
	'addon_uploaded',
	'addon_disabled',
	'addon_enabled',
	'addon_deleted',
	'addon_updated',
	'modpack_changed',
	'modpack_unlinked',
	'server_repaired',
	'server_reset',
	'server_started',
	'server_stopped',
	'server_restarted',
	'server_killed',
	'port_allocation_added',
	'port_allocation_removed',
	'loader_version_edited',
	'game_version_edited',
	'server_properties_modified',
	'file_uploaded',
	'file_deleted',
	'file_renamed',
	'file_edited',
	'sftp_login',
	'console_command_executed',
	'console_cleared',
	'backup_created',
	'backup_renamed',
	'backup_restored',
	'backup_deleted',
	'startup_command_modified',
	'java_runtime_modified',
	'java_version_modified',
] as const satisfies readonly Archon.Actions.v1.ActionName[]
type ActionLogFilterActionName = (typeof actionLogActionNames)[number]
const actionLogActionNameSet = new Set<string>(actionLogActionNames)

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

const members = computed<ServerAccessMember[]>(() =>
	(serverUsersQuery.data.value ?? [])
		.map((serverUser) => {
			const userId = serverUser.user.id
			const username = serverUser.user.username || userId
			const role = apiPermissionsToAccessRole(serverUser.permissions)

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
				isOwner: role === 'owner',
			}
		})
		.sort((a, b) => {
			const ownerSort = Number(b.isOwner) - Number(a.isOwner)
			return ownerSort === 0 ? a.user.username.localeCompare(b.user.username) : ownerSort
		}),
)

const worldOptions = computed(
	() => serverFull.value?.worlds.map((world) => ({ id: world.id, name: world.name })) ?? [],
)
const isAuditLogWorldFilterVisible = computed(() => worldOptions.value.length > 0)

const worldById = computed(
	() => new Map(worldOptions.value.map((world) => [world.id, world] as const)),
)

const backupById = computed(() => {
	const backups = new Map<string, Archon.Backups.v1.Backup>()
	for (const world of serverFull.value?.worlds ?? []) {
		for (const backup of world.backups ?? []) {
			backups.set(backup.id, backup)
		}
	}
	return backups
})

const memberSearch = ref('')
const roleFilter = ref<RoleFilter>('all')
const auditLogFilters = ref<Record<string, string[]>>({
	users: [],
	worlds: [],
	actions: [],
})
const auditLogTimeframeMode = ref<TimeFrameMode>('preset')
const auditLogTimeframePreset = ref<TimeFramePreset>('last_30_days')
const auditLogTimeframeLastAmount = ref(30)
const auditLogTimeframeLastUnit = ref<TimeFrameLastUnit>('days')
const auditLogTimeframeCustomStartDate = ref('')
const auditLogTimeframeCustomEndDate = ref('')
const auditLogSortDirection = ref<Archon.Actions.v1.SortOrder>('desc')

const actionLogDateFilter = computed(() => {
	const range = getAuditLogTimeframeRange()

	return {
		min_datetime: range?.start.toISOString(),
		max_datetime: range?.end.toISOString(),
	}
})

const actionLogEndpointFilter = computed<Archon.Actions.v1.ActionLogFilter | undefined>(() => {
	const users = selectedAuditLogFilterValues('users')
	const worlds = isAuditLogWorldFilterVisible.value ? selectedAuditLogWorldFilterValues() : []
	const actions = selectedAuditLogFilterValues('actions').filter(isActionLogActionName)
	const filter: Archon.Actions.v1.ActionLogFilter = {}

	if (users.length > 0) filter.users = users
	if (worlds.length > 0) filter.worlds = worlds
	if (actions.length > 0) filter.actions = actions

	return Object.keys(filter).length > 0 ? filter : undefined
})
const actionLogBaseQueryKey = ['servers', 'action-log', 'v1', 'infinite', serverId] as const
const actionLogQueryKey = computed(() => {
	const filter = actionLogEndpointFilter.value
	const dateFilter = actionLogDateFilter.value

	return [
		...actionLogBaseQueryKey,
		filter ?? null,
		dateFilter.min_datetime ?? null,
		dateFilter.max_datetime ?? null,
		auditLogSortDirection.value,
	]
})
const actionLogQuery = useInfiniteQuery({
	queryKey: actionLogQueryKey,
	queryFn: ({ pageParam = 0 }) => {
		const offset = typeof pageParam === 'number' ? pageParam : 0
		return client.archon.actions_v1.list(serverId, {
			limit: ACTION_LOG_PAGE_SIZE,
			offset,
			order: auditLogSortDirection.value,
			filter: actionLogEndpointFilter.value,
			...actionLogDateFilter.value,
		})
	},
	getNextPageParam: (lastPage) =>
		typeof lastPage.next_offset === 'number' ? lastPage.next_offset : undefined,
	initialPageParam: 0,
	placeholderData: (previousData) => previousData,
	staleTime: 30_000,
})
const actionLogFilterSignature = computed(() =>
	JSON.stringify([
		actionLogEndpointFilter.value ?? null,
		actionLogDateFilter.value.min_datetime ?? null,
		actionLogDateFilter.value.max_datetime ?? null,
	]),
)
const isActionLogFilterTransitioning = ref(false)
const isActionLogSortTransitioning = ref(false)
let actionLogFilterTransitionTimeout: ReturnType<typeof setTimeout> | null = null
let actionLogSortTransitionTimeout: ReturnType<typeof setTimeout> | null = null

watch(actionLogFilterSignature, (_signature, previousSignature) => {
	if (previousSignature === undefined) return
	startActionLogFilterTransition()
})

watch(
	auditLogSortDirection,
	(_direction, previousDirection) => {
		if (previousDirection === undefined) return
		startActionLogSortTransition()
	},
	{ flush: 'sync' },
)

watch(
	() => actionLogQuery.isFetching.value,
	(isFetching) => {
		if (!isFetching && isActionLogSortTransitioning.value) {
			finishActionLogSortTransition()
		}
	},
	{ flush: 'post' },
)

onBeforeUnmount(() => {
	if (actionLogFilterTransitionTimeout) {
		clearTimeout(actionLogFilterTransitionTimeout)
	}
	if (actionLogSortTransitionTimeout) {
		clearTimeout(actionLogSortTransitionTimeout)
	}
})

const auditEntries = computed<ServerAuditLogEntry[]>(() => {
	const pages = actionLogQuery.data.value?.pages ?? []
	const entryIdCounts = new Map<string, number>()

	return pages.flatMap((actionLog) =>
		actionLog.data.map((entry) => {
			const entryId = getActionLogEntryId(entry)
			const entryIdCount = entryIdCounts.get(entryId) ?? 0
			entryIdCounts.set(entryId, entryIdCount + 1)

			return apiActionLogEntryToAuditEntry(
				entry,
				actionLog,
				entryIdCount === 0 ? entryId : `${entryId}-${entryIdCount}`,
			)
		}),
	)
})
const hasShownLoadError = ref(false)
const hasShownActionLogLoadError = ref(false)
const hasMoreActionLogEntries = computed(
	() => !actionLogQuery.isPlaceholderData.value && actionLogQuery.hasNextPage.value,
)
const isLoadingMoreActionLogEntries = computed(() => actionLogQuery.isFetchingNextPage.value)
const isActionLogFiltering = computed(() => isActionLogFilterTransitioning.value)

const auditLogUserFilterOptions = computed<DropdownFilterBarOption[]>(() => {
	const options = new Map<string, DropdownFilterBarOption>()

	for (const page of actionLogQuery.data.value?.pages ?? []) {
		for (const entry of page.data) {
			if (entry.actor.type === 'support') {
				const userId = entry.actor.user_id ?? null
				const user = userId ? page.users[userId] : undefined
				if (!options.has(SUPPORT_ACTION_LOG_USER_FILTER)) {
					options.set(SUPPORT_ACTION_LOG_USER_FILTER, {
						value: SUPPORT_ACTION_LOG_USER_FILTER,
						label: user?.username
							? `${formatMessage(messages.supportActor)} (${user.username})`
							: formatMessage(messages.supportActor),
						searchTerms: [
							SUPPORT_ACTION_LOG_USER_FILTER,
							formatMessage(messages.supportActor),
							userId,
							user?.username,
						].filter(Boolean) as string[],
					})
				}
				continue
			}

			const id = entry.actor.user_id
			const user = page.users[id]
			if (!options.has(id)) {
				options.set(id, {
					value: id,
					label: user?.username ?? id,
					searchTerms: [id, user?.username].filter(Boolean) as string[],
				})
			}
		}
	}

	return [...options.values()].sort(compareFilterOptions)
})

const auditLogWorldFilterOptions = computed<DropdownFilterBarOption[]>(() => [
	{
		value: SERVER_SCOPED_ACTION_LOG_WORLD_FILTER,
		label: formatMessage(messages.serverScopedInstance),
		searchTerms: [
			SERVER_SCOPED_ACTION_LOG_WORLD_FILTER,
			formatMessage(messages.serverScopedInstance),
		],
	},
	...worldOptions.value.map((world) => ({
		value: world.id,
		label: world.name,
		searchTerms: [world.id, world.name],
	})),
])

const auditLogActionFilterOptions = computed<DropdownFilterBarOption[]>(() =>
	actionLogActionNames.map((action) => ({
		value: action,
		label: formatActionLogAction(action),
		searchTerms: [action, action.replaceAll('_', ' ')],
	})),
)

const auditLogFilterCategories = computed<DropdownFilterBarCategory[]>(() => {
	const categories: DropdownFilterBarCategory[] = [
		{
			key: 'users',
			label: formatMessage(messages.userFilter),
			options: auditLogUserFilterOptions.value,
		},
	]

	if (isAuditLogWorldFilterVisible.value) {
		categories.push({
			key: 'worlds',
			label: formatMessage(messages.instanceFilter),
			options: auditLogWorldFilterOptions.value,
		})
	}

	categories.push({
		key: 'actions',
		label: formatMessage(messages.actionTypeFilter),
		options: auditLogActionFilterOptions.value,
		searchable: true,
		searchPlaceholder: formatMessage(messages.actionTypeFilterSearch),
		submenuClass: 'w-[22rem]',
		previewDropdownMinWidth: '20rem',
	})

	return categories
})

const hasActiveAuditLogDateFilter = computed(
	() => !!actionLogDateFilter.value.min_datetime || !!actionLogDateFilter.value.max_datetime,
)

const hasActiveAuditLogFilters = computed(
	() =>
		hasActiveAuditLogDateFilter.value ||
		(isAuditLogWorldFilterVisible.value
			? (['users', 'worlds', 'actions'] satisfies AuditLogFilterKey[])
			: (['users', 'actions'] satisfies AuditLogFilterKey[])
		).some((key) => selectedAuditLogFilterValues(key).length > 0),
)

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

function selectedAuditLogFilterValues(key: AuditLogFilterKey): string[] {
	const values = auditLogFilters.value[key]
	return values ? [...values] : []
}

function selectedAuditLogWorldFilterValues(): Array<string | null> {
	return selectedAuditLogFilterValues('worlds').map((world) =>
		world === SERVER_SCOPED_ACTION_LOG_WORLD_FILTER ? null : world,
	)
}

function parseDateInputValue(value: string) {
	const [yearValue, monthValue, dayValue] = value.split('-').map(Number)
	if (!yearValue || !monthValue || !dayValue) return null

	const date = new Date(yearValue, monthValue - 1, dayValue)
	if (
		date.getFullYear() !== yearValue ||
		date.getMonth() !== monthValue - 1 ||
		date.getDate() !== dayValue
	) {
		return null
	}

	return date
}

function parseDateTimeInputValue(value: string) {
	const date = new Date(value)
	return Number.isNaN(date.getTime()) ? null : date
}

function addDays(date: Date, days: number) {
	const nextDate = new Date(date)
	nextDate.setDate(nextDate.getDate() + days)
	return nextDate
}

function subtractCalendarMonths(date: Date, months: number) {
	const nextDate = new Date(date)
	const day = nextDate.getDate()
	nextDate.setDate(1)
	nextDate.setMonth(nextDate.getMonth() - months)
	const daysInMonth = new Date(nextDate.getFullYear(), nextDate.getMonth() + 1, 0).getDate()
	nextDate.setDate(Math.min(day, daysInMonth))
	return nextDate
}

function getRoundedNow() {
	const now = Date.now()
	return new Date(Math.floor(now / 60000) * 60000)
}

function getAuditLogTimeframeRange(): { start: Date; end: Date } | null {
	const now = getRoundedNow()

	if (auditLogTimeframeMode.value === 'last') {
		return getLastAuditLogTimeframeRange(now)
	}

	if (auditLogTimeframeMode.value === 'custom_range') {
		const startDate = parseDateInputValue(auditLogTimeframeCustomStartDate.value)
		const endDate = parseDateInputValue(auditLogTimeframeCustomEndDate.value)
		if (!startDate || !endDate) return null

		const [minDate, maxDate] =
			startDate.getTime() > endDate.getTime() ? [endDate, startDate] : [startDate, endDate]

		return {
			start: startOfDay(minDate),
			end: endOfDay(maxDate),
		}
	}

	if (auditLogTimeframeMode.value === 'custom_datetime_range') {
		const startDate = parseDateTimeInputValue(auditLogTimeframeCustomStartDate.value)
		const endDate = parseDateTimeInputValue(auditLogTimeframeCustomEndDate.value)
		if (!startDate || !endDate) return null

		return startDate.getTime() > endDate.getTime()
			? { start: endDate, end: startDate }
			: { start: startDate, end: endDate }
	}

	return getPresetAuditLogTimeframeRange(now)
}

function getPresetAuditLogTimeframeRange(now: Date): { start: Date; end: Date } | null {
	switch (auditLogTimeframePreset.value) {
		case 'today':
			return { start: startOfDay(now), end: endOfDay(now) }
		case 'yesterday': {
			const yesterday = addDays(now, -1)
			return { start: startOfDay(yesterday), end: endOfDay(yesterday) }
		}
		case 'last_7_days':
			return { start: startOfDay(addDays(now, -6)), end: endOfDay(now) }
		case 'last_14_days':
			return { start: startOfDay(addDays(now, -13)), end: endOfDay(now) }
		case 'last_30_days':
			return { start: startOfDay(addDays(now, -29)), end: endOfDay(now) }
		case 'last_90_days':
			return { start: startOfDay(addDays(now, -89)), end: endOfDay(now) }
		case 'last_180_days':
			return { start: startOfDay(addDays(now, -179)), end: endOfDay(now) }
		case 'year_to_date':
			return { start: new Date(now.getFullYear(), 0, 1), end: endOfDay(now) }
		case 'all_time':
			return null
	}
}

function getLastAuditLogTimeframeRange(now: Date): { start: Date; end: Date } {
	const amount = Math.max(1, Math.floor(auditLogTimeframeLastAmount.value))

	switch (auditLogTimeframeLastUnit.value) {
		case 'hours':
			return { start: new Date(now.getTime() - amount * 60 * 60 * 1000), end: now }
		case 'days':
			return { start: new Date(now.getTime() - amount * 24 * 60 * 60 * 1000), end: now }
		case 'weeks':
			return { start: new Date(now.getTime() - amount * 7 * 24 * 60 * 60 * 1000), end: now }
		case 'months':
			return { start: subtractCalendarMonths(now, amount), end: now }
	}
}

function startOfDay(date: Date) {
	return new Date(date.getFullYear(), date.getMonth(), date.getDate())
}

function endOfDay(date: Date) {
	return new Date(date.getFullYear(), date.getMonth(), date.getDate(), 23, 59, 59, 999)
}

function isActionLogActionName(action: string): action is ActionLogFilterActionName {
	return actionLogActionNameSet.has(action)
}

function compareFilterOptions(left: DropdownFilterBarOption, right: DropdownFilterBarOption) {
	return left.label.localeCompare(right.label)
}

function formatActionLogAction(action: ActionLogFilterActionName): string {
	return formatMessage(messages[action])
}

function loadMoreActionLogEntries() {
	if (
		isActionLogFilterTransitioning.value ||
		actionLogQuery.isPlaceholderData.value ||
		!actionLogQuery.hasNextPage.value ||
		actionLogQuery.isFetchingNextPage.value
	) {
		return
	}
	void actionLogQuery.fetchNextPage()
}

function startActionLogFilterTransition() {
	isActionLogFilterTransitioning.value = true

	if (actionLogFilterTransitionTimeout) {
		clearTimeout(actionLogFilterTransitionTimeout)
	}

	actionLogFilterTransitionTimeout = setTimeout(() => {
		isActionLogFilterTransitioning.value = false
		actionLogFilterTransitionTimeout = null
	}, ACTION_LOG_FILTER_OVERLAY_MS)
}

function startActionLogSortTransition() {
	isActionLogSortTransitioning.value = true

	if (actionLogSortTransitionTimeout) {
		clearTimeout(actionLogSortTransitionTimeout)
	}

	actionLogSortTransitionTimeout = setTimeout(() => {
		isActionLogSortTransitioning.value = false
		actionLogSortTransitionTimeout = null
	}, 2500)
}

function finishActionLogSortTransition() {
	if (actionLogSortTransitionTimeout) {
		clearTimeout(actionLogSortTransitionTimeout)
	}

	actionLogSortTransitionTimeout = setTimeout(() => {
		isActionLogSortTransitioning.value = false
		actionLogSortTransitionTimeout = null
	}, 120)
}

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

watch(
	() => actionLogQuery.error.value,
	(actionLogError) => {
		if (hasShownActionLogLoadError.value || !actionLogError) return

		hasShownActionLogLoadError.value = true
		addNotification({
			type: 'error',
			title: formatMessage(messages.loadFailedTitle),
			text: formatErrorMessage(actionLogError) ?? formatMessage(messages.loadFailedText),
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

function serializeUserScope(scopes: string[]): Archon.ServerUsers.v1.UserScope {
	return scopes.join(' | ')
}

function formatErrorMessage(error: unknown): string | undefined {
	return error instanceof Error ? error.message : undefined
}

function apiActionLogEntryToAuditEntry(
	entry: Archon.Actions.v1.ActionEntry,
	actionLog: Archon.Actions.v1.ActionLogResponse,
	id: string,
): ServerAuditLogEntry {
	const event = parseAuditEvent(entry, {
		serverId,
		users: actionLog.users,
		addons: actionLog.addons,
		worldById: worldById.value,
		backupById: backupById.value,
		versions: actionLog.versions ?? {},
	})

	return {
		id,
		actor: event.props.actor,
		world: event.props.world,
		event,
		timestamp: entry.timestamp,
	}
}

function getActionLogEntryId(entry: Archon.Actions.v1.ActionEntry) {
	return JSON.stringify([
		entry.timestamp,
		entry.actor.type,
		entry.actor.type === 'user' ? entry.actor.user_id : (entry.actor.user_id ?? 'support'),
		entry.server_id,
		entry.world_id ?? null,
		entry.action.action,
		stableStringify(entry.action.metadata),
	])
}

function stableStringify(value: unknown): string {
	if (value === undefined) {
		return 'undefined'
	}

	if (value === null || typeof value !== 'object') {
		return JSON.stringify(value) ?? String(value)
	}

	if (Array.isArray(value)) {
		return `[${value.map((item) => stableStringify(item)).join(',')}]`
	}

	return `{${Object.entries(value as Record<string, unknown>)
		.sort(([leftKey], [rightKey]) => leftKey.localeCompare(rightKey))
		.map(([key, item]) => `${JSON.stringify(key)}:${stableStringify(item)}`)
		.join(',')}}`
}

async function invalidateServerUsers() {
	await queryClient.invalidateQueries({ queryKey: serverUsersQueryKey })
}

async function invalidateActionLog() {
	await queryClient.invalidateQueries({ queryKey: actionLogBaseQueryKey })
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

async function resolveInviteUser(target: string): Promise<ServerAccessInviteSuggestion | null> {
	const user = await client.labrinth.users_v2.get(target)
	return {
		id: user.id,
		username: user.username,
		avatarUrl: user.avatar_url || undefined,
	}
}

function resolveMemberUserId(member: ServerAccessMember): string {
	return member.user.id
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

	const existingMember = findMemberByTarget(target)
	if (existingMember) {
		await updateMemberRole(existingMember, payload.role)
		if (payload.addAsFriend) {
			const userId = await resolveMemberUserId(existingMember)
			await sendFriendRequest(userId)
		}
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
		if (payload.addAsFriend) await sendFriendRequest(user.id)
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
	try {
		await client.labrinth.friends_v3.add(userIdOrUsername)
	} catch (error) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.friendRequestFailedTitle),
			text: formatErrorMessage(error),
		})
	}
}
</script>
