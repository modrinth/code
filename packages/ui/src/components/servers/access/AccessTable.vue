<template>
	<Table
		v-if="members.length > 0"
		v-model:sort-column="sortColumn"
		v-model:sort-direction="sortDirection"
		class="hidden sm:block"
		:columns="columns"
		:data="tableMembers"
		row-key="id"
		table-min-width="42rem"
	>
		<template #cell-user="{ row: member }">
			<AutoLink
				:to="getUserProfileLink(member.user.username)"
				:target="userProfileTarget(member.user.username)"
				class="inline-flex max-w-full min-w-0 items-center gap-2"
				:class="getUserProfileLink(member.user.username) ? 'text-primary hover:underline' : ''"
			>
				<Avatar
					:src="member.user.avatarUrl"
					:alt="formatMessage(messages.userAvatarAlt, { username: member.user.username })"
					:tint-by="member.user.username"
					size="22px"
					circle
					no-shadow
				/>
				<span class="min-w-0 truncate font-medium">
					{{ member.user.username }}
				</span>
			</AutoLink>
		</template>

		<template #cell-role="{ row: member }">
			<span
				v-if="member.isOwner"
				class="inline-flex h-7 items-center rounded-full border border-solid px-2.5 py-1 text-sm font-semibold leading-none"
				:class="roleClasses(member.role)"
			>
				{{ formatRole(member.role) }}
			</span>
			<div v-else v-tooltip="accessManagementTooltip" class="w-fit">
				<Combobox
					:model-value="member.role"
					:options="roleComboboxOptions"
					:display-value="formatRole(member.role)"
					:disabled="!canManageUsers"
					:trigger-class="`${roleTriggerClass(member.role)} !inline-flex !w-auto !h-7 !min-h-0 !rounded-full !border !border-solid !px-2.5 !py-1 gap-1 !text-sm !font-semibold !leading-5`"
					dropdown-class="!rounded-[24px] !bg-surface-3"
					dropdown-min-width="18rem"
					force-direction="down"
					@update:model-value="(role) => handleUpdateRole(member, role)"
				>
					<template #selected>
						<span class="font-semibold leading-5" :class="roleTextClass(member.role)">
							{{ formatRole(member.role) }}
						</span>
					</template>
				</Combobox>
			</div>
		</template>

		<template #cell-joined="{ row: member }">
			<span
				v-if="member.pending"
				class="inline-flex h-7 items-center rounded-full border border-surface-4 border-solid bg-surface-4 px-2.5 py-1 text-sm font-semibold text-secondary"
			>
				{{ formatMessage(messages.pendingLabel) }}
			</span>
			<span v-else-if="member.joinedAt" v-tooltip="formatDate(member.joinedAt)">
				{{ formatRelativeTime(member.joinedAt) }}
			</span>
			<span v-else>{{ formatMessage(messages.unknownJoinedDate) }}</span>
		</template>

		<template #cell-actions="{ row: member }">
			<div v-if="!member.isOwner" class="flex items-center justify-end gap-1">
				<ButtonStyled v-if="member.pending" circular type="transparent">
					<button
						v-tooltip="resendInviteTooltip(member)"
						:aria-label="resendInviteLabel(member)"
						:disabled="resendInviteDisabled(member)"
						class="text-secondary hover:!filter-none hover:text-contrast focus-visible:!filter-none active:!scale-100 active:!filter-none disabled:cursor-not-allowed disabled:opacity-50 disabled:hover:text-secondary"
						@click="handleResendInvite(member)"
					>
						<SendIcon aria-hidden="true" />
					</button>
				</ButtonStyled>
				<ButtonStyled circular type="transparent">
					<button
						v-tooltip="memberAccessActionTooltip(member)"
						:aria-label="memberAccessActionLabel(member)"
						:disabled="!canManageUsers"
						class="text-secondary hover:!filter-none hover:text-red focus-visible:!filter-none active:!scale-100 active:!filter-none disabled:cursor-not-allowed disabled:opacity-50 disabled:hover:text-secondary"
						@click="member.pending ? handleCancelInvite(member) : handleRemoveMember(member)"
					>
						<XIcon v-if="member.pending" aria-hidden="true" />
						<UserXIcon v-else aria-hidden="true" />
					</button>
				</ButtonStyled>
			</div>
		</template>
	</Table>

	<div
		v-if="members.length > 0"
		class="overflow-hidden rounded-2xl border border-solid border-surface-4 sm:hidden"
	>
		<div
			class="grid min-h-14 grid-cols-[minmax(0,1.35fr)_7.75rem_minmax(6rem,0.8fr)_4rem] bg-surface-3"
		>
			<div class="flex items-center pl-4 font-semibold text-secondary">
				<button
					type="button"
					class="flex min-w-0 cursor-pointer items-center gap-1 border-none bg-transparent p-0 font-semibold transition-colors hover:text-contrast"
					:class="sortColumn === 'user' ? 'text-contrast' : 'text-secondary'"
					@click="toggleSort('user')"
				>
					<span class="min-w-0 truncate">{{ formatMessage(messages.userColumn) }}</span>
					<component :is="sortIcon('user')" v-if="sortIcon('user')" class="size-4" />
				</button>
			</div>
			<div class="flex items-center font-semibold text-secondary">
				<button
					type="button"
					class="flex cursor-pointer items-center gap-1 border-none bg-transparent p-0 font-semibold transition-colors hover:text-contrast"
					:class="sortColumn === 'role' ? 'text-contrast' : 'text-secondary'"
					@click="toggleSort('role')"
				>
					{{ formatMessage(messages.roleColumn) }}
					<component :is="sortIcon('role')" v-if="sortIcon('role')" class="size-4" />
				</button>
			</div>
			<div class="flex items-center justify-end font-semibold text-secondary">
				<button
					type="button"
					class="flex cursor-pointer items-center gap-1 border-none bg-transparent p-0 font-semibold transition-colors hover:text-contrast"
					:class="sortColumn === 'joined' ? 'text-contrast' : 'text-secondary'"
					@click="toggleSort('joined')"
				>
					{{ formatMessage(messages.joinedColumn) }}
					<component :is="sortIcon('joined')" v-if="sortIcon('joined')" class="size-4" />
				</button>
			</div>
			<div class="flex items-center justify-end pr-4 font-semibold text-secondary">
				<span class="sr-only">{{ formatMessage(messages.actionsColumn) }}</span>
			</div>
		</div>
		<div
			v-for="(member, index) in sortedMembers"
			:key="member.id"
			class="grid min-h-16 grid-cols-[minmax(0,1.35fr)_7.75rem_minmax(6rem,0.8fr)_4rem] items-center border-0 border-t border-solid border-surface-4"
			:class="index % 2 === 0 ? 'bg-surface-2' : 'bg-surface-1.5'"
		>
			<div class="flex min-w-0 items-center pl-4">
				<AutoLink
					v-tooltip="member.user.username"
					:to="getUserProfileLink(member.user.username)"
					:target="userProfileTarget(member.user.username)"
					class="inline-flex min-w-0 items-center gap-2"
					:class="getUserProfileLink(member.user.username) ? 'text-primary hover:underline' : ''"
				>
					<Avatar
						:src="member.user.avatarUrl"
						:alt="formatMessage(messages.userAvatarAlt, { username: member.user.username })"
						:tint-by="member.user.username"
						size="24px"
						circle
						no-shadow
					/>
					<span class="min-w-0 truncate font-medium">
						{{ member.user.username }}
					</span>
				</AutoLink>
			</div>
			<div class="min-w-0 py-3 pr-2">
				<span
					v-if="member.isOwner"
					class="inline-flex h-7 max-w-full items-center truncate rounded-full border border-solid px-2.5 py-1 text-sm font-semibold leading-none"
					:class="roleClasses(member.role)"
				>
					{{ formatRole(member.role) }}
				</span>
				<div v-else v-tooltip="accessManagementTooltip" class="min-w-0">
					<Combobox
						:model-value="member.role"
						:options="roleComboboxOptions"
						:display-value="formatRole(member.role)"
						:disabled="!canManageUsers"
						:trigger-class="
							roleTriggerClass(member.role) +
							` !inline-flex !w-auto !max-w-full !h-7 !min-h-0 !rounded-full !border !border-solid !px-2.5 !py-1 gap-1 !text-sm !font-semibold !leading-5`
						"
						dropdown-class="!rounded-[24px] !bg-surface-3"
						dropdown-min-width="18rem"
						force-direction="down"
						@update:model-value="(role) => handleUpdateRole(member, role)"
					>
						<template #selected>
							<span
								class="min-w-0 truncate font-semibold leading-5"
								:class="roleTextClass(member.role)"
							>
								{{ formatRole(member.role) }}
							</span>
						</template>
					</Combobox>
				</div>
			</div>
			<div class="min-w-0 py-3 pr-2 text-right text-secondary">
				<span
					v-if="member.pending"
					class="inline-flex h-7 max-w-full items-center rounded-full border border-surface-4 border-solid bg-surface-4 px-2.5 py-1 text-sm font-semibold text-secondary"
				>
					{{ formatMessage(messages.pendingLabel) }}
				</span>
				<span
					v-else-if="member.joinedAt"
					v-tooltip="formatDate(member.joinedAt)"
					class="inline-block max-w-full truncate"
				>
					{{ formatRelativeTime(member.joinedAt) }}
				</span>
				<span v-else>{{ formatMessage(messages.unknownJoinedDate) }}</span>
			</div>
			<div class="flex min-w-0 items-center justify-end pr-4">
				<ButtonStyled v-if="!member.isOwner" circular type="transparent">
					<TeleportOverflowMenu
						:options="memberActionOptions(member)"
						btn-class="hover:!filter-none focus-visible:!filter-none active:!scale-100 active:!filter-none"
					>
						<MoreVerticalIcon aria-hidden="true" class="size-5" />
						<span class="sr-only">
							{{ formatMessage(messages.memberActionsLabel, { username: member.user.username }) }}
						</span>
						<template #resend-invite>
							<SendIcon aria-hidden="true" />
							{{ resendInviteLabel(member) }}
						</template>
						<template #cancel-invite>
							<XIcon aria-hidden="true" />
							{{ formatMessage(messages.cancelInvite) }}
						</template>
						<template #remove-user>
							<UserXIcon aria-hidden="true" />
							{{ formatMessage(messages.removeUser) }}
						</template>
					</TeleportOverflowMenu>
				</ButtonStyled>
			</div>
		</div>
	</div>

	<div v-else class="overflow-hidden rounded-2xl border border-solid border-surface-4">
		<div
			class="grid min-h-14 grid-cols-[3.75rem_7.25rem_minmax(0,1fr)_2.75rem] bg-surface-3 sm:h-14 sm:grid-cols-[32%_28%_28%_12%]"
		>
			<div class="flex items-center pl-4 font-semibold text-secondary">
				{{ formatMessage(messages.userColumn) }}
			</div>
			<div class="flex items-center font-semibold text-secondary">
				{{ formatMessage(messages.roleColumn) }}
			</div>
			<div class="flex items-center font-semibold text-secondary">
				{{ formatMessage(messages.joinedColumn) }}
			</div>
			<div class="flex items-center justify-end pr-4 font-semibold text-secondary">
				{{ formatMessage(messages.actionsColumn) }}
			</div>
		</div>
		<div
			class="border-0 border-t border-solid border-surface-4 bg-surface-2 px-4 py-8 text-center text-secondary"
		>
			{{ formatMessage(messages.emptyState) }}
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	ChevronDownIcon,
	ChevronUpIcon,
	MoreVerticalIcon,
	SendIcon,
	UserXIcon,
	XIcon,
} from '@modrinth/assets'
import { type Component, computed, onMounted, onUnmounted, ref } from 'vue'

