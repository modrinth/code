<script setup lang="ts">
import {
	Button,
	commonMessages,
	ConfirmModal,
	defineMessages,
	Field,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { isAdmin } from '@modrinth/utils'
import { useMutation, useQuery } from '@tanstack/vue-query'
import { computed, nextTick, ref, useId, useTemplateRef } from 'vue'

import ConfirmTransferProjectModal from '~/components/ui/ConfirmTransferProjectModal.vue'
import ProjectOwnerCombobox from '~/components/ui/ProjectOwnerCombobox.vue'

const {
	projectV2: project,
	allMembers,
	currentMember,
	organization,
	invalidate,
} = injectProjectPageContext()
const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	transferDescription: {
		id: 'project.settings.ownership.transfer-description',
		defaultMessage:
			'Give full control of this project to another member or an organization. You’ll stay on as a member, but only the new owner can transfer it back.',
	},
	transferTitle: {
		id: 'project.settings.ownership.transfer-title',
		defaultMessage: 'Transfer {project}',
	},
	transferConfirmation: {
		id: 'project.settings.ownership.transfer-confirmation',
		defaultMessage:
			'The new owner will get full control of this project, including members, revenue, transferring, and deleting it. Only the new owner can transfer it back.',
	},
	managedBy: {
		id: 'project.settings.ownership.managed-by',
		defaultMessage: 'This project is managed by {organization}.',
	},
	transferTarget: {
		id: 'project.settings.ownership.transfer-target',
		defaultMessage: 'Member or organization',
	},
	missingTargets: {
		id: 'project.settings.ownership.missing-targets',
		defaultMessage:
			'Invite a member from the Members page or create an organization before transferring ownership.',
	},
	organizationsLoadFailed: {
		id: 'project.settings.ownership.organizations-load-failed',
		defaultMessage: 'Organizations could not be loaded. Try again later.',
	},
	removeTitle: {
		id: 'project.settings.ownership.remove-title',
		defaultMessage: 'Remove project from organization?',
	},
	removeDescription: {
		id: 'project.settings.ownership.remove-description',
		defaultMessage:
			'You will become the owner of this project and it will no longer be managed by the organization.',
	},
	organization: {
		id: 'project.settings.ownership.organization',
		defaultMessage: 'Organization',
	},
	owner: {
		id: 'project.settings.ownership.owner',
		defaultMessage: 'Owner',
	},
	transferred: {
		id: 'project.settings.ownership.transferred',
		defaultMessage: 'Ownership transferred',
	},
	transferredDescription: {
		id: 'project.settings.ownership.transferred-description',
		defaultMessage: 'Your project ownership has been updated.',
	},
	transferFailed: {
		id: 'project.settings.ownership.transfer-failed',
		defaultMessage: 'Transfer failed',
	},
	removeFailed: {
		id: 'project.settings.ownership.remove-failed',
		defaultMessage: 'Unable to remove project',
	},
	transferOwnership: {
		id: 'project.settings.ownership.transfer',
		defaultMessage: 'Transfer ownership',
	},
	transferRequiresOwner: {
		id: 'project.settings.ownership.transfer.requires-owner',
		defaultMessage: 'Only the project owner can transfer ownership.',
	},
	transferInProgress: {
		id: 'project.settings.ownership.transfer.in-progress',
		defaultMessage: 'An ownership transfer is in progress.',
	},
	removeFromOrganization: {
		id: 'project.settings.ownership.remove-from-organization',
		defaultMessage: 'Remove from organization',
	},
	removeFromOrganizationDescription: {
		id: 'project.settings.ownership.remove-from-organization.description',
		defaultMessage: 'Take ownership of this project and remove it from {organization}.',
	},
})
const transferTargetId = useId()
const auth = await useAuth()
const selectedTarget = ref('')
const selectionModal = useTemplateRef('selectionModal')
function show(event: MouseEvent) {
	selectedTarget.value = ''
	selectionModal.value?.show(event)
}
async function confirmTarget(event: MouseEvent) {
	if (!target.value) return
	selectionModal.value?.hide()
	await nextTick()
	transferModal.value?.show(event)
}
const transferModal = useTemplateRef('transferModal')
const removeModal = useTemplateRef('removeModal')
const REMOVE_PROJECT = 1 << 5
const canRemoveFromOrganization = computed(() => {
	const user = auth.value.user
	const member = organization.value?.members.find((member) => member.user.id === user?.id)
	return (
		!!member?.accepted &&
		(isAdmin(user) || ((member.organization_permissions ?? 0) & REMOVE_PROJECT) === REMOVE_PROJECT)
	)
})
const owner = computed(() => allMembers.value.find((member) => member.is_owner))
const { data: organizations, isError } = useQuery({
	queryKey: computed(() => ['user', auth.value?.user?.id, 'organizations']),
	queryFn: () => client.labrinth.users_v2.getOrganizations(auth.value!.user!.id),
	enabled: computed(() => !!auth.value?.user?.id && !!currentMember.value?.is_owner),
})
const targets = computed(() => [
	...allMembers.value
		.filter((member) => member.accepted && !member.is_owner)
		.map((member) => ({
			value: `user:${member.user.id}`,
			label: member.user.username,
			userId: member.user.id,
			avatar_url: member.user.avatar_url,
			username: member.user.username,
			role: member.role,
		})),
	...(organizations.value ?? [])
		.filter((org) => org.id !== organization.value?.id)
		.map((org) => ({
			value: `organization:${org.id}`,
			label: org.name,
			isOrganization: true,
			organizationId: org.id,
			avatar_url: org.icon_url,
			name: org.name,
			role: formatMessage(messages.organization),
		})),
])
const target = computed(() => targets.value.find((target) => target.value === selectedTarget.value))
const transfer = useMutation({
	mutationFn: async () => {
		if (!currentMember.value?.is_owner || !target.value) return
		if ('organizationId' in target.value) {
			await client.labrinth.organizations_v3.addProject(target.value.organizationId, {
				project_id: project.value.id,
			})
		} else {
			await client.labrinth.teams_v2.transferOwnership(project.value.team, {
				user_id: target.value.userId,
			})
		}
	},
	onSuccess: async () => {
		selectedTarget.value = ''
		await invalidate()
		addNotification({
			title: formatMessage(messages.transferred),
			text: formatMessage(messages.transferredDescription),
			type: 'success',
		})
	},
	onError: (error) =>
		addNotification({
			title: formatMessage(messages.transferFailed),
			text: error.message,
			type: 'error',
		}),
})
const transferDisabledReason = computed(() => {
	if (!currentMember.value?.is_owner) return messages.transferRequiresOwner
	if (transfer.isPending.value) return messages.transferInProgress
	return undefined
})
const removeFromOrganization = useMutation({
	mutationFn: ({
		organizationId,
		projectId,
		userId,
	}: {
		organizationId: string
		projectId: string
		userId: string
	}) =>
		client.labrinth.organizations_v3.removeProject(organizationId, projectId, {
			new_owner: userId,
		}),
	onSuccess: async () => {
		await invalidate()
	},
	onError: (error) =>
		addNotification({
			title: formatMessage(messages.removeFailed),
			text: error.message,
			type: 'error',
		}),
})

