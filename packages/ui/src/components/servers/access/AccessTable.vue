<template>
	<Table
		v-if="members.length > 0"
		class="hidden sm:block"
		:columns="columns"
		:data="tableMembers"
		row-key="id"
	>
		<template #cell-user="{ row: member }">
			<div class="flex min-w-0 items-center gap-2">
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
			</div>
		</template>

		<template #cell-role="{ row: member }">
			<span
				v-if="member.isOwner"
				class="inline-flex h-7 items-center rounded-full border border-solid px-2.5 py-1 text-sm font-semibold leading-none"
				:class="roleClasses(member.role)"
			>
				{{ formatRole(member.role) }}
			</span>
			<div v-else class="w-fit">
				<Combobox
					:model-value="member.role"
					:options="roleComboboxOptions"
					:display-value="formatRole(member.role)"
					:trigger-class="
						roleTriggerClass(member.role) +
						` !inline-flex !w-auto !h-7 !min-h-0 !rounded-full !border !border-solid !px-2.5 !py-1 gap-1 !text-sm !font-semibold !leading-5`
					"
					dropdown-class="!rounded-[24px] !bg-surface-3"
					dropdown-min-width="18rem"
					force-direction="down"
					@update:model-value="(role) => emit('updateRole', member, role)"
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
				class="inline-flex h-7 items-center rounded-full border border-surface-5 border-solid bg-surface-4 px-2.5 py-1 text-sm font-semibold text-secondary"
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
						v-tooltip="formatMessage(messages.resendInvite)"
						:aria-label="formatMessage(messages.resendInvite)"
						class="text-secondary hover:text-contrast"
						@click="emit('resendInvite', member)"
					>
						<SendIcon aria-hidden="true" />
					</button>
				</ButtonStyled>
				<ButtonStyled circular type="transparent">
					<button
						v-tooltip="
							member.pending
								? formatMessage(messages.cancelInvite)
								: formatMessage(messages.removeUser)
						"
						:aria-label="
							member.pending
								? formatMessage(messages.cancelInvite)
								: formatMessage(messages.removeUser)
						"
						class="text-secondary hover:text-red"
						@click="member.pending ? emit('cancelInvite', member) : emit('removeMember', member)"
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
		class="overflow-hidden rounded-2xl border border-solid border-surface-5 sm:hidden"
	>
		<div class="grid min-h-14 grid-cols-[3.75rem_7.25rem_minmax(0,1fr)_2.75rem] bg-surface-3">
			<div class="flex items-center pl-4 font-semibold text-secondary">
				{{ formatMessage(messages.userColumn) }}
			</div>
			<div class="flex items-center font-semibold text-secondary">
				{{ formatMessage(messages.roleColumn) }}
			</div>
			<div class="flex items-center justify-end font-semibold text-secondary">
				{{ formatMessage(messages.joinedColumn) }}
			</div>
			<div class="flex items-center justify-end pr-4 font-semibold text-secondary">
				<span class="sr-only">{{ formatMessage(messages.actionsColumn) }}</span>
			</div>
		</div>
		<div
			v-for="(member, index) in members"
			:key="member.id"
			class="grid min-h-16 grid-cols-[3.75rem_7.25rem_minmax(0,1fr)_2.75rem] items-center border-0 border-t border-solid border-surface-5"
			:class="index % 2 === 0 ? 'bg-surface-2' : 'bg-surface-1.5'"
		>
			<div class="flex min-w-0 items-center pl-4">
				<span v-tooltip="member.user.username" class="inline-flex shrink-0">
					<Avatar
						:src="member.user.avatarUrl"
						:alt="formatMessage(messages.userAvatarAlt, { username: member.user.username })"
						:tint-by="member.user.username"
						size="24px"
						circle
						no-shadow
					/>
				</span>
			</div>
			<div class="min-w-0 py-3 pr-2">
				<span
					v-if="member.isOwner"
					class="inline-flex h-7 max-w-full items-center truncate rounded-full border border-solid px-2.5 py-1 text-sm font-semibold leading-none"
					:class="roleClasses(member.role)"
				>
					{{ formatRole(member.role) }}
				</span>
				<div v-else class="min-w-0">
					<Combobox
						:model-value="member.role"
						:options="roleComboboxOptions"
						:display-value="formatRole(member.role)"
						:trigger-class="
							roleTriggerClass(member.role) +
							` !inline-flex !w-auto !max-w-full !h-7 !min-h-0 !rounded-full !border !border-solid !px-2.5 !py-1 gap-1 !text-sm !font-semibold !leading-5`
						"
						dropdown-class="!rounded-[24px] !bg-surface-3"
						dropdown-min-width="18rem"
						force-direction="down"
						@update:model-value="(role) => emit('updateRole', member, role)"
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
					class="inline-flex h-7 max-w-full items-center rounded-full border border-surface-5 border-solid bg-surface-4 px-2.5 py-1 text-sm font-semibold text-secondary"
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
			<div class="flex min-w-0 items-center justify-end pr-2">
				<ButtonStyled v-if="!member.isOwner" circular type="transparent">
					<TeleportOverflowMenu :options="memberActionOptions(member)">
						<MoreVerticalIcon aria-hidden="true" class="size-5" />
						<span class="sr-only">
							{{ formatMessage(messages.memberActionsLabel, { username: member.user.username }) }}
						</span>
						<template #resend-invite>
							<SendIcon aria-hidden="true" />
							{{ formatMessage(messages.resendInvite) }}
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

	<div v-else class="overflow-hidden rounded-2xl border border-solid border-surface-5">
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
			class="border-0 border-t border-solid border-surface-5 bg-surface-2 px-4 py-8 text-center text-secondary"
		>
			{{ formatMessage(messages.emptyState) }}
		</div>
	</div>