import { useFormatDateTime, useRelativeTime } from '../../../composables'
import { defineMessages, useVIntl } from '../../../composables/i18n'
import { commonMessages } from '../../../utils/common-messages'
import AutoLink from '../../base/AutoLink.vue'
import Avatar from '../../base/Avatar.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'
import Combobox, { type ComboboxOption } from '../../base/Combobox.vue'
import Table, { type SortDirection, type TableColumn } from '../../base/Table.vue'
import TeleportOverflowMenu from '../../base/TeleportOverflowMenu.vue'
import type {
	ServerAccessMember,
	ServerAccessRole,
	ServerAccessRoleOption,
	ServerAccessUserProfileLink,
} from './types'

const props = withDefaults(
	defineProps<{
		members: ServerAccessMember[]
		roles: ServerAccessRoleOption[]
		canManageUsers?: boolean
		permissionDeniedMessage?: string
		userProfileLink?: (username: string) => ServerAccessUserProfileLink
	}>(),
	{
		canManageUsers: true,
	},
)

const emit = defineEmits<{
	updateRole: [member: ServerAccessMember, role: ServerAccessRole]
	resendInvite: [member: ServerAccessMember]
	cancelInvite: [member: ServerAccessMember]
	removeMember: [member: ServerAccessMember]
}>()

