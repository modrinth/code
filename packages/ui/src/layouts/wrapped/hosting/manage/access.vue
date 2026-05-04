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
				/>
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
			@cancel-invite="cancelInvite"
			@remove-member="removeMember"
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
	</div>
</template>

<script setup lang="ts">
import { SearchIcon, UserPlusIcon } from '@modrinth/assets'
import { computed, ref } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import Combobox, { type ComboboxOption } from '#ui/components/base/Combobox.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
import {
	AccessTable,
	AuditLogTable,
	GrantAccessModal,
	type GrantServerAccessPayload,
	type ServerAccessInviteSuggestion,
	type ServerAccessMember,
	type ServerAccessRole,
	type ServerAccessRoleOption,
	type ServerAuditLogEntry,
	type ServerAuditLogFilters,
} from '#ui/components/servers/access'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectNotificationManager } from '#ui/providers'

type RoleFilter = ServerAccessRole | 'all'

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()
const grantAccessModal = ref<InstanceType<typeof GrantAccessModal> | null>(null)

const messages = defineMessages({
	searchUsersPlaceholder: {
		id: 'servers.access-page.search-users-placeholder',
		defaultMessage: 'Search {count} {count, plural, one {user} other {users}}...',
	},
	inviteFriends: {
		id: 'servers.access-page.invite-friends',
		defaultMessage: 'Invite friends',
	},
	activityLogTitle: {
		id: 'servers.access-page.activity-log-title',
		defaultMessage: 'Activity log',
	},
	allRoles: {
		id: 'servers.access-page.role-filter.all',
		defaultMessage: 'Role: All',
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
		defaultMessage: 'Viewer',
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
})

const nowMinus = (amount: number, unit: 'hour' | 'day' | 'week' | 'month') => {
	const unitMs = {
		hour: 60 * 60 * 1000,
		day: 24 * 60 * 60 * 1000,
		week: 7 * 24 * 60 * 60 * 1000,
		month: 30 * 24 * 60 * 60 * 1000,
	}[unit]
	return new Date(Date.now() - amount * unitMs).toISOString()
}

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
		subLabel: role.description,
	})),
])

const selectedRoleFilterLabel = computed(
	() =>
		roleFilterOptions.value.find((option) => option.value === roleFilter.value)?.label ??
		formatMessage(messages.allRoles),
)

const members = ref<ServerAccessMember[]>([
	{
		id: 'member-owner',
		user: {
			id: 'user-prospector',
			username: 'Prospector',
		},
		role: 'owner',
		joinedAt: nowMinus(1, 'month'),
		isOwner: true,
	},
	{
		id: 'member-geometrically',
		user: {
			id: 'user-geometrically',
			username: 'Geometrically',
		},
		role: 'editor',
		joinedAt: nowMinus(3, 'week'),
	},
	{
		id: 'member-imb',
		user: {
			id: 'user-imb',
			username: 'IMB',
		},
		role: 'viewer',
		joinedAt: null,
		pending: true,
	},
])

const worldOptions = [
	{ id: 'world-create-smp', name: 'Create SMP' },
	{ id: 'world-smp-season-4', name: 'SMP Season 4' },
]

const auditEntries = ref<ServerAuditLogEntry[]>([
	{
		id: 'audit-support-file',
		actor: { id: 'support', username: 'Support' },
		world: null,
		action: { type: 'file_edited', file: 'server.properties' },
		timestamp: nowMinus(1, 'hour'),
	},
	{
		id: 'audit-world-start',
		actor: members.value[1].user,
		world: null,
		action: { type: 'world_started', worldName: 'Create SMP' },
		timestamp: nowMinus(5, 'hour'),
	},
	{
		id: 'audit-mod-install',
		actor: members.value[1].user,
		world: worldOptions[1],
		action: { type: 'content_installed', contentType: 'mod', name: 'Create Aeronautics' },
		timestamp: nowMinus(5, 'hour'),
	},
	{
		id: 'audit-modpack-install',
		actor: members.value[1].user,
		world: worldOptions[1],
		action: { type: 'content_installed', contentType: 'modpack', name: 'Cobblemon x Create' },
		timestamp: nowMinus(5, 'hour'),
	},
	{
		id: 'audit-file-edit',
		actor: members.value[1].user,
		world: worldOptions[1],
		action: { type: 'file_edited', file: 'server.properties' },
		timestamp: nowMinus(5, 'hour'),
	},
	{
		id: 'audit-role-change',
		actor: members.value[0].user,
		world: null,
		action: { type: 'role_changed', target: 'Geometrically' },
		timestamp: nowMinus(2, 'day'),
	},
])

