<script setup lang="ts">
import { SaveIcon, UserXIcon } from '@modrinth/assets'
import {
	Button,
	Checkbox,
	commonMessages,
	defineMessages,
	Input,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { computed } from 'vue'

import ProjectIssueCard from '~/components/ui/project-issue-card/index.vue'

import { hasPermission, permissionMessages, projectPermissions } from './permissions'
import type { MemberChanges, MemberDraft } from './types'

const props = defineProps<{
	member: MemberDraft
	permissions: number | null
	isServerProject: boolean
	organization: boolean
	busy: boolean
}>()
const emit = defineEmits<{
	update: [changes: MemberChanges]
	save: []
	remove: []
}>()

const { formatMessage } = useVIntl()
const messages = defineMessages({
	override: { id: 'project.settings.members.override.title', defaultMessage: 'Override values' },
	overrideDescription: {
		id: 'project.settings.members.override.description',
		defaultMessage:
			'Override organization default values and assign custom permissions, roles, and monetization weights to this user on the project.',
	},
	role: { id: 'project.settings.members.role.title', defaultMessage: 'Role' },
	roleDescription: {
		id: 'project.settings.members.role.description',
		defaultMessage: 'The title of the role that this member plays for this project.',
	},
	weight: { id: 'project.settings.members.weight.title', defaultMessage: 'Monetization weight' },
	weightDescription: {
		id: 'project.settings.members.weight.description',
		defaultMessage:
			"Relative to all other members' monetization weights, this determines what portion of this project's revenue goes to this member.",
	},
	permissions: { id: 'project.settings.members.permissions', defaultMessage: 'Permissions' },
	updateContent: {
		id: 'project.settings.members.permission.update-content',
		defaultMessage: 'Update content',
	},
})

const id = computed(() => `member-${props.member.user.username}`)
const canEdit = computed(() => hasPermission(props.permissions, projectPermissions.editMember))
const canEditFields = computed(
	() => canEdit.value && (!props.organization || props.member.override),
)
const canSave = computed(
	() => canEdit.value && (!props.organization || props.member.oldOverride || props.member.override),
)
const editablePermissions = computed(() => {
	const keys = Object.keys(projectPermissions) as (keyof typeof projectPermissions)[]
	return keys
		.filter((key) => !props.isServerProject || key !== 'deleteVersion')
		.map((key) => ({
			bit: projectPermissions[key],
			label: formatMessage(
				key === 'uploadVersion' && props.isServerProject
					? messages.updateContent
					: permissionMessages[key],
			),
		}))
})
</script>

<template>
	<div class="flex flex-col">
		<div v-if="organization" class="adjacent-input">
			<label :for="`${id}-override-perms`" class="w-fit">
				<span class="label__title">{{ formatMessage(messages.override) }}</span>
				<span class="label__description">{{ formatMessage(messages.overrideDescription) }}</span>
			</label>
			<Toggle
				:id="`${id}-override-perms`"
				:model-value="member.override ?? false"
				:disabled="!canEdit"
				@update:model-value="emit('update', { override: $event })"
			/>
		</div>
		<div class="adjacent-input">
			<label :for="`${id}-role`" class="w-fit">
				<span class="label__title">{{ formatMessage(messages.role) }}</span>
				<span class="label__description">{{ formatMessage(messages.roleDescription) }}</span>
			</label>
			<Input
				:id="`${id}-role`"
				:model-value="member.role"
				:disabled="!canEditFields"
				@update:model-value="emit('update', { role: String($event ?? '') })"
			/>
		</div>
		<ProjectIssueCard target="modify_team_member_role" :user-id="member.user.id" />
		<div v-if="!isServerProject" class="adjacent-input">
			<label :for="`${id}-monetization-weight`" class="w-fit">
				<span class="label__title">{{ formatMessage(messages.weight) }}</span>
				<span class="label__description">{{ formatMessage(messages.weightDescription) }}</span>
			</label>
			<Input
				:id="`${id}-monetization-weight`"
				:model-value="member.payouts_split ?? undefined"
				type="number"
				:disabled="!canEditFields"
				@update:model-value="
					emit('update', { payouts_split: typeof $event === 'number' ? $event : undefined })
				"
			/>
		</div>
		<template v-if="!member.is_owner">
			<span class="label">
				<span class="label__title">{{ formatMessage(messages.permissions) }}</span>
			</span>
			<div class="permissions grid grid-cols-1 gap-2 sm:grid-cols-2">
				<Checkbox
					v-for="{ bit, label } in editablePermissions"
					:key="bit"
					:model-value="hasPermission(member.permissions, bit)"
					:disabled="!canEditFields || !hasPermission(permissions, bit)"
					:label="label"
					@update:model-value="emit('update', { permissions: (member.permissions ?? 0) ^ bit })"
				/>
			</div>
		</template>
		<div class="input-group">
			<Button type="colored" color="brand" :disabled="!canSave || busy" @click="emit('save')">
				<SaveIcon />
				{{ formatMessage(commonMessages.saveChangesButton) }}
			</Button>
			<Button
				v-if="!organization && !member.is_owner"
				type="colored"
				color="red"
				:disabled="!hasPermission(permissions, projectPermissions.removeMember) || busy"
				@click="emit('remove')"
			>
				<UserXIcon />
				{{ formatMessage(permissionMessages.removeMember) }}
			</Button>
		</div>
	</div>
</template>
