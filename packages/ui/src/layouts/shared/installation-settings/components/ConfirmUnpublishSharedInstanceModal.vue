<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" fade="warning" max-width="500px">
		<Admonition type="warning" :header="formatMessage(messages.admonitionHeader)">
			{{ formatMessage(messages.admonitionBody) }}
		</Admonition>

		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button @click="modal?.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="orange">
					<button
						v-tooltip="props.actionDisabled ? props.actionDisabledTooltip : undefined"
						:disabled="props.actionDisabled"
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
import { useDebugLogger } from '#ui/composables/debug-logger'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

const props = defineProps<{
	actionDisabled?: boolean
	actionDisabledTooltip?: string
}>()

const emit = defineEmits<{
	(e: 'unpublish'): void
}>()

const { formatMessage } = useVIntl()
const debug = useDebugLogger('ConfirmUnpublishSharedInstanceModal')
const modal = ref<InstanceType<typeof NewModal>>()

const messages = defineMessages({
	header: {
		id: 'installation-settings.unpublish-shared-instance.modal.header',
		defaultMessage: 'Unpublish shared instance',
	},
	admonitionHeader: {
		id: 'installation-settings.unpublish-shared-instance.modal.admonition-header',
		defaultMessage: 'Unpublishing shared instance',
	},
	admonitionBody: {
		id: 'installation-settings.unpublish-shared-instance.modal.admonition-body',
		defaultMessage:
			"This deletes the shared instance from Modrinth's servers. People using it in the Modrinth App will stop receiving updates, but your local instance and its content will stay on this device.",
	},
	action: {
		id: 'installation-settings.unpublish-shared-instance.modal.action',
		defaultMessage: 'Unpublish shared instance',
	},
})

function show() {
	debug('show: called', {
		hasModalRef: !!modal.value,
		actionDisabled: props.actionDisabled,
	})
	modal.value?.show()
}

function confirm() {
	debug('confirm: called', {
		hasModalRef: !!modal.value,
		actionDisabled: props.actionDisabled,
	})
	if (props.actionDisabled) {
		debug('confirm: ignored actionDisabled')
		return
	}
	modal.value?.hide()
	emit('unpublish')
}

defineExpose({
	show,
})
</script>
