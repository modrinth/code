<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.leavePageTitle)"
		fade="warning"
		max-width="500px"
	>
		<div class="flex flex-col gap-6">
			<Admonition type="critical" :header="formatMessage(messages.uploadInProgress)">
				{{ formatMessage(messages.leavePageBody) }}
			</Admonition>
		</div>

		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button class="!border !border-surface-4" @click="cancel">
						<XIcon />
						{{ formatMessage(messages.stayOnPageButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="red">
					<button @click="leave">
						<RightArrowIcon />
						{{ formatMessage(messages.leavePageButton) }}
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
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	leavePageTitle: {
		id: 'instances.confirm-leave-modal.title',
		defaultMessage: 'Leave page?',
	},
	uploadInProgress: {
		id: 'instances.confirm-leave-modal.upload-in-progress',
		defaultMessage: 'Upload in progress',
	},
	leavePageBody: {
		id: 'instances.confirm-leave-modal.body',
		defaultMessage:
			'Files are still being uploaded. Leaving this page will cancel the upload and your changes may be lost.',
	},
	stayOnPageButton: {
		id: 'instances.confirm-leave-modal.stay',
		defaultMessage: 'Stay on page',
	},
	leavePageButton: {
		id: 'instances.confirm-leave-modal.leave',
		defaultMessage: 'Leave page',
	},
})

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
