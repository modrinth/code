<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header, { count, itemType })"
		:fade="variant === 'server' ? 'warning' : 'danger'"
		max-width="500px"
		:on-hide="() => backupCreator?.cancelBackup()"
	>
		<div class="flex flex-col gap-6">
			<Admonition
				:type="variant === 'server' ? 'warning' : 'critical'"
				:header="formatMessage(messages.admonitionHeader)"
			>
				{{ formatMessage(messages.admonitionBody) }}
			</Admonition>
			<InlineBackupCreator
				ref="backupCreator"
				backup-name="Before deletion"
				@update:buttons-disabled="buttonsDisabled = $event"
			/>
		</div>

		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button class="!border !border-surface-4" @click="modal?.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled :color="variant === 'server' ? 'orange' : 'red'">
					<button :disabled="buttonsDisabled" @click="confirm">
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

import Admonition from '#ui/components/base/Admonition.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import InlineBackupCreator from './InlineBackupCreator.vue'

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
			'Deleting a mod can permanently affect your world and may cause missing content or unexpected issues when it loads again.',
	},
	deleteButton: {
		id: 'content.confirm-deletion.delete-button',
		defaultMessage: 'Delete {count} {itemType}{count, plural, one {} other {s}}',
	},
})

withDefaults(
	defineProps<{
		count: number
		itemType: string
		variant?: 'instance' | 'server'
	}>(),
	{
		variant: 'instance',
	},
)

const emit = defineEmits<{
	(e: 'delete'): void
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const backupCreator = ref<InstanceType<typeof InlineBackupCreator>>()
const buttonsDisabled = ref(false)

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
