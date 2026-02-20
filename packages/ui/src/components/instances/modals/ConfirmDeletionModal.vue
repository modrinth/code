<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header, { count, itemType })"
		:fade="variant === 'server' ? 'warning' : 'danger'"
		max-width="500px"
	>
		<div class="flex flex-col gap-6">
			<Admonition
				:type="variant === 'server' ? 'warning' : 'critical'"
				:header="formatMessage(messages.admonitionHeader)"
			>
				{{
					formatMessage(
						variant === 'server' ? messages.admonitionBodyServer : messages.admonitionBody,
					)
				}}
			</Admonition>
			<span class="text-primary">
				<IntlFormatted
					v-if="variant === 'server'"
					:message-id="backupLink ? messages.warningBodyServerBackup : messages.warningBodyServer"
				>
					<template #backup="{ children }">
						<RouterLink :to="backupLink!" class="text-link hover:underline" @click="modal?.hide()">
							<component :is="() => children" />
						</RouterLink>
					</template>
				</IntlFormatted>
				<template v-else>
					{{ formatMessage(messages.warningBody) }}
				</template>
			</span>
		</div>

		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button @click="modal?.hide()" class="!border !border-surface-4">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled :color="variant === 'server' ? 'orange' : 'red'">
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
import IntlFormatted from '../../base/IntlFormatted.vue'
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
			'Deleting a mod can permanently affect your worlds and may cause missing content or unexpected issues when loading again. Consider making a backup before continuing.',
	},
	admonitionBodyServer: {
		id: 'content.confirm-deletion.admonition-body-server',
		defaultMessage:
			'Deleting a mod can permanently affect your world and may cause missing content or unexpected issues when it loads again.',
	},
	warningBody: {
		id: 'content.confirm-deletion.warning-body',
		defaultMessage:
			'This action is irreversible. Consider making a backup of your worlds before continuing.',
	},
	warningBodyServer: {
		id: 'content.confirm-deletion.warning-body-server',
		defaultMessage:
			'We recommend creating a backup before proceeding so you can restore your server if anything breaks.',
	},
	warningBodyServerBackup: {
		id: 'content.confirm-deletion.warning-body-server-backup',
		defaultMessage:
			'We recommend creating a <backup>backup</backup> before proceeding so you can restore your server if anything breaks.',
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
		backupLink?: string
	}>(),
	{
		variant: 'instance',
	},
)

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
