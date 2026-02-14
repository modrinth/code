<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header, { count, itemType })"
		fade="danger"
		max-width="500px"
	>
		<div class="flex flex-col gap-6">
			<Admonition type="critical" :header="formatMessage(messages.admonitionHeader)">
				{{ formatMessage(messages.admonitionBody) }}
			</Admonition>
			<span class="text-primary"> {{ formatMessage(messages.warningBody) }}</span>
		</div>

		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled>
					<button @click="modal?.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="red">
					<button @click="confirm">
						<TrashIcon />
						{{ formatMessage(messages.deleteButton, { count, itemType }) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { TrashIcon, XIcon } from '@modrinth/assets'
import { ref } from 'vue'

import { defineMessages, useVIntl } from '../../../composables/i18n'
import { commonMessages } from '../../../utils/common-messages'
import Admonition from '../../base/Admonition.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'
import NewModal from '../../modal/NewModal.vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'content.confirm-deletion.header',
		defaultMessage: 'Delete {itemType}{count, plural, one {} other {s}}',
	},
	admonitionHeader: {
		id: 'content.confirm-deletion.admonition-header',
		defaultMessage: 'Deletion warning',
	},
	admonitionBody: {
		id: 'content.confirm-deletion.admonition-body',
		defaultMessage:
			'Removing content from your instance may corrupt worlds where they were used. Are you sure you want to continue?',
	},
	warningBody: {
		id: 'content.confirm-deletion.warning-body',
		defaultMessage:
			'This action is irreversable. Consider making a backup of your worlds before continuing.',
	},
	deleteButton: {
		id: 'content.confirm-deletion.delete-button',
		defaultMessage: 'Delete {count} {itemType}{count, plural, one {} other {s}}',
	},
})

defineProps<{
	count: number
	itemType: string
}>()

const emit = defineEmits<{
	(e: 'delete'): void
}>()

const modal = ref<InstanceType<typeof NewModal>>()

function show() {
	modal.value?.show()
}

function confirm() {
	modal.value?.hide()
	emit('delete')
}

defineExpose({
	show,
})
</script>
