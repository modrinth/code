<script setup lang="ts">
import { CrownIcon, EditIcon, OrganizationIcon, UserXIcon } from '@modrinth/assets'
import {
	Avatar,
	Badge,
	defineMessages,
	IconButton,
	Table,
	type TableColumn,
	TagItem,
	useVIntl,
} from '@modrinth/ui'
import { computed } from 'vue'

import {
	hasPermission,
	permissionsHiddenMessage,
	projectAccessPresets,
	projectPermissions,
} from './permissions'
import { revenuePercentMessage } from './revenue'
import type { MemberDraft } from './types'

const props = defineProps<{
	members: MemberDraft[]
	userId?: string
	permissions?: number | null
	isServerProject?: boolean
	showRevenue?: boolean
	organizationRoster?: {
		accessLabel: (member: MemberDraft) => string
		canRemove: (member: MemberDraft) => boolean
	}
	busy: boolean
	revenueShare?: (member: MemberDraft) => number
}>()
const emit = defineEmits<{
	edit: [member: MemberDraft, event: MouseEvent]
	remove: [member: MemberDraft, event: MouseEvent]
}>()

const { formatMessage } = useVIntl()
const messages = defineMessages({
	empty: {
		id: 'project.settings.members.empty',
		defaultMessage: 'No members yet.',
	},
	organizationMember: {
		id: 'project.settings.members.organization-member',
		defaultMessage: 'Organization member',
	},
	member: {
		id: 'project.settings.members.column.member',
		defaultMessage: 'Member',
	},
	role: {
		id: 'project.settings.members.column.role',
		defaultMessage: 'Role title',
	},
	access: {
		id: 'project.settings.members.column.access',
		defaultMessage: 'Access',
	},
	revenue: {
		id: 'project.settings.members.column.revenue',
		defaultMessage: 'Revenue',
	},
	actions: {
		id: 'project.settings.members.column.actions',
		defaultMessage: 'Actions',
	},
	projectOwner: {
		id: 'project.settings.members.project-owner',
		defaultMessage: 'Project owner',
	},
	organizationOwner: {
		id: 'project.settings.members.organization-owner',
		defaultMessage: 'Organization owner',
	},
	you: { id: 'project.settings.members.you', defaultMessage: 'You' },
	fullAccess: {
		id: 'project.settings.members.full-access',
		defaultMessage: 'Full access',
	},
	customAccess: {
		id: 'project.settings.members.custom-access',
		defaultMessage: 'Custom',
	},
	organizationDefaults: {
		id: 'project.settings.members.organization-defaults',
		defaultMessage: 'Organization defaults',
	},
	edit: {
		id: 'project.settings.members.edit-user',
		defaultMessage: 'Edit {username}',
	},
	cancelInvite: {
		id: 'organization.settings.members.cancel-invite-user',
		defaultMessage: 'Cancel invite for {username}',
	},
	remove: {
		id: 'project.settings.members.remove-user',
		defaultMessage: 'Remove {username}',
	},
})
const presets = computed(() => projectAccessPresets(props.isServerProject ?? false))
const rows = computed(() => props.members.map((member) => ({ ...member, id: member.user.id })))

type Column = 'member' | 'role' | 'access' | 'revenue' | 'actions'
const columns = computed<TableColumn<Column>[]>(() => [
	{ key: 'member', label: formatMessage(messages.member) },
	{
		key: 'role',
		label: formatMessage(messages.role),
		cellClass: 'break-words',
	},
	{ key: 'access', label: formatMessage(messages.access) },
	...(props.showRevenue
		? [
				{
					key: 'revenue' as const,
					label: formatMessage(messages.revenue),
					cellClass: 'whitespace-nowrap',
				},
			]
		: []),
	{
		key: 'actions',
		label: formatMessage(messages.actions),
		align: 'right',
		width: props.organizationRoster ? '145px' : '110px',
	},
])

