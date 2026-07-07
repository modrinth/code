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
	>
		<div class="flex flex-col gap-6">
			<Admonition type="warning" :header="formatMessage(messages.admonitionHeader)">
				{{
					visibleCount === 1 ? formatMessage(messages.singleBody) : formatMessage(messages.bulkBody)
				}}
			</Admonition>
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
						:disabled="props.actionDisabled"
						@click="confirm"
					>
						<SlashIcon />
						{{
							visibleCount === 1
								? formatMessage(messages.confirmButton)
								: formatMessage(messages.confirmManyButton, {
										count: visibleCount,
										itemType: formatContentTypeSentence(
											formatMessage,
											visibleItemType,
											visibleCount,
										),
									})
						}}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { SlashIcon, XIcon } from '@modrinth/assets'
import { nextTick, ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages, formatContentTypeSentence } from '#ui/utils/common-messages'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'content.confirm-disable.header',
		defaultMessage: 'Disable {itemType}',
	},
	admonitionHeader: {
		id: 'content.confirm-disable.shared-instance.admonition-header',
		defaultMessage: 'This is part of the shared instance',
	},
	singleBody: {
		id: 'content.confirm-disable.shared-instance.single-body',
		defaultMessage:
			'Disabling it only changes your local copy. Future shared instance updates may re-enable, restore, or change it again.',
	},
	bulkBody: {
		id: 'content.confirm-disable.shared-instance.bulk-body',
		defaultMessage:
			'Some selected projects are part of the shared instance. Disabling them only changes your local copy, and future shared instance updates may re-enable, restore, or change them again.',
	},
	confirmButton: {
		id: 'content.confirm-disable.shared-instance.confirm-button',
		defaultMessage: 'Disable anyway',
	},
	confirmManyButton: {
		id: 'content.confirm-disable.shared-instance.confirm-many-button',
		defaultMessage: 'Disable {count, number} {itemType} anyway',
	},
})

const props = withDefaults(
	defineProps<{
		count: number
		itemType: string
		actionDisabled?: boolean
		actionDisabledTooltip?: string
	}>(),
	{
		actionDisabled: false,
		actionDisabledTooltip: undefined,
	},
)

const emit = defineEmits<{
	(e: 'disable'): void
}>()

const modal = ref<InstanceType<typeof NewModal>>()
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
	emit('disable')
}

defineExpose({
	show,
})
</script>
