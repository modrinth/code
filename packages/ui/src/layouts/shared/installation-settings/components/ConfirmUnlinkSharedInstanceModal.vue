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
				backup-name="Before unlinking shared instance"
				@update:buttons-disabled="buttonsDisabled = $event"
			/>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled type="outlined">
					<button @click="modal?.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="orange">
					<button
						v-tooltip="props.actionDisabled ? props.actionDisabledTooltip : undefined"
						:disabled="buttonsDisabled || props.actionDisabled"
						@click="confirm"
					>
						<UnlinkIcon />
						{{ formatMessage(messages.action) }}
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

import InlineBackupCreator from '../../content-tab/components/modals/InlineBackupCreator.vue'

const props = defineProps<{
	actionDisabled?: boolean
	actionDisabledTooltip?: string
}>()

const emit = defineEmits<{
	unlink: []
}>()

const { formatMessage } = useVIntl()
const modal = ref<InstanceType<typeof NewModal>>()
const backupCreator = ref<InstanceType<typeof InlineBackupCreator>>()
const buttonsDisabled = ref(false)

const messages = defineMessages({
	header: {
		id: 'installation-settings.unlink-shared-instance.modal.header',
		defaultMessage: 'Unlink shared instance',
	},
	admonitionHeader: {
		id: 'installation-settings.unlink-shared-instance.modal.admonition-header',
		defaultMessage: 'Unlinking shared instance',
	},
	admonitionBody: {
		id: 'installation-settings.unlink-shared-instance.modal.admonition-body',
		defaultMessage:
			'This only affects your local instance. Your installed content will stay on this device, and the shared instance and other people using it will not be affected.',
	},
	action: {
		id: 'installation-settings.unlink-shared-instance.modal.action',
		defaultMessage: 'Unlink shared instance',
	},
})

function show() {
	modal.value?.show()
}

function confirm() {
	if (buttonsDisabled.value || props.actionDisabled) return

	modal.value?.hide()
	emit('unlink')
}

defineExpose({ show })
</script>
