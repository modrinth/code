<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header, { action: downgrade ? 'downgrade' : 'update' })"
		fade="warning"
		max-width="500px"
		:on-hide="() => backupCreator?.cancelBackup()"
	>
		<div class="flex flex-col gap-6">
			<Admonition
				type="warning"
				:header="
					formatMessage(messages.admonitionHeader, { action: downgrade ? 'downgrade' : 'update' })
				"
			>
				{{ formatMessage(messages.admonitionBody) }}
			</Admonition>
			<InlineBackupCreator
				ref="backupCreator"
				:backup-name="downgrade ? 'Before modpack downgrade' : 'Before modpack update'"
				@update:buttons-disabled="buttonsDisabled = $event"
			/>
		</div>

		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button class="!border !border-surface-4" @click="handleCancel">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="orange">
					<button :disabled="buttonsDisabled" @click="handleConfirm">
						<DownloadIcon />
						{{
							formatMessage(messages.confirmButton, { action: downgrade ? 'downgrade' : 'update' })
						}}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { DownloadIcon, XIcon } from '@modrinth/assets'
import { ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import InlineBackupCreator from './InlineBackupCreator.vue'

defineProps<{
	downgrade?: boolean
	server?: boolean
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'content.confirm-modpack-update.header',
		defaultMessage: '{action, select, downgrade {Downgrade} other {Update}} modpack',
	},
	admonitionHeader: {
		id: 'content.confirm-modpack-update.admonition-header',
		defaultMessage: '{action, select, downgrade {Downgrade} other {Update}} warning',
	},
	admonitionBody: {
		id: 'content.confirm-modpack-update.admonition-body',
		defaultMessage: 'Any mods or content you added on top of the modpack will be deleted.',
	},
	confirmButton: {
		id: 'content.confirm-modpack-update.confirm-button',
		defaultMessage: '{action, select, downgrade {Downgrade} other {Update}} modpack',
	},
})

const emit = defineEmits<{
	(e: 'confirm' | 'cancel'): void
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const backupCreator = ref<InstanceType<typeof InlineBackupCreator>>()
const buttonsDisabled = ref(false)

function show() {
	modal.value?.show()
}

function handleConfirm() {
	modal.value?.hide()
	emit('confirm')
}

function handleCancel() {
	modal.value?.hide()
	emit('cancel')
}

defineExpose({
	show,
})
</script>
