<template>
	<NewModal
		ref="modal"
		:header="
			formatMessage(messages.header, {
				itemType: formatContentTypeSentence(formatMessage, visibleItemType, visibleCount),
			})
		"
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
				:backup-name="props.backupTip ? `Before deletion (${props.backupTip})` : 'Before deletion'"
				:target-type="props.targetType"
				@update:buttons-disabled="buttonsDisabled = $event"
			/>
		</div>

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
						:disabled="buttonsDisabled || props.actionDisabled"
						@click="confirm"
					>
						<TrashIcon />
						{{
							formatMessage(messages.deleteButton, {
								count: visibleCount,
								itemType: formatContentTypeSentence(formatMessage, visibleItemType, visibleCount),
							})
						}}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { TrashIcon, XIcon } from '@modrinth/assets'
import { nextTick, ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages, formatContentTypeSentence } from '#ui/utils/common-messages'

import InlineBackupCreator from './InlineBackupCreator.vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'content.confirm-deletion.header',
		defaultMessage: 'Delete {itemType}',
	},
	admonitionHeader: {
		id: 'content.confirm-deletion.admonition-header',
		defaultMessage: 'Deletion warning',
	},
	admonitionBody: {
		id: 'content.confirm-deletion.admonition-body',
		defaultMessage:
			'Deleting a mod can permanently affect your instance and may cause missing content or unexpected issues when it starts again.',
	},
	deleteButton: {
		id: 'content.confirm-deletion.delete-button',
		defaultMessage: 'Delete {count, number} {itemType}',
	},
})

const props = withDefaults(
	defineProps<{
		count: number
		itemType: string
		variant?: 'instance' | 'server'
		backupTip?: string
		actionDisabled?: boolean
		actionDisabledTooltip?: string
		targetType?: 'server' | 'instance'
	}>(),
	{
		variant: 'instance',
		backupTip: undefined,
		actionDisabled: false,
		actionDisabledTooltip: undefined,
		targetType: undefined,
	},
)

const emit = defineEmits<{
	(e: 'delete'): void
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const backupCreator = ref<InstanceType<typeof InlineBackupCreator>>()
const buttonsDisabled = ref(false)
const visibleCount = ref(props.count)
const visibleItemType = ref(props.itemType)

async function show() {
	await nextTick()
	visibleCount.value = props.count
	visibleItemType.value = props.itemType
	modal.value?.show()
}

function confirm() {
	if (props.actionDisabled) return
	modal.value?.hide()
	emit('delete')
}

defineExpose({
	show,
})
</script>
