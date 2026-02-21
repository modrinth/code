<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" fade="warning" max-width="500px">
		<div class="flex flex-col gap-6">
			<Admonition type="warning" :header="formatMessage(messages.admonitionHeader)">
				{{ formatMessage(messages.admonitionBody, { count }) }}
			</Admonition>
			<span class="text-primary"> {{ formatMessage(messages.warningBody) }}</span>
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
						<DownloadIcon />
						{{ formatMessage(messages.updateButton, { count }) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { DownloadIcon, XIcon } from '@modrinth/assets'
import { ref } from 'vue'

import { defineMessages, useVIntl } from '../../../composables/i18n'
import { commonMessages } from '../../../utils/common-messages'
import Admonition from '../../base/Admonition.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'
import NewModal from '../../modal/NewModal.vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'content.confirm-bulk-update.header',
		defaultMessage: 'Update projects',
	},
	admonitionHeader: {
		id: 'content.confirm-bulk-update.admonition-header',
		defaultMessage: 'Update warning',
	},
	admonitionBody: {
		id: 'content.confirm-bulk-update.admonition-body',
		defaultMessage:
			'Are you sure you want to update {count, plural, one {# project} other {# projects}} to their latest compatible version?',
	},
	warningBody: {
		id: 'content.confirm-bulk-update.warning-body',
		defaultMessage:
			"Updating can break your instance. New incompatibilities may be introduced. It's recommended to update content one-by-one. Proceed with caution and back up your instance first.",
	},
	updateButton: {
		id: 'content.confirm-bulk-update.update-button',
		defaultMessage: 'Update {count, plural, one {# project} other {# projects}}',
	},
})

defineProps<{
	count: number
}>()

const emit = defineEmits<{
	(e: 'update'): void
}>()

const modal = ref<InstanceType<typeof NewModal>>()

function show() {
	modal.value?.show()
}

function confirm() {
	modal.value?.hide()
	emit('update')
}

defineExpose({
	show,
})
</script>
