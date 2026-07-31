<template>
	<NewModal ref="modal" fade="warning" :header="formatMessage(messages.header)" max-width="500px">
		<p class="m-0 text-secondary">
			{{ formatMessage(messages.body) }}
		</p>
		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="outlined" @click="handleCancel">
					<XIcon />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button type="colored" color="red" @click="handleDiscard">
					<TrashIcon />
					{{ formatMessage(messages.discard) }}
				</Button>
				<Button type="colored" color="green" @click="handleSave">
					<SaveIcon />
					{{ formatMessage(commonMessages.saveButton) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { SaveIcon, TrashIcon, XIcon } from '@modrinth/assets'
import { ref } from 'vue'

import { Button } from '#ui/components/base/buttons'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'files.unsaved-changes-modal.header',
		defaultMessage: 'Unsaved changes',
	},
	body: {
		id: 'files.unsaved-changes-modal.body',
		defaultMessage:
			'You have unsaved changes that will be lost if you leave. Would you like to save before leaving?',
	},
	discard: {
		id: 'files.unsaved-changes-modal.discard',
		defaultMessage: 'Discard',
	},
})

export type UnsavedChangesResult = 'cancel' | 'discard' | 'save'

const modal = ref<InstanceType<typeof NewModal>>()
let resolvePromise: ((value: UnsavedChangesResult) => void) | null = null

function prompt(): Promise<UnsavedChangesResult> {
	return new Promise((resolve) => {
		resolvePromise = resolve
		modal.value?.show()
	})
}

function resolve(result: UnsavedChangesResult) {
	modal.value?.hide()
	resolvePromise?.(result)
	resolvePromise = null
}

function handleCancel() {
	resolve('cancel')
}

function handleDiscard() {
	resolve('discard')
}

function handleSave() {
	resolve('save')
}

defineExpose({ prompt })
</script>
