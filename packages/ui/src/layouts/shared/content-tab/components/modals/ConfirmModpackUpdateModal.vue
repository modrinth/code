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
				{{
					formatMessage(messages.admonitionBody, {
						action: downgrade ? 'downgrade' : 'update',
					})
				}}
			</Admonition>
			<InlineBackupCreator
				ref="backupCreator"
				:backup-name="backupName"
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
import { computed, ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import InlineBackupCreator from './InlineBackupCreator.vue'

const props = defineProps<{
	downgrade?: boolean
	backupTip?: string
}>()

const { formatMessage } = useVIntl()

const backupName = computed(() => {
	const action = props.downgrade ? 'downgrade' : 'update'
	return props.backupTip
		? `Before modpack ${action} (${props.backupTip})`
		: `Before modpack ${action}`
})

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
		defaultMessage:
			'{action, select, downgrade {Downgrading} other {Updating}} may cause compatibility issues. Mods or content you added on top of the modpack will be kept, but may not be compatible with the new version.',
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
