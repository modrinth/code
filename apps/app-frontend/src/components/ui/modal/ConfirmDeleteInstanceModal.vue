<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" fade="danger" max-width="500px">
		<Admonition type="critical" :header="formatMessage(messages.admonitionHeader)">
			{{ formatMessage(messages.admonitionBody) }}
		</Admonition>

		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button class="!border !border-surface-4" @click="modal?.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="red">
					<button @click="confirm">
						<TrashIcon />
						{{ formatMessage(messages.deleteButton) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { TrashIcon, XIcon } from '@modrinth/assets'
import {
	Admonition,
	ButtonStyled,
	commonMessages,
	defineMessages,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { ref } from 'vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'app.instance.confirm-delete.header',
		defaultMessage: 'Delete instance',
	},
	admonitionHeader: {
		id: 'app.instance.confirm-delete.admonition-header',
		defaultMessage: 'This action cannot be undone',
	},
	admonitionBody: {
		id: 'app.instance.confirm-delete.admonition-body',
		defaultMessage:
			'All data for your instance will be permanently deleted, including your worlds, configs, and all installed content.',
	},
	deleteButton: {
		id: 'app.instance.confirm-delete.delete-button',
		defaultMessage: 'Delete instance',
	},
})

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
