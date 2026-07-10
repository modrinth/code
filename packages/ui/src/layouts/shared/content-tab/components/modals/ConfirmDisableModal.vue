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
			<Admonition type="warning" :header="visibleWarning?.admonitionHeader ?? ''">
				{{ visibleWarning?.admonitionBody }}
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
						{{ visibleWarning?.actionLabel }}
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

import type { ContentActionWarning } from '../../types'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'content.confirm-disable.header',
		defaultMessage: 'Disable {itemType}',
	},
})

const props = withDefaults(
	defineProps<{
		count: number
		itemType: string
		warning?: ContentActionWarning | null
		actionDisabled?: boolean
		actionDisabledTooltip?: string
	}>(),
	{
		actionDisabled: false,
		actionDisabledTooltip: undefined,
		warning: null,
	},
)

const emit = defineEmits<{
	(e: 'disable'): void
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const visibleCount = ref(props.count)
const visibleItemType = ref(props.itemType)
const visibleWarning = ref(props.warning)

async function show() {
	await nextTick()
	visibleCount.value = props.count
	visibleItemType.value = props.itemType
	visibleWarning.value = props.warning
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
