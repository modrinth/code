<template>
	<NewModal
		ref="modal"
		:header="localizeIfPossible(title)"
		fade="warning"
		max-width="500px"
	>
		<div class="flex flex-col gap-6">
			<Admonition :type="admonitionType" :header="localizeIfPossible(header)">
				{{ localizeIfPossible(body) }}
			</Admonition>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled type="outlined">
					<button class="!border !border-surface-4" @click="cancel">
						<XIcon />
						{{ localizeIfPossible(stayLabel) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="red">
					<button @click="leave">
						<RightArrowIcon />
						{{ localizeIfPossible(leaveLabel) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { RightArrowIcon, XIcon } from '@modrinth/assets'
import { ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import {
	defineMessage,
	type MessageDescriptor,
	useVIntl,
} from '#ui/composables/i18n'

import NewModal from './NewModal.vue'

const { formatMessage } = useVIntl()

withDefaults(
	defineProps<{
		title?: MessageDescriptor | string
		header?: MessageDescriptor | string
		body?: MessageDescriptor | string
		stayLabel?: MessageDescriptor | string
		leaveLabel?: MessageDescriptor | string
		admonitionType?: 'warning' | 'critical' | 'info'
	}>(),
	{
		title: () =>
			defineMessage({
				id: 'ui.confirm-leave-modal.title',
				defaultMessage: 'Leave page?',
			}),
		header: () =>
			defineMessage({
				id: 'ui.confirm-leave-modal.header',
				defaultMessage: 'You have unsaved changes',
			}),
		body: () =>
			defineMessage({
				id: 'ui.confirm-leave-modal.body',
				defaultMessage:
					'You have unsaved changes that will be lost if you leave this page.',
			}),
		stayLabel: () =>
			defineMessage({
				id: 'ui.confirm-leave-modal.stay',
				defaultMessage: 'Stay on page',
			}),
		leaveLabel: () =>
			defineMessage({
				id: 'ui.confirm-leave-modal.leave',
				defaultMessage: 'Leave page',
			}),
		admonitionType: 'critical',
	},
)

function localizeIfPossible(message: MessageDescriptor | string) {
	return typeof message === 'string' ? message : formatMessage(message)
}

const modal = ref<InstanceType<typeof NewModal>>()
let resolvePromise: ((value: boolean) => void) | null = null

function prompt(): Promise<boolean> {
	return new Promise((resolve) => {
		resolvePromise = resolve
		modal.value?.show()
	})
}

function leave() {
	modal.value?.hide()
	resolvePromise?.(true)
	resolvePromise = null
}

function cancel() {
	modal.value?.hide()
	resolvePromise?.(false)
	resolvePromise = null
}

defineExpose({ prompt })
</script>