const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()
const formatDate = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })

const messages = defineMessages({
	userColumn: {
		id: 'servers.access-table.column.user',
		defaultMessage: 'User',
	},
	roleColumn: {
		id: 'servers.access-table.column.role',
		defaultMessage: 'Role',
	},
	joinedColumn: {
		id: 'servers.access-table.column.joined',
		defaultMessage: 'Joined',
	},
	actionsColumn: {
		id: 'servers.access-table.column.actions',
		defaultMessage: 'Actions',
	},
	memberActionsLabel: {
		id: 'servers.access-table.member-actions-label',
		defaultMessage: 'Actions for {username}',
	},
	pendingLabel: {
		id: 'servers.access-table.pending',
		defaultMessage: 'Pending',
	},
	unknownJoinedDate: {
		id: 'servers.access-table.unknown-joined-date',
		defaultMessage: '—',
	},
	resendInvite: {
		id: 'servers.access-table.action.resend-invite',
		defaultMessage: 'Resend invite',
	},
	cancelInvite: {
		id: 'servers.access-table.action.cancel-invite',
		defaultMessage: 'Cancel invite',
	},
	removeUser: {
		id: 'servers.access-table.action.remove-user',
		defaultMessage: 'Revoke access',
	},
	emptyState: {
		id: 'servers.access-table.empty',
		defaultMessage: 'No users match your filters.',
	},
	userAvatarAlt: {
		id: 'servers.access-table.user-avatar-alt',
		defaultMessage: "{username}'s avatar",
	},
	ownerRole: {
		id: 'servers.access-role.owner',
		defaultMessage: 'Owner',
	},
	editorRole: {
		id: 'servers.access-role.editor',
		defaultMessage: 'Editor',
	},
	viewerRole: {
		id: 'servers.access-role.viewer',
		defaultMessage: 'Limited',
	},
	resendInviteCooldown: {
		id: 'servers.access-table.action.resend-invite-cooldown',
		defaultMessage: 'Resend in {seconds}s',
	},
})

