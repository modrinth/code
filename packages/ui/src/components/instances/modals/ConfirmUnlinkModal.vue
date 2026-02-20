<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" fade="warning" max-width="500px">
		<div class="flex flex-col gap-6">
			<Admonition type="warning" :header="formatMessage(messages.admonitionHeader)">
				{{ formatMessage(server ? messages.serverAdmonitionBody : messages.admonitionBody) }}
			</Admonition>
			<span class="text-primary">
				{{ formatMessage(server ? messages.serverWarningBody : messages.warningBody) }}
			</span>
		</div>

		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button class="!border !border-surface-4" @click="modal?.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="orange">
					<button @click="confirm">
						<UnlinkIcon />
						{{ formatMessage(server ? messages.header : messages.unlinkButton) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { UnlinkIcon, XIcon } from '@modrinth/assets'
import { ref } from 'vue'

import { defineMessages, useVIntl } from '../../../composables/i18n'
import { commonMessages } from '../../../utils/common-messages'
import Admonition from '../../base/Admonition.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'
import NewModal from '../../modal/NewModal.vue'

defineProps<{
	server?: boolean
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'content.confirm-unlink.header',
		defaultMessage: 'Unlink modpack',
	},
	admonitionHeader: {
		id: 'content.confirm-unlink.admonition-header',
		defaultMessage: 'Unlink warning',
	},
	admonitionBody: {
		id: 'content.confirm-unlink.admonition-body',
		defaultMessage:
			'Are you sure you want to unlink the modpack from your instance? Modpack content will remain installed, but will no longer be managed.',
	},
	warningBody: {
		id: 'content.confirm-unlink.warning-body',
		defaultMessage:
			'This action is irreversable. You will need to create a new instance with the modpack if you change your mind.',
	},
	serverAdmonitionBody: {
		id: 'content.confirm-unlink.server-admonition-body',
		defaultMessage:
			'Unlinking will merge all mods, resource packs and or plugins associated with this modpack with your own mods.',
	},
	serverWarningBody: {
		id: 'content.confirm-unlink.server-warning-body',
		defaultMessage: 'We will automatically create a backup if you continue.',
	},
	unlinkButton: {
		id: 'content.confirm-unlink.unlink-button',
		defaultMessage: 'Unlink',
	},
})

const emit = defineEmits<{
	(e: 'unlink'): void
}>()

const modal = ref<InstanceType<typeof NewModal>>()

function show() {
	modal.value?.show()
}

function confirm() {
	modal.value?.hide()
	emit('unlink')
}

defineExpose({
	show,
})
</script>
