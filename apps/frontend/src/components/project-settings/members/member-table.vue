<script setup lang="ts">
import { CrownIcon, EditIcon, UserXIcon } from '@modrinth/assets'
import { Avatar, Badge, defineMessages, IconButton, useVIntl } from '@modrinth/ui'

import MemberEditor from './member-editor.vue'
import { hasPermission, permissionMessages, projectPermissions } from './permissions'
import type { MemberChanges, MemberDraft } from './types'

defineProps<{
	members: MemberDraft[]
	openMembers: string[]
	userId?: string
	permissions: number | null
	isServerProject: boolean
	showRevenue: boolean
	organization?: boolean
	busy: boolean
	revenuePercentage: (member: MemberDraft) => number
}>()
const emit = defineEmits<{
	toggle: [id: string]
	update: [member: MemberDraft, changes: MemberChanges]
	save: [member: MemberDraft]
	remove: [member: MemberDraft]
}>()

const { formatMessage } = useVIntl()
const messages = defineMessages({
	member: { id: 'project.settings.members.column.member', defaultMessage: 'Member' },
	role: { id: 'project.settings.members.column.role', defaultMessage: 'Role title' },
	access: { id: 'project.settings.members.column.access', defaultMessage: 'Access' },
	revenue: { id: 'project.settings.members.column.revenue', defaultMessage: 'Revenue' },
	actions: { id: 'project.settings.members.column.actions', defaultMessage: 'Actions' },
	projectOwner: { id: 'project.settings.members.project-owner', defaultMessage: 'Project owner' },
	organizationOwner: {
		id: 'project.settings.members.organization-owner',
		defaultMessage: 'Organization owner',
	},
	you: { id: 'project.settings.members.you', defaultMessage: 'You' },
	fullAccess: { id: 'project.settings.members.full-access', defaultMessage: 'Full access' },
	customAccess: { id: 'project.settings.members.custom-access', defaultMessage: 'Custom' },
	organizationDefaults: {
		id: 'project.settings.members.organization-defaults',
		defaultMessage: 'Organization defaults',
	},
	fullAccessDescription: {
		id: 'project.settings.members.full-access.description',
		defaultMessage: 'Full access to this project',
	},
	noPermissions: {
		id: 'project.settings.members.no-permissions',
		defaultMessage: 'No permissions',
	},
	editDescription: {
		id: 'project.settings.members.permission.edit-description',
		defaultMessage: 'Edit description',
	},
	removeMembers: {
		id: 'project.settings.members.permission.remove-members',
		defaultMessage: 'Remove members',
	},
	editMembers: {
		id: 'project.settings.members.permission.edit-members',
		defaultMessage: 'Edit members',
	},
	edit: { id: 'project.settings.members.edit-user', defaultMessage: 'Edit {username}' },
	remove: { id: 'project.settings.members.remove-user', defaultMessage: 'Remove {username}' },
})
const permissionDescriptionMessages = {
	...permissionMessages,
	editBody: messages.editDescription,
	removeMember: messages.removeMembers,
	editMember: messages.editMembers,
}
const permissionKeys = Object.keys(projectPermissions) as (keyof typeof projectPermissions)[]

function permissionDescription(member: MemberDraft) {
	if (member.is_owner) return formatMessage(messages.fullAccessDescription)
	const labels = permissionKeys
		.filter((key) => hasPermission(member.permissions, projectPermissions[key]))
		.map((key) => formatMessage(permissionDescriptionMessages[key]))
	return labels.join(', ') || formatMessage(messages.noPermissions)
}
</script>

<template>
	<div class="overflow-x-auto rounded-2xl border border-solid border-surface-5 bg-surface-3">
		<table class="w-full border-collapse text-left">
			<thead>
				<tr class="text-xs uppercase text-secondary">
					<th class="px-4 py-3">{{ formatMessage(messages.member) }}</th>
					<th class="px-4 py-3">{{ formatMessage(messages.role) }}</th>
					<th class="px-4 py-3">{{ formatMessage(messages.access) }}</th>
					<th v-if="showRevenue" class="px-4 py-3">{{ formatMessage(messages.revenue) }}</th>
					<th class="px-4 py-3">
						<span class="sr-only">{{ formatMessage(messages.actions) }}</span>
					</th>
				</tr>
			</thead>
			<tbody v-for="member in members" :key="member.user.id" class="member">
				<tr class="border-0 border-t border-solid border-surface-5">
					<td class="px-4 py-3">
						<div class="flex items-center gap-2">
							<Avatar :src="member.user.avatar_url" :alt="member.user.username" size="sm" circle />
							<NuxtLink :to="`/user/${member.user.username}`" class="font-semibold text-contrast">
								{{ member.user.username }}
							</NuxtLink>
							<CrownIcon
								v-if="member.is_owner"
								v-tooltip="
									formatMessage(organization ? messages.organizationOwner : messages.projectOwner)
								"
								class="size-4 text-orange"
							/>
							<span
								v-if="!organization && member.user.id === userId"
								class="text-xs text-secondary"
							>
								{{ formatMessage(messages.you) }}
							</span>
							<Badge v-if="!member.accepted" type="pending" />
						</div>
					</td>
					<td class="px-4 py-3">{{ member.role }}</td>
					<td class="px-4 py-3">
						<span
							v-tooltip="permissionDescription(member)"
							tabindex="0"
							class="cursor-help border-0 border-b border-dotted border-surface-5"
						>
							{{
								formatMessage(
									member.is_owner
										? messages.fullAccess
										: organization && !member.override
											? messages.organizationDefaults
											: messages.customAccess,
								)
							}}
						</span>
					</td>
					<td v-if="showRevenue" class="whitespace-nowrap px-4 py-3">
						{{ member.payouts_split }} ({{ revenuePercentage(member) }}%)
					</td>
					<td class="px-4 py-3">
						<div :class="organization ? 'contents' : 'flex justify-end gap-2'">
							<IconButton
								:label="formatMessage(messages.edit, { username: member.user.username })"
								@click="emit('toggle', member.user.id)"
							>
								<EditIcon />
							</IconButton>
							<IconButton
								v-if="!organization && !member.is_owner"
								:label="formatMessage(messages.remove, { username: member.user.username })"
								:disabled="!hasPermission(permissions, projectPermissions.removeMember) || busy"
								@click="emit('remove', member)"
							>
								<UserXIcon />
							</IconButton>
						</div>
					</td>
				</tr>
				<tr v-if="openMembers.includes(member.user.id)">
					<td
						:colspan="showRevenue ? 5 : 4"
						class="border-0 border-t border-solid border-surface-5 p-4"
					>
						<MemberEditor
							:member="member"
							:permissions="permissions"
							:is-server-project="isServerProject"
							:organization="organization ?? false"
							:busy="busy"
							@update="emit('update', member, $event)"
							@save="emit('save', member)"
							@remove="emit('remove', member)"
						/>
					</td>
				</tr>
			</tbody>
		</table>
	</div>
</template>