type AccessTableColumn = 'user' | 'role' | 'joined' | 'actions'
type AccessTableSortableColumn = Exclude<AccessTableColumn, 'actions'>
type AccessTableRow = ServerAccessMember & Record<string, unknown>
type OverflowMenuOption = {
	id: string
	icon?: Component
	action: () => void
	shown?: boolean
	color?: 'standard' | 'brand' | 'red' | 'orange' | 'green' | 'blue' | 'purple'
	disabled?: boolean
	tooltip?: string
}

const columns = computed<TableColumn<AccessTableColumn>[]>(() => [
	{ key: 'user', label: formatMessage(messages.userColumn), width: '32%', enableSorting: true },
	{ key: 'role', label: formatMessage(messages.roleColumn), width: '28%', enableSorting: true },
	{ key: 'joined', label: formatMessage(messages.joinedColumn), enableSorting: true },
	{ key: 'actions', label: formatMessage(messages.actionsColumn), align: 'right', width: '7rem' },
])

const sortColumn = ref<string | undefined>('role')
const sortDirection = ref<SortDirection>('asc')
const now = ref(Date.now())
let nowInterval: ReturnType<typeof setInterval> | null = null
const canManageUsers = computed(() => props.canManageUsers)
const permissionDeniedMessage = computed(
	() => props.permissionDeniedMessage ?? formatMessage(commonMessages.noPermissionAction),
)
const accessManagementTooltip = computed(() =>
	canManageUsers.value ? undefined : permissionDeniedMessage.value,
)

