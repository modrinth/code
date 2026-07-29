<template>
	<Modal ref="modal" :header="formatMessage(messages.header)" fade="danger" max-width="500px">
		<Admonition type="critical" :header="formatMessage(messages.admonitionHeader)">
			<IntlFormatted :message-id="messages.admonitionBody" :values="{ code: inviteCode }">
				<template #monospace="{ children }">
					<code class="font-mono"><component :is="() => children" /></code>
				</template>
			</IntlFormatted>
		</Admonition>

		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled type="outlined">
					<button @click="modal?.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="red">
					<button @click="confirm">
						<XIcon />
						{{ formatMessage(messages.revokeButton) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</Modal>
</template>

<script setup lang="ts">
import { XIcon } from '@modrinth/assets'
import {
	Admonition,
	ButtonStyled,
	commonMessages,
	defineMessages,
	IntlFormatted,
	Modal,
	useVIntl,
} from '@modrinth/ui'
import { ref } from 'vue'

const { formatMessage } = useVIntl()
const modal = ref<InstanceType<typeof Modal>>()
const inviteCode = ref('')

const emit = defineEmits<{
	revoke: [inviteCode: string]
}>()

function show(code: string) {
	inviteCode.value = code
	modal.value?.show()
}

function confirm() {
	modal.value?.hide()
	emit('revoke', inviteCode.value)
}

const messages = defineMessages({
	header: {
		id: 'instance.settings.sharing.revoke-invite.header',
		defaultMessage: 'Revoke invite',
	},
	admonitionHeader: {
		id: 'instance.settings.sharing.revoke-invite.admonition-header',
		defaultMessage: 'This action cannot be undone',
	},
	admonitionBody: {
		id: 'instance.settings.sharing.revoke-invite.admonition-body',
		defaultMessage:
			'The invite link <monospace>{code}</monospace> will stop working immediately. People who already joined will keep access.',
	},
	revokeButton: {
		id: 'instance.settings.sharing.revoke-invite.confirm',
		defaultMessage: 'Revoke invite',
	},
})

defineExpose({ show })
</script>