</template>

<script setup lang="ts">
import { MoreVerticalIcon, SendIcon, UserXIcon, XIcon } from '@modrinth/assets'
import { type Component, computed } from 'vue'

import { useFormatDateTime, useRelativeTime } from '../../../composables'
import { defineMessages, useVIntl } from '../../../composables/i18n'
import Avatar from '../../base/Avatar.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'
import Combobox, { type ComboboxOption } from '../../base/Combobox.vue'
import Table, { type TableColumn } from '../../base/Table.vue'
import TeleportOverflowMenu from '../../base/TeleportOverflowMenu.vue'
import type { ServerAccessMember, ServerAccessRole, ServerAccessRoleOption } from './types'

const props = defineProps<{
	members: ServerAccessMember[]
	roles: ServerAccessRoleOption[]
}>()

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
		defaultMessage: 'Unknown',
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
		defaultMessage: 'Remove user',
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
		defaultMessage: 'Viewer',
	},
})

type AccessTableColumn = 'user' | 'role' | 'joined' | 'actions'
type AccessTableRow = ServerAccessMember & Record<string, unknown>
type OverflowMenuOption = {
	id: string
	icon?: Component
	action: () => void
	shown?: boolean
	color?: 'standard' | 'brand' | 'red' | 'orange' | 'green' | 'blue' | 'purple'
}

const columns = computed<TableColumn<AccessTableColumn>[]>(() => [
	{ key: 'user', label: formatMessage(messages.userColumn), width: '32%' },
	{ key: 'role', label: formatMessage(messages.roleColumn), width: '28%' },
	{ key: 'joined', label: formatMessage(messages.joinedColumn), width: '28%' },
	{ key: 'actions', label: formatMessage(messages.actionsColumn), align: 'right', width: '12%' },
])

const tableMembers = computed<AccessTableRow[]>(() => props.members as AccessTableRow[])

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

function memberActionOptions(member: ServerAccessMember): OverflowMenuOption[] {
	return [
		{
			id: 'resend-invite',
			icon: SendIcon,
			action: () => emit('resendInvite', member),
			shown: member.pending,
		},
		{
			id: 'cancel-invite',
			icon: XIcon,
			action: () => emit('cancelInvite', member),
			color: 'red',
			shown: member.pending,
		},
		{
			id: 'remove-user',
			icon: UserXIcon,
			action: () => emit('removeMember', member),
			color: 'red',
			shown: !member.pending,
		},
	]
}
</script>
