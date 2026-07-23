<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header)"
		max-width="470px"
		@after-hide="emit('clear')"
	>
		<div class="flex flex-col gap-4">
			<Admonition type="warning">{{ formatMessage(messages.warning, { username }) }}</Admonition>
			<div class="flex min-w-0 items-center gap-2 rounded-[20px] bg-surface-2 p-3">
				<Avatar
					:src="row?.avatarUrl"
					:alt="formatMessage(messages.avatarAlt, { username })"
					:tint-by="username"
					size="40px"
					circle
					no-shadow
				/>
				<div class="flex min-w-0 flex-1 flex-col gap-0.5">
					<span class="min-w-0 truncate font-medium text-contrast">{{ username }}</span>
					<span class="truncate text-sm text-secondary">{{
						row ? methodLabels[row.method] : ''
					}}</span>
				</div>
			</div>
			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.effectsLabel) }}</span>
				<ul class="m-0 list-disc pl-6 text-primary">
					<li v-for="effect in effects" :key="effect.id" class="leading-6 marker:text-secondary">
						{{ formatMessage(effect) }}
					</li>
				</ul>
			</div>
			<div class="flex justify-end gap-2 pt-1">
				<ButtonStyled type="outlined"
					><button class="!border !border-surface-5" @click="modal?.hide()">
						<XIcon aria-hidden="true" />{{ formatMessage(commonMessages.cancelButton) }}
					</button></ButtonStyled
				>
				<ButtonStyled color="orange"
					><button :disabled="!row" @click="confirm">
						<UserXIcon aria-hidden="true" />{{ formatMessage(messages.removeButton) }}
					</button></ButtonStyled
				>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import { UserXIcon, XIcon } from '@modrinth/assets'
import {
	Admonition,
	Avatar,
	ButtonStyled,
	commonMessages,
	defineMessages,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import { methodLabels, type ShareRow } from './shared-instance-share-types'

const props = defineProps<{ row: ShareRow | null; memberCount: number }>()
const emit = defineEmits<{ confirm: [row: ShareRow]; clear: [] }>()
const modal = ref<InstanceType<typeof NewModal>>()
const { formatMessage } = useVIntl()
const username = computed(() => props.row?.username ?? '')
const messages = defineMessages({
	header: { id: 'app.instance.share.remove-user-modal.header', defaultMessage: 'Revoke access' },
	warning: {
		id: 'app.instance.share.remove-user-modal.warning-body',
		defaultMessage:
			"If you revoke {username}'s access to this shared instance, you'll need to invite them again before they can receive updates.",
	},
	avatarAlt: {
		id: 'app.instance.share.remove-user-modal.user-avatar-alt',
		defaultMessage: "{username}'s avatar",
	},
	effectsLabel: {
		id: 'app.instance.share.remove-user-modal.effects-label',
		defaultMessage: 'What happens?',
	},
	effectAccess: {
		id: 'app.instance.share.remove-user-modal.effect-access',
		defaultMessage: 'They will no longer receive updates for this shared instance',
	},
	effectInstalledCopy: {
		id: 'app.instance.share.remove-user-modal.effect-installed-copy',
		defaultMessage: 'Any copy they already installed will stay on their device',
	},
	effectInviteAgain: {
		id: 'app.instance.share.remove-user-modal.effect-invite-again',
		defaultMessage: 'You can invite them again later',
	},
	effectLastUser: {
		id: 'app.instance.share.remove-user-modal.effect-last-user',
		defaultMessage: 'This is the last user, sharing will be turned off for this instance',
	},
	removeButton: {
		id: 'app.instance.share.remove-user-modal.remove-button',
		defaultMessage: 'Revoke access',
	},
})
const effects = computed(() => {
	const result = [messages.effectAccess, messages.effectInstalledCopy, messages.effectInviteAgain]
	if (props.memberCount === 1) result.push(messages.effectLastUser)
	return result
})

function show(event?: MouseEvent) {
	modal.value?.show(event)
}
function confirm() {
	if (props.row) {
		modal.value?.hide()
		emit('confirm', props.row)
	}
}
defineExpose({ show })
</script>
