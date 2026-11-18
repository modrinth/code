<script setup lang="ts">
import { Button, commonMessages, defineMessages, NewModal, useVIntl } from '@modrinth/ui'
import { computed, shallowRef, useId, useTemplateRef } from 'vue'

import type { MemberDraft } from './types'

const props = defineProps<{
	teamName: string
	organization?: boolean
	canRemove?: (member: MemberDraft) => boolean
	hasRevenue?: boolean
	busy: boolean
	removing: boolean
}>()
const emit = defineEmits<{ confirm: [member: MemberDraft] }>()
const modal = useTemplateRef('modal')
const member = shallowRef<MemberDraft>()
const descriptionId = useId()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'project.settings.members.remove-modal.title',
		defaultMessage: 'Remove {username}?',
	},
	description: {
		id: 'project.settings.members.remove-modal.description',
		defaultMessage:
			'They’ll lose access to {project} straight away and stop receiving their share of revenue. You can add them again later.',
	},
	serverDescription: {
		id: 'project.settings.members.remove-modal.server-description',
		defaultMessage: 'They’ll lose access to {project} straight away. You can add them again later.',
	},
	pendingDescription: {
		id: 'project.settings.members.remove-modal.pending-description',
		defaultMessage:
			'They’ll no longer be able to accept their invite to {project}. You can invite them again later.',
	},
	organizationDescription: {
		id: 'organization.settings.members.remove-modal.description',
		defaultMessage:
			'They’ll lose membership in {project} straight away. You can invite them again later.',
	},
	cancelInviteTitle: {
		id: 'organization.settings.members.remove-modal.cancel-invite-title',
		defaultMessage: 'Cancel invite for {username}?',
	},
	cancelInvite: {
		id: 'organization.settings.members.remove-modal.cancel-invite',
		defaultMessage: 'Cancel invite',
	},
	confirm: { id: 'project.settings.members.remove-modal.confirm', defaultMessage: 'Remove member' },
})

const description = computed(() =>
	formatMessage(
		member.value?.accepted
			? props.organization
				? messages.organizationDescription
				: props.hasRevenue
					? messages.description
					: messages.serverDescription
			: messages.pendingDescription,
		{ project: props.teamName },
	),
)
const cancelingInvite = computed(() => props.organization && member.value?.accepted === false)
const canConfirm = computed(
	() => !!member.value && !props.busy && (props.canRemove?.(member.value) ?? true),
)
const title = computed(() =>
	member.value
		? formatMessage(cancelingInvite.value ? messages.cancelInviteTitle : messages.title, {
				username: member.value.user.username,
			})
		: undefined,
)

function show(target: MemberDraft, event: MouseEvent) {
	member.value = target
	modal.value?.show(event)
}

function confirm() {
	if (member.value && canConfirm.value) emit('confirm', member.value)
}

defineExpose({ show, hide: () => modal.value?.hide() })
</script>

<template>
	<NewModal
		ref="modal"
		width="28rem"
		:header="title"
		:aria-describedby="descriptionId"
		:disable-close="removing"
		:on-after-hide="() => (member = undefined)"
	>
		<p v-if="member" :id="descriptionId" class="m-0 pb-2">{{ description }}</p>
		<template #actions>
			<div class="flex flex-wrap justify-end gap-2">
				<Button :disabled="removing" @click="modal?.hide()">
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button type="colored" color="red" :disabled="!canConfirm" @click="confirm">
					{{ formatMessage(cancelingInvite ? messages.cancelInvite : messages.confirm) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>
