<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header)"
		fade="danger"
		max-width="500px"
		:on-hide="() => backupCreator?.cancelBackup()"
	>
		<div class="flex flex-col gap-6">
			<Admonition type="critical" :header="formatMessage(messages.admonitionHeader)">
				{{ formatMessage(messages.admonitionBody) }}
			</Admonition>
			<InlineBackupCreator
				ref="backupCreator"
				:backup-name="backupTip ? `Before reinstall (${backupTip})` : 'Before reinstall'"
				:target-type="targetType"
				@update:buttons-disabled="buttonsDisabled = $event"
			/>
		</div>

		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button @click="modal?.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="red">
					<button :disabled="buttonsDisabled" @click="confirm">
						<DownloadIcon />
						{{ formatMessage(messages.reinstallButton) }}
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
import { useDebugLogger } from '#ui/composables/debug-logger'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import InlineBackupCreator from './InlineBackupCreator.vue'

const { formatMessage } = useVIntl()
const debug = useDebugLogger('ConfirmReinstallModal')

const messages = defineMessages({
	header: {
		id: 'instance.confirm-reinstall.header',
		defaultMessage: 'Reinstall modpack',
	},
	admonitionHeader: {
		id: 'instance.confirm-reinstall.admonition-header',
		defaultMessage: 'Reinstallation warning',
	},
	admonitionBody: {
		id: 'instance.confirm-reinstall.admonition-body',
		defaultMessage:
			'Reinstalling will reset all installed or modified content to what is provided by the modpack, removing any mods or content you have added on top of the original installation.',
	},
	reinstallButton: {
		id: 'instance.confirm-reinstall.reinstall-button',
		defaultMessage: 'Reinstall modpack',
	},
})

defineProps<{
	server?: boolean
	backupTip?: string
	targetType?: 'server' | 'instance'
}>()

const emit = defineEmits<{
	(e: 'reinstall'): void
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const backupCreator = ref<InstanceType<typeof InlineBackupCreator>>()
const buttonsDisabled = ref(false)

function show() {
	debug('show: called', {
		hasModalRef: !!modal.value,
		hasBackupCreatorRef: !!backupCreator.value,
		buttonsDisabled: buttonsDisabled.value,
	})
	modal.value?.show()
	debug('show: returned from modal.show', {
		hasModalRef: !!modal.value,
		hasBackupCreatorRef: !!backupCreator.value,
		buttonsDisabled: buttonsDisabled.value,
	})
}

function confirm() {
	debug('confirm: called', {
		hasModalRef: !!modal.value,
		buttonsDisabled: buttonsDisabled.value,
	})
	modal.value?.hide()
	emit('reinstall')
	debug('confirm: emitted reinstall')
}

defineExpose({
	show,
})
</script>