function accessLabel(member: MemberDraft) {
	if (member.is_owner) return formatMessage(messages.fullAccess)
	if (props.organizationRoster) return props.organizationRoster.accessLabel(member)
	if (member.permissions == null) return formatMessage(permissionsHiddenMessage)
	if (member.override === false && member.organizationDefaults) {
		return formatMessage(messages.organizationDefaults)
	}
	const mask = member.permissions & ~(props.isServerProject ? projectPermissions.deleteVersion : 0)
	return formatMessage(
		presets.value.find((preset) => preset.mask === mask)?.label ?? messages.customAccess,
	)
}
</script>

<template>
	<Table :columns="columns" :data="rows" table-layout="auto" table-min-width="34rem">
		<template #header-actions>
			<span class="sr-only">{{ formatMessage(messages.actions) }}</span>
		</template>
		<template #cell-member="{ row: member }">
			<div class="flex items-center gap-2 py-2.5">
				<Avatar
					:src="member.user.avatar_url"
					:alt="member.user.username"
					:tint-by="member.user.username"
					size="2.5rem"
					circle
				/>
				<NuxtLink :to="`/user/${member.user.username}`" class="font-semibold text-contrast">
					{{ member.user.username }}
				</NuxtLink>
				<OrganizationIcon
					v-if="member.organizationDefaults"
					v-tooltip="formatMessage(messages.organizationMember)"
					role="img"
					:aria-label="formatMessage(messages.organizationMember)"
					class="size-4 shrink-0 text-secondary"
				/>
				<CrownIcon
					v-if="member.is_owner"
					v-tooltip="
						formatMessage(
							member.organizationDefaults || organizationRoster
								? messages.organizationOwner
								: messages.projectOwner,
						)
					"
					class="size-4 shrink-0 text-orange"
				/>
				<TagItem v-if="member.user.id === userId">
					{{ formatMessage(messages.you) }}
				</TagItem>
				<Badge v-if="!member.accepted" type="pending" />
			</div>
		</template>
		<template #cell-access="{ row: member }">
			{{ accessLabel(member) }}
		</template>
		<template #cell-revenue="{ row: member }">
			<span v-if="revenueShare && member.payouts_split != null" class="text-primary">
				{{ formatMessage(revenuePercentMessage, { share: revenueShare(member) }) }}
			</span>
			<span v-else>—</span>
		</template>
		<template #cell-actions="{ row: member }">
			<div class="flex justify-end gap-1">
				<IconButton
					v-tooltip="formatMessage(messages.edit, { username: member.user.username })"
					:label="formatMessage(messages.edit, { username: member.user.username })"
					type="quiet"
					:disabled="busy"
					@click="emit('edit', member, $event)"
				>
					<EditIcon />
				</IconButton>
				<slot name="member-actions" :member="member" />
				<IconButton
					v-if="!member.is_owner && !member.organizationDefaults"
					v-tooltip="
						formatMessage(
							organizationRoster && !member.accepted ? messages.cancelInvite : messages.remove,
							{ username: member.user.username },
						)
					"
					:label="
						formatMessage(
							organizationRoster && !member.accepted ? messages.cancelInvite : messages.remove,
							{ username: member.user.username },
						)
					"
					type="quiet"
					:disabled="
						busy ||
						!(
							organizationRoster?.canRemove(member) ??
							hasPermission(permissions, projectPermissions.removeMember)
						)
					"
					class="enabled:hover:!bg-red enabled:focus-visible:!bg-red [&:enabled:is(:hover,:focus-visible)>svg]:text-[var(--color-accent-contrast)]"
					@click="emit('remove', member, $event)"
				>
					<UserXIcon />
				</IconButton>
			</div>
		</template>
		<template #empty-state>
			<div class="p-6 text-center text-secondary">
				{{ formatMessage(messages.empty) }}
			</div>
		</template>
	</Table>
</template>
