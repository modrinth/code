<template>
	<NewModal
		ref="modal"
		:header="formatMessage(shouldCancel ? messages.cancelHeader : messages.header, { username })"
		fade="danger"
		max-width="500px"
	>
		<div class="flex flex-col gap-6">
			<Admonition
				type="critical"
				:header="
					formatMessage(shouldCancel ? messages.cancelAdmonitionHeader : messages.admonitionHeader)
				"
			>
				{{
					formatMessage(shouldCancel ? messages.cancelAdmonitionBody : messages.admonitionBody, {
						username,
					})
				}}
			</Admonition>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled type="outlined">
					<button class="!border !border-surface-4" @click="hide">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="red">
					<button @click="confirm">
						<XIcon v-if="shouldCancel" />
						<UserXIcon v-else />
						{{ formatMessage(shouldCancel ? messages.cancelButton : messages.removeButton) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { UserXIcon, XIcon } from '@modrinth/assets'
import { ref } from 'vue'

import Admonition from '../../base/Admonition.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'
import NewModal from '../../modal/NewModal.vue'
import { defineMessages, useVIntl } from '../../../composables/i18n'
import { commonMessages } from '../../../utils/common-messages'

withDefaults(
	defineProps<{
		username: string
		shouldCancel?: boolean
	}>(),
	{
		shouldCancel: false,
	},
)

const emit = defineEmits<{
	remove: []
}>()

const { formatMessage } = useVIntl()
const modal = ref<InstanceType<typeof NewModal>>()

const messages = defineMessages({
	header: {
		id: 'servers.remove-access-modal.header',
		defaultMessage: 'Remove user',
	},
	cancelHeader: {
		id: 'servers.remove-access-modal.cancel-header',
		defaultMessage: 'Cancel invite',
	},
	admonitionHeader: {
		id: 'servers.remove-access-modal.admonition-header',
		defaultMessage: 'Removal warning',
	},
	cancelAdmonitionHeader: {
		id: 'servers.remove-access-modal.cancel-admonition-header',
		defaultMessage: 'Cancellation warning',
	},
	admonitionBody: {
		id: 'servers.remove-access-modal.admonition-body',
		defaultMessage:
			'{username} will no longer be able to manage or view this server. You can add them again later.',
	},
	cancelAdmonitionBody: {
		id: 'servers.remove-access-modal.cancel-admonition-body',
		defaultMessage: '{username} will need a new invitation before they can join this server.',
	},
	removeButton: {
		id: 'servers.remove-access-modal.remove-button',
		defaultMessage: 'Remove user',
	},
	cancelButton: {
		id: 'servers.remove-access-modal.cancel-button',
		defaultMessage: 'Cancel invite',
	},
})

function show() {
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

function confirm() {
	hide()
	emit('remove')
}

defineExpose({ show, hide })
</script>
