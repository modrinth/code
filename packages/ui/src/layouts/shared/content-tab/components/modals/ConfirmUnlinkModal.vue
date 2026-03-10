<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header)"
		fade="warning"
		max-width="500px"
		:on-hide="() => backupCreator?.cancelBackup()"
	>
		<div class="flex flex-col gap-6">
			<Admonition type="warning" :header="formatMessage(messages.admonitionHeader)">
				{{ formatMessage(messages.admonitionBody) }}
			</Admonition>
			<InlineBackupCreator
				ref="backupCreator"
				backup-name="Before unlink"
				@update:buttons-disabled="buttonsDisabled = $event"
			/>
		</div>

		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button
						class="!border !border-surface-4"
						@click="modal?.hide()"
					>
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="orange">
					<button :disabled="buttonsDisabled" @click="confirm">
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

import Admonition from '#ui/components/base/Admonition.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import InlineBackupCreator from './InlineBackupCreator.vue'

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
		defaultMessage: 'Unlinking modpack',
	},
	admonitionBody: {
		id: 'content.confirm-unlink.admonition-body',
		defaultMessage:
			'Mods and content will be merged with what you added on top of the modpack, and it will stop receiving updates.',
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
const backupCreator = ref<InstanceType<typeof InlineBackupCreator>>()
const buttonsDisabled = ref(false)

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
