<template>
	<NewModal ref="modal" fade="danger" :header="formatMessage(messages.header)" max-width="500px">
		<Admonition type="critical" class="md:min-w-[400px]">
			<template #header>{{ formatMessage(messages.deletingName, { name: item?.name }) }}</template>
			{{ formatMessage(messages.deleteWarning, { type: item?.type }) }}
		</Admonition>
		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button class="!border !border-surface-4" @click="hide">
						<XIcon class="h-5 w-5" />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="red">
					<button @click="handleSubmit">
						<TrashIcon class="h-5 w-5" />
						{{ formatMessage(commonMessages.deleteLabel) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { TrashIcon, XIcon } from '@modrinth/assets'
import { ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'files.delete-modal.header',
		defaultMessage: 'Delete file',
	},
	deletingName: {
		id: 'files.delete-modal.deleting-name',
		defaultMessage: 'Deleting "{name}"',
	},
	deleteWarning: {
		id: 'files.delete-modal.warning',
		defaultMessage:
			'{type, select, directory {This folder and all its contents will be permanently deleted. This action cannot be undone.} other {This file will be permanently deleted. This action cannot be undone.}}',
	},
})

defineProps<{
	item: {
		name: string
		type: string
	} | null
}>()

const emit = defineEmits<{
	delete: []
}>()

const modal = ref<InstanceType<typeof NewModal>>()

const handleSubmit = () => {
	emit('delete')
	hide()
}

const show = () => {
	modal.value?.show()
}

const hide = () => {
	modal.value?.hide()
}

defineExpose({ show, hide })
</script>