const inviteSuggestions: ServerAccessInviteSuggestion[] = [
	{ id: 'user-fetch', username: 'Fetch', email: 'fetch@example.com' },
	{ id: 'user-emma', username: 'Emma', email: 'emma@example.com' },
	{ id: 'user-boris', username: 'Boris', email: 'boris@example.com' },
	{ id: 'user-coolbot', username: 'Coolbot', email: 'coolbot@example.com' },
]

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

function pushAuditEntry(entry: Omit<ServerAuditLogEntry, 'id' | 'timestamp'>) {
	auditEntries.value = [
		{
			...entry,
			id: `audit-${Date.now()}-${Math.random().toString(36).slice(2)}`,
			timestamp: new Date().toISOString(),
		},
		...auditEntries.value,
	]
}

function updateMemberRole(member: ServerAccessMember, role: ServerAccessRole) {
	if (member.isOwner || member.role === role) return

	members.value = members.value.map((existingMember) =>
		existingMember.id === member.id ? { ...existingMember, role } : existingMember,
	)
	pushAuditEntry({
		actor: members.value[0].user,
		world: null,
		action: { type: 'role_changed', target: member.user.username },
	})
	addNotification({
		type: 'success',
		title: formatMessage(messages.roleUpdatedTitle),
		text: formatMessage(messages.roleUpdatedText, {
			target: member.user.username,
			role: formatRole(role),
		}),
	})
}

function resendInvite(member: ServerAccessMember) {
	addNotification({
		type: 'success',
		title: formatMessage(messages.inviteResentTitle),
		text: formatMessage(messages.inviteResentText, { target: member.user.username }),
	})
}

function cancelInvite(member: ServerAccessMember) {
	members.value = members.value.filter((existingMember) => existingMember.id !== member.id)
	pushAuditEntry({
		actor: members.value[0].user,
		world: null,
		action: { type: 'member_removed', target: member.user.username },
	})
	addNotification({
		type: 'success',
		title: formatMessage(messages.inviteCancelledTitle),
		text: formatMessage(messages.inviteCancelledText, { target: member.user.username }),
	})
}

function removeMember(member: ServerAccessMember) {
	members.value = members.value.filter((existingMember) => existingMember.id !== member.id)
	pushAuditEntry({
		actor: members.value[0].user,
		world: null,
		action: { type: 'member_removed', target: member.user.username },
	})
	addNotification({
		type: 'success',
		title: formatMessage(messages.memberRemovedTitle),
		text: formatMessage(messages.memberRemovedText, { target: member.user.username }),
	})
}

function grantAccess(payload: GrantServerAccessPayload) {
	const suggestion = inviteSuggestions.find(
		(item) => item.username === payload.target || item.email === payload.target,
	)
	const username = suggestion?.username ?? payload.target
	const existingMember = members.value.find(
		(member) =>
			member.user.username.toLowerCase() === username.toLowerCase() ||
			member.user.id === suggestion?.id,
	)

	if (existingMember) {
		updateMemberRole(existingMember, payload.role)
		return
	}

	const member: ServerAccessMember = {
		id: `member-${Date.now()}`,
		user: {
			id: suggestion?.id ?? `pending-${payload.target.toLowerCase()}`,
			username,
			avatarUrl: suggestion?.avatarUrl,
		},
		role: payload.role,
		joinedAt: null,
		pending: true,
	}

	members.value = [member, ...members.value]
	pushAuditEntry({
		actor: members.value.find((existingMember) => existingMember.isOwner)?.user ?? member.user,
		world: null,
		action: { type: 'member_invited', target: username },
	})
	addNotification({
		type: 'success',
		title: formatMessage(messages.inviteSentTitle),
		text: formatMessage(messages.inviteSentText, {
			target: username,
			role: formatRole(payload.role),
		}),
	})
}
</script>