const roleSortOrder: Record<ServerAccessRole, number> = {
	owner: 0,
	editor: 1,
	viewer: 2,
}

const sortedMembers = computed(() => {
	const direction = sortDirection.value === 'asc' ? 1 : -1
	const column = normalizeSortColumn(sortColumn.value)

	return [...props.members].sort((a, b) => {
		const compared = compareMembers(a, b, column)
		if (compared !== 0) return compared * direction

		return a.user.username.localeCompare(b.user.username)
	})
})

const tableMembers = computed<AccessTableRow[]>(() => sortedMembers.value as AccessTableRow[])

onMounted(() => {
	nowInterval = setInterval(() => {
		now.value = Date.now()
	}, 1000)
})

onUnmounted(() => {
	if (nowInterval) clearInterval(nowInterval)
})

function formatRole(role: ServerAccessRole): string {
	switch (role) {
		case 'owner':
			return formatMessage(messages.ownerRole)
		case 'editor':
			return formatMessage(messages.editorRole)
		case 'viewer':
			return formatMessage(messages.viewerRole)
	}
}

const roleComboboxOptions = computed<ComboboxOption<ServerAccessRole>[]>(() =>
	props.roles
		.filter((role) => role.value !== 'owner')
		.map((role) => ({
			value: role.value,
			label: role.label,
			subLabel: role.description,
		})),
)

function compareMembers(
	a: ServerAccessMember,
	b: ServerAccessMember,
	column: AccessTableSortableColumn,
): number {
	switch (column) {
		case 'user':
			return a.user.username.localeCompare(b.user.username)
		case 'role':
			return roleSortOrder[a.role] - roleSortOrder[b.role]
		case 'joined':
			return joinedTimestamp(a) - joinedTimestamp(b)
	}
}

function normalizeSortColumn(column: string | undefined): AccessTableSortableColumn {
	return column === 'user' || column === 'role' || column === 'joined' ? column : 'joined'
}

function joinedTimestamp(member: ServerAccessMember): number {
	if (member.pending) return Number.NEGATIVE_INFINITY
	return member.joinedAt ? new Date(member.joinedAt).getTime() : 0
}

function toggleSort(column: AccessTableSortableColumn) {
	if (sortColumn.value === column) {
		sortDirection.value = sortDirection.value === 'asc' ? 'desc' : 'asc'
		return
	}

	sortColumn.value = column
	sortDirection.value = 'asc'
}

function sortIcon(column: AccessTableSortableColumn): Component | null {
	if (sortColumn.value !== column) return null
	return sortDirection.value === 'asc' ? ChevronUpIcon : ChevronDownIcon
}

