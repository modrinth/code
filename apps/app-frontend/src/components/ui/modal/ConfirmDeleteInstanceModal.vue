<template>
	<NewModal ref="modal" :header="modalHeader" fade="danger" max-width="500px">
		<Admonition type="critical" :header="formatMessage(messages.admonitionHeader)">
			{{ admonitionBody }}
		</Admonition>

		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button @click="modal?.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="red">
					<button @click="confirm">
						<TrashIcon />
						{{ deleteButtonLabel }}
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
import { computed, ref } from 'vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'app.instance.confirm-delete.header',
		defaultMessage: 'Delete instance',
	},
	bulkHeader: {
		id: 'app.instance.confirm-delete.bulk-header',
		defaultMessage: 'Delete {count} instances',
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
	bulkAdmonitionBody: {
		id: 'app.instance.confirm-delete.bulk-admonition-body',
		defaultMessage:
			'All data for these {count} instances will be permanently deleted, including their worlds, configs, and all installed content.',
	},
	deleteButton: {
		id: 'app.instance.confirm-delete.delete-button',
		defaultMessage: 'Delete instance',
	},
	bulkDeleteButton: {
		id: 'app.instance.confirm-delete.bulk-delete-button',
		defaultMessage: 'Delete {count} instances',
	},
})

const { count = 1 } = defineProps<{
	count?: number
}>()

const emit = defineEmits<{
	(e: 'delete'): void
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const modalHeader = computed(() =>
	count === 1 ? formatMessage(messages.header) : formatMessage(messages.bulkHeader, { count }),
)
const admonitionBody = computed(() =>
	count === 1
		? formatMessage(messages.admonitionBody)
		: formatMessage(messages.bulkAdmonitionBody, { count }),
)
const deleteButtonLabel = computed(() =>
	count === 1
		? formatMessage(messages.deleteButton)
		: formatMessage(messages.bulkDeleteButton, { count }),
)

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
