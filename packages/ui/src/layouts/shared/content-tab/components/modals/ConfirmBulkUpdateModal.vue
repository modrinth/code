<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header)"
		fade="warning"
		max-width="500px"
		:disable-close="disableClose"
	>
		<div class="flex flex-col gap-6">
			<Admonition type="warning" :header="formatMessage(messages.admonitionHeader)">
				{{ formatMessage(messages.admonitionBody, { count }) }}
			</Admonition>
			<InlineBackupCreator
				backup-name="Before bulk update"
				@update:disable-close="disableClose = $event"
				@update:buttons-disabled="buttonsDisabled = $event"
			/>
		</div>

		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button
						class="!border !border-surface-4"
						:disabled="buttonsDisabled"
						@click="modal?.hide()"
					>
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="orange">
					<button :disabled="buttonsDisabled" @click="confirm">
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

import Admonition from '#ui/components/base/Admonition.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import InlineBackupCreator from './InlineBackupCreator.vue'

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
			"Are you sure you want to update {count, plural, one {# project} other {# projects}} to their latest compatible version? It's recommended to update content one-by-one.",
	},
	updateButton: {
		id: 'content.confirm-bulk-update.update-button',
		defaultMessage: 'Update {count, plural, one {# project} other {# projects}}',
	},
})

defineProps<{
	count: number
	server?: boolean
}>()

const emit = defineEmits<{
	(e: 'update'): void
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const disableClose = ref(false)
const buttonsDisabled = ref(false)

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
