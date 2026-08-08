<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.title)"
		:closable="false"
		fade="warning"
		max-width="544px"
	>
		<div class="flex flex-col">
			<p class="mb-4 mt-0 leading-normal">
				{{ formatMessage(messages.body) }}
			</p>
			<div class="flex justify-end">
				<Button type="colored" color="brand" native-type="button" @click="hide">
					<CheckCircleIcon class="size-4" />
					{{ formatMessage(commonMessages.iUnderstandButton) }}
				</Button>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import { CheckCircleIcon } from '@modrinth/assets'
import { Button, commonMessages, defineMessages, NewModal, useVIntl } from '@modrinth/ui'
import { useTemplateRef } from 'vue'

const { formatMessage } = useVIntl()
const modal = useTemplateRef('modal')

const messages = defineMessages({
	title: {
		id: 'project.ai-image-warning-modal.title',
		defaultMessage: 'AI-generated images not allowed',
	},
	body: {
		id: 'project.ai-image-warning-modal.body',
		defaultMessage:
			'Using AI-generated images to represent your project is not allowed. Attempting to work around detection may lead to your Modrinth account being suspended.',
	},
})

function show() {
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

defineExpose({ show, hide })
</script>