function confirmRemoveFromOrganization() {
	const org = organization.value
	const user = auth.value.user
	if (!org || !user || !canRemoveFromOrganization.value || removeFromOrganization.isPending.value)
		return
	removeFromOrganization.mutate({
		organizationId: org.id,
		projectId: project.value.id,
		userId: user.id,
	})
}
</script>

<template>
	<div class="danger-zone-row">
		<div>
			<h3 class="mb-1 mt-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.transferOwnership) }}
			</h3>
			<p class="m-0 leading-normal text-secondary">
				{{ formatMessage(messages.transferDescription) }}
			</p>
		</div>
		<div
			v-tooltip="transferDisabledReason ? formatMessage(transferDisabledReason) : undefined"
			class="inline-flex shrink-0"
			:class="{ 'cursor-not-allowed': transferDisabledReason }"
		>
			<Button
				size="sm"
				type="colored"
				color="red"
				class="disabled:pointer-events-none"
				:disabled="!!transferDisabledReason"
				@click="show($event)"
			>
				{{ formatMessage(messages.transferOwnership) }}
			</Button>
		</div>
		<NewModal
			ref="selectionModal"
			:header="formatMessage(messages.transferTitle, { project: project.title })"
			width="550px"
		>
			<div class="flex flex-col gap-4 pb-2">
				<p class="m-0">
					{{ formatMessage(messages.transferConfirmation) }}
				</p>
				<p v-if="organization" class="m-0 text-secondary">
					{{ formatMessage(messages.managedBy, { organization: organization.name }) }}
				</p>
				<Field :label="formatMessage(messages.transferTarget)" :input-id="transferTargetId">
					<ProjectOwnerCombobox
						v-model="selectedTarget"
						:options="targets"
						:input-id="transferTargetId"
						:disabled="transfer.isPending.value"
						hide-chevron
					/>
				</Field>
				<p v-if="!targets.length && !isError" class="m-0 text-secondary">
					{{ formatMessage(messages.missingTargets) }}
				</p>
				<p v-if="isError" role="alert" class="m-0 text-red">
					{{ formatMessage(messages.organizationsLoadFailed) }}
				</p>
			</div>
			<template #actions
				><div class="flex justify-end gap-2">
					<Button @click="selectionModal?.hide()">{{
						formatMessage(commonMessages.cancelButton)
					}}</Button
					><Button
						type="colored"
						color="red"
						:disabled="!target || transfer.isPending.value"
						@click="confirmTarget($event)"
						>{{ formatMessage(commonMessages.continueButton) }}</Button
					>
				</div></template
			>
		</NewModal>
		<ConfirmTransferProjectModal
			v-if="target"
			ref="transferModal"
			:project="{ name: project.title, icon_url: project.icon_url ?? null }"
			:current-owner="{
				avatar_url: (owner ? owner.user.avatar_url : organization?.icon_url) ?? null,
				username: owner?.user.username ?? organization?.name ?? '',
				role: owner?.role ?? formatMessage(messages.owner),
				organizationId: owner ? undefined : organization?.id,
			}"
			:transfer-to="target"
			:confirm-label="formatMessage(messages.transferOwnership)"
			:on-confirm="() => transfer.mutate()"
		/>
	</div>
	<div v-if="organization && canRemoveFromOrganization" class="danger-zone-row">
		<div>
			<h3 class="mb-1 mt-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.removeFromOrganization) }}
			</h3>
			<p class="m-0 leading-normal text-secondary">
				{{
					formatMessage(messages.removeFromOrganizationDescription, {
						organization: organization.name,
					})
				}}
			</p>
		</div>
		<Button
			size="sm"
			type="colored"
			color="red"
			:disabled="removeFromOrganization.isPending.value"
			@click="removeModal?.show($event)"
		>
			{{ formatMessage(messages.removeFromOrganization) }}
		</Button>
		<ConfirmModal
			ref="removeModal"
			:title="formatMessage(messages.removeTitle)"
			:description="formatMessage(messages.removeDescription)"
			:proceed-label="formatMessage(commonMessages.removeButton)"
			@proceed="confirmRemoveFromOrganization"
		/>
	</div>
</template>
