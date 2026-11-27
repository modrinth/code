<template>
	<NewModal
		ref="modal"
		:header="localizeIfPossible(title)"
		fade="warning"
		max-width="500px"
		:on-hide="cancel"
	>
		<div class="flex flex-col gap-6">
			<Admonition :type="admonitionType" :header="localizeIfPossible(header)">
				{{ localizeIfPossible(body) }}
			</Admonition>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="outlined" @click="modal?.hide()">
					<XIcon />
					{{ localizeIfPossible(stayLabel) }}
				</Button>
				<Button type="colored" color="red" @click="leave">
					<RightArrowIcon />
					{{ localizeIfPossible(leaveLabel) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { RightArrowIcon, XIcon } from '@modrinth/assets'
import { onBeforeUnmount, ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import { Button } from '#ui/components/base/buttons'
import { defineMessage, type MessageDescriptor, useVIntl } from '#ui/composables/i18n'

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
				defaultMessage: 'You have unsaved changes that will be lost if you leave this page.',
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
let pendingPrompt: Promise<boolean> | undefined
let resolvePrompt: ((confirmed: boolean) => void) | undefined

function prompt(): Promise<boolean> {
	if (pendingPrompt) return pendingPrompt
	const confirmationModal = modal.value
	if (!confirmationModal) return Promise.resolve(false)

	const promise = new Promise<boolean>((resolve) => {
		resolvePrompt = resolve
	})
	pendingPrompt = promise
	confirmationModal.show()
	return promise
}

function leave() {
	finish(true)
	modal.value?.hide()
}

function cancel() {
	finish(false)
}

function finish(confirmed: boolean) {
	const resolve = resolvePrompt
	resolvePrompt = undefined
	pendingPrompt = undefined
	resolve?.(confirmed)
}

onBeforeUnmount(cancel)
defineExpose({ prompt })
</script>