function roleClasses(role: ServerAccessRole): string {
	switch (role) {
		case 'owner':
			return 'border-orange !bg-highlight-orange !text-orange'
		case 'editor':
			return 'border-green !bg-highlight-green !text-brand'
		case 'viewer':
			return 'border-blue !bg-highlight-blue !text-blue'
	}
}

function roleTextClass(role: ServerAccessRole): string {
	switch (role) {
		case 'owner':
			return '!text-orange'
		case 'editor':
			return '!text-green'
		case 'viewer':
			return '!text-blue'
	}
}

function roleTriggerClass(role: ServerAccessRole): string {
	return roleClasses(role)
}

function getUserProfileLink(username: string): ServerAccessUserProfileLink {
	if (!username || username.includes('@')) return undefined
	return props.userProfileLink?.(username) ?? `/user/${encodeURIComponent(username)}`
}

function userProfileTarget(username: string): string | undefined {
	const link = getUserProfileLink(username)
	return typeof link === 'string' && link.startsWith('http') ? '_blank' : undefined
}

function resendInviteCooldownSeconds(member: ServerAccessMember): number {
	const availableAt = member.inviteResendAvailableAt
		? new Date(member.inviteResendAvailableAt).getTime()
		: 0
	return Math.max(0, Math.ceil((availableAt - now.value) / 1000))
}

function resendInviteCooldownDisabled(member: ServerAccessMember): boolean {
	return resendInviteCooldownSeconds(member) > 0
}

function resendInviteDisabled(member: ServerAccessMember): boolean {
	return !canManageUsers.value || resendInviteCooldownDisabled(member)
}

function resendInviteLabel(member: ServerAccessMember): string {
	const seconds = resendInviteCooldownSeconds(member)
	return seconds > 0
		? formatMessage(messages.resendInviteCooldown, { seconds })
		: formatMessage(messages.resendInvite)
}

function resendInviteTooltip(member: ServerAccessMember): string {
	return canManageUsers.value ? resendInviteLabel(member) : permissionDeniedMessage.value
}

function handleResendInvite(member: ServerAccessMember) {
	if (resendInviteDisabled(member)) return
	emit('resendInvite', member)
}

function memberAccessActionLabel(member: ServerAccessMember): string {
	return member.pending ? formatMessage(messages.cancelInvite) : formatMessage(messages.removeUser)
}

function memberAccessActionTooltip(member: ServerAccessMember): string {
	return canManageUsers.value ? memberAccessActionLabel(member) : permissionDeniedMessage.value
}

function handleUpdateRole(member: ServerAccessMember, role: ServerAccessRole) {
	if (!canManageUsers.value) return
	emit('updateRole', member, role)
}

function handleCancelInvite(member: ServerAccessMember) {
	if (!canManageUsers.value) return
	emit('cancelInvite', member)
}

function handleRemoveMember(member: ServerAccessMember) {
	if (!canManageUsers.value) return
	emit('removeMember', member)
}

function memberActionOptions(member: ServerAccessMember): OverflowMenuOption[] {
	return [
		{
			id: 'resend-invite',
			icon: SendIcon,
			action: () => handleResendInvite(member),
			shown: member.pending,
			disabled: resendInviteDisabled(member),
			tooltip: resendInviteTooltip(member),
		},
		{
			id: 'cancel-invite',
			icon: XIcon,
			action: () => handleCancelInvite(member),
			color: 'red',
			shown: member.pending,
			disabled: !canManageUsers.value,
			tooltip: memberAccessActionTooltip(member),
		},
		{
			id: 'remove-user',
			icon: UserXIcon,
			action: () => handleRemoveMember(member),
			color: 'red',
			shown: !member.pending,
			disabled: !canManageUsers.value,
			tooltip: memberAccessActionTooltip(member),
		},
	]
}
</script>
