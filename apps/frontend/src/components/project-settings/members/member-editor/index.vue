<script setup lang="ts">
import {
	Avatar,
	Button,
	commonMessages,
	defineMessages,
	Field,
	Input,
	IntlFormatted,
	NewModal,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref, shallowRef, useId, useTemplateRef } from 'vue'

import ProjectIssueCard from '~/components/ui/project-issue-card/index.vue'

import { hasPermission, permissionsHiddenMessage, projectPermissions } from '../permissions'
import { maxPayoutWeight } from '../revenue'
import type { MemberDraft } from '../types'
import ProjectAccessPicker, {
	type ProjectAccessGroup,
	type ProjectAccessPreset,
} from './project-access-picker.vue'

const props = defineProps<{
	permissions?: number | null
	organization?: { canEdit: boolean }
	accessOptions?: { presets: ProjectAccessPreset[]; groups: ProjectAccessGroup[] }
	isServerProject?: boolean
	showRevenueSplit?: boolean
	canChangeOverride?: (member: MemberDraft) => boolean
	saving: boolean
}>()
const emit = defineEmits<{ save: [member: MemberDraft, original: MemberDraft] }>()

const { formatMessage } = useVIntl()
const id = useId()
const modal = useTemplateRef('modal')
const roleInput = useTemplateRef('roleInput')
const draft = ref<MemberDraft>()
const original = shallowRef<MemberDraft>()
const editSession = ref(0)
const roleLimit = 32
const messages = defineMessages({
	edit: {
		id: 'project.settings.members.editor.title',
		defaultMessage: 'Edit <avatar></avatar> {username}',
	},
	override: { id: 'project.settings.members.override.title', defaultMessage: 'Override values' },
	overrideDescription: {
		id: 'project.settings.members.editor.override-description',
		defaultMessage: 'Set project-specific values for this organization member.',
	},
	role: { id: 'project.settings.members.column.role', defaultMessage: 'Role title' },
	roleDescription: {
		id: 'project.settings.members.editor.role-description',
		defaultMessage: 'A title shown next to their name on the project page.',
	},
	organizationRoleDescription: {
		id: 'organization.settings.members.editor.role-description',
		defaultMessage: 'A title shown next to their name on the organization page.',
	},
	roleCount: {
		id: 'project.settings.members.editor.role-count',
		defaultMessage: '{count, number}/{limit, number}',
	},
	access: { id: 'project.settings.members.column.access', defaultMessage: 'Access' },
	payoutWeight: {
		id: 'project.settings.members.weight.title',
		defaultMessage: 'Monetization weight',
	},
	payoutWeightDescription: {
		id: 'project.settings.members.weight.description',
		defaultMessage:
			"Relative to all other members' monetization weights, this determines what portion of this project's revenue goes to this member.",
	},
	invalidPayoutWeight: {
		id: 'project.settings.members.editor.invalid-payout-weight',
		defaultMessage: 'Enter a weight between 0 and {max, number}.',
	},
	ownerAccess: {
		id: 'project.settings.members.editor.owner-access',
		defaultMessage: 'Owners always have every permission.',
	},
	restrictedAccess: {
		id: 'project.settings.members.editor.restricted-access',
		defaultMessage:
			'This member has permissions you can’t grant. You can edit their other details.',
	},
})

const canEdit = computed(
	() =>
		props.organization?.canEdit ?? hasPermission(props.permissions, projectPermissions.editMember),
)
const canChangeOverride = computed(
	() => canEdit.value && !!original.value && props.canChangeOverride?.(original.value) === true,
)
const canEditFields = computed(
	() => canEdit.value && !props.saving && draft.value?.override !== false,
)
const showPayoutWeight = computed(
	() =>
		!props.organization &&
		!props.isServerProject &&
		original.value?.payouts_split != null &&
		(!original.value.accepted || !props.showRevenueSplit),
)
const invalidPayoutWeight = computed(() => {
	if (!showPayoutWeight.value || draft.value?.override === false) return false
	const weight = draft.value?.payouts_split
	return (
		weight !== original.value?.payouts_split &&
		(weight == null || !Number.isFinite(weight) || weight < 0 || weight > maxPayoutWeight)
	)
})
const grantable = computed(() => props.permissions ?? 0)
const hasRestrictedAccess = computed(
	() => !!((original.value?.permissions ?? 0) & ~grantable.value),
)
const accessMask = computed({
	get: () => draft.value?.permissions ?? 0,
	set: (permissions: number) => {
		if (draft.value) draft.value.permissions = permissions
	},
})
const canSave = computed(() => {
	const member = draft.value
	if (!member || !canEdit.value || props.saving || invalidPayoutWeight.value) return false
	return (
		(member.override === member.oldOverride || canChangeOverride.value) &&
		(member.override !== false || member.oldOverride === true)
	)
})

function show(member: MemberDraft, event: MouseEvent) {
	editSession.value++
	original.value = member
	draft.value = { ...member }
	modal.value?.show(event)
}

