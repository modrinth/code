<script setup lang="ts">
import { TransferIcon } from '@modrinth/assets'
import {
	Button,
	Combobox,
	ConfirmModal,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { isAdmin } from '@modrinth/utils'
import { useMutation, useQuery } from '@tanstack/vue-query'
import { computed, nextTick, ref, useTemplateRef } from 'vue'

import ConfirmTransferProjectModal from '~/components/ui/ConfirmTransferProjectModal.vue'

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
			subLabel: 'Project member',
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
			subLabel: 'Organization',
			organizationId: org.id,
			avatar_url: org.icon_url,
			name: org.name,
			role: 'Organization',
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
			title: 'Ownership transferred',
			text: 'Your project ownership has been updated.',
			type: 'success',
		})
	},
	onError: (error) =>
		addNotification({
			title: 'Transfer failed',
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
		client.labrinth.organizations_v3.removeProject(organizationId, projectId, { new_owner: userId }),
	onSuccess: async () => {
		await invalidate()
	},
	onError: (error) =>
		addNotification({
			title: 'Unable to remove project',
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
	<div id="ownership" class="danger-zone-row">
		<div>
			<h3 class="mb-1 mt-0 text-base font-semibold text-contrast">
				{{ formatMessage(messages.transferOwnership) }}
			</h3>
			<p class="m-0 text-sm leading-relaxed text-secondary">
				Give full control of this project to another member or an organization. You’ll stay on as a
				member, but only the new owner can transfer it back.
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
		<NewModal ref="selectionModal" :header="`Transfer ${project.title}`" width="550px">
			<div class="flex flex-col gap-4 pb-2">
				<p class="m-0">
					The new owner will get full control of this project, including members, revenue,
					transferring, and deleting it. Only the new owner can transfer it back.
				</p>
				<p v-if="organization" class="m-0 text-secondary">
					This project is managed by {{ organization.name }}.
				</p>
				<label for="project-transfer-target" class="font-semibold text-contrast"
					>Member or organization</label
				>
				<Combobox
					id="project-transfer-target"
					v-model="selectedTarget"
					:options="targets"
					searchable
					placeholder="Select a member or organization"
					:disabled="transfer.isPending.value"
				/>
				<p v-if="!targets.length && !isError" class="m-0 text-secondary">
					Invite a member from the Members page or create an organization before transferring
					ownership.
				</p>
				<p v-if="isError" role="alert" class="m-0 text-red">
					Organizations could not be loaded. Try again later.
				</p>
			</div>
			<template #actions
				><div class="flex justify-end gap-2">
					<Button @click="selectionModal?.hide()">Cancel</Button
					><Button
						type="colored"
						color="red"
						:disabled="!target || transfer.isPending.value"
						@click="confirmTarget($event)"
						>Continue</Button
					>
				</div></template
			>
		</NewModal>
		<ConfirmTransferProjectModal
			v-if="target"
			ref="transferModal"
			:project="{ name: project.title, icon_url: project.icon_url ?? null }"
			:current-owner="{
				avatar_url: owner?.user.avatar_url ?? null,
				username: owner?.user.username ?? organization?.name ?? '',
				role: owner?.role ?? 'Owner',
			}"
			:transfer-to="target"
			:on-confirm="() => transfer.mutate()"
		/>
	</div>
	<div v-if="organization && canRemoveFromOrganization" class="danger-zone-row">
		<div>
			<h3 class="mb-1 mt-0 text-base font-semibold text-contrast">
				{{ formatMessage(messages.removeFromOrganization) }}
			</h3>
			<p class="m-0 text-sm leading-relaxed text-secondary">
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
			title="Remove project from organization?"
			description="You will become the owner of this project and it will no longer be managed by the organization."
			proceed-label="Remove"
			@proceed="confirmRemoveFromOrganization"
		/>
	</div>
</template>