function reset() {
	draft.value = undefined
	original.value = undefined
}

function submit() {
	if (canSave.value && draft.value && original.value) {
		emit('save', { ...draft.value }, original.value)
	}
}

defineExpose({ show, hide: () => modal.value?.hide() })
</script>

<template>
	<NewModal
		ref="modal"
		width="38rem"
		scrollable
		actions-divider
		:disable-close="saving"
		:on-after-show="() => roleInput?.focus()"
		:on-after-hide="reset"
	>
		<template #title>
			<span
				v-if="draft"
				class="flex min-w-0 flex-wrap items-center gap-2 text-2xl font-semibold text-contrast"
			>
				<IntlFormatted :message-id="messages.edit" :values="{ username: draft.user.username }">
					<template #avatar>
						<Avatar :src="draft?.user.avatar_url" alt="" size="32px" circle />
					</template>
				</IntlFormatted>
			</span>
		</template>
		<form
			v-if="draft"
			:id="id"
			:key="editSession"
			class="flex flex-col gap-6"
			novalidate
			@submit.prevent="submit"
		>
			<div v-if="draft.oldOverride !== undefined" class="flex items-start justify-between gap-4">
				<div class="flex flex-col gap-1">
					<label :for="`${id}-override`" class="font-semibold text-contrast">
						{{ formatMessage(messages.override) }}
					</label>
					<span class="text-sm text-secondary">{{
						formatMessage(messages.overrideDescription)
					}}</span>
				</div>
				<Toggle
					:id="`${id}-override`"
					:model-value="draft.override ?? false"
					:disabled="!canChangeOverride || saving"
					class="shrink-0"
					@update:model-value="draft.override = $event"
				/>
			</div>

			<Field :label="formatMessage(messages.role)" :input-id="`${id}-role`">
				<template #right>
					<span class="text-sm tabular-nums text-secondary">
						{{ formatMessage(messages.roleCount, { count: draft.role.length, limit: roleLimit }) }}
					</span>
				</template>
				<Input
					:id="`${id}-role`"
					ref="roleInput"
					:model-value="draft.role"
					:maxlength="roleLimit"
					wrapper-class="w-full"
					:disabled="!canEditFields"
					:aria-describedby="`${id}-role-description`"
					@update:model-value="draft.role = String($event ?? '')"
				/>
				<template #subtext>
					<span :id="`${id}-role-description`">
						{{
							formatMessage(
								organization ? messages.organizationRoleDescription : messages.roleDescription,
							)
						}}
					</span>
				</template>
				<template #bottom>
					<ProjectIssueCard
						v-if="!organization"
						target="modify_team_member_role"
						:user-id="draft.user.id"
					/>
				</template>
			</Field>

			<Field
				v-if="showPayoutWeight"
				:label="formatMessage(messages.payoutWeight)"
				:input-id="`${id}-payout-weight`"
			>
				<Input
					:id="`${id}-payout-weight`"
					:model-value="draft.payouts_split ?? undefined"
					type="number"
					:min="0"
					:max="maxPayoutWeight"
					wrapper-class="w-full"
					:disabled="!canEditFields"
					:error="invalidPayoutWeight"
					:aria-describedby="`${id}-payout-weight-description`"
					@update:model-value="
						draft.payouts_split = typeof $event === 'number' ? $event : undefined
					"
				/>
				<template #subtext>
					<span :id="`${id}-payout-weight-description`">
						{{
							invalidPayoutWeight
								? formatMessage(messages.invalidPayoutWeight, { max: maxPayoutWeight })
								: formatMessage(messages.payoutWeightDescription)
						}}
					</span>
				</template>
			</Field>

			<div
				v-if="draft.is_owner || (!organization && draft.permissions == null)"
				class="flex flex-col gap-2"
			>
				<h3 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.access) }}
				</h3>
				<p class="m-0 text-secondary">
					{{ formatMessage(draft.is_owner ? messages.ownerAccess : permissionsHiddenMessage) }}
				</p>
			</div>
			<slot v-else-if="organization" name="access" :draft="draft" :can-edit="canEditFields" />
			<div v-else-if="accessOptions" class="flex flex-col gap-2">
				<ProjectAccessPicker
					v-model="accessMask"
					:presets="accessOptions.presets"
					:groups="accessOptions.groups"
					:grantable="grantable"
					:disabled="!canEditFields || hasRestrictedAccess"
				/>
				<p v-if="canEditFields && hasRestrictedAccess" class="m-0 text-sm text-secondary">
					{{ formatMessage(messages.restrictedAccess) }}
				</p>
			</div>
		</form>

		<template #actions>
			<div class="flex flex-wrap justify-end gap-2">
				<Button :disabled="saving" @click="modal?.hide()">
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button type="colored" color="brand" native-type="submit" :form="id" :disabled="!canSave">
					{{ formatMessage(commonMessages.saveChangesButton) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>
