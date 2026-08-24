<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.title)"
		:on-hide="handleHide"
		:on-after-hide="handleAfterHide"
		:closable="false"
		fade="warning"
		max-width="544px"
	>
		<div class="flex flex-col">
			<p class="mt-0 mb-4 leading-normal">
				{{ formatMessage(messages.body1) }}
			</p>
			<p class="mt-0 mb-4 leading-normal">
				{{ formatMessage(messages.body2) }}
			</p>
			<p class="mt-0 mb-4 leading-normal">
				{{ formatMessage(messages.body3) }}
			</p>

			<div class="flex justify-between items-center">
				<Checkbox v-model="dontShowAgain" :label="formatMessage(messages.dontShowAgain)" />
				<Button type="colored" color="brand" native-type="button" @click="acknowledge">
					<CheckCircleIcon class="size-4" />
					{{ formatMessage(commonMessages.iUnderstandButton) }}
				</Button>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import { CheckCircleIcon } from '@modrinth/assets'
import { ref, useTemplateRef } from 'vue'

import { Button } from '#ui/components/base/buttons'

import { defineMessages, useVIntl } from '../../composables/i18n'
import { commonMessages } from '../../utils/common-messages'
import Checkbox from '../base/Checkbox.vue'
import NewModal from './NewModal.vue'

const emit = defineEmits<{
	dismiss: [dontShowAgain: boolean]
}>()

const { formatMessage } = useVIntl()
const modal = useTemplateRef('modal')
const dontShowAgain = ref(false)
let pendingDontShowAgain = false

const messages = defineMessages({
	title: {
		id: 'search.photosensitivity-warning-modal.title',
		defaultMessage: 'Results are not guaranteed to be safe',
	},
	body1: {
		id: 'search.photosensitivity-warning-modal.body.1',
		defaultMessage:
			'We cannot guarantee that all content on Modrinth has been labeled appropriately.',
	},
	body2: {
		id: 'search.photosensitivity-warning-modal.body.2',
		defaultMessage:
			'Content labels for photosensitivity triggers are self-assigned by the creators who upload their content to Modrinth. These projects have not gone through any safety testing.',
	},
	body3: {
		id: 'search.photosensitivity-warning-modal.body.3',
		defaultMessage: 'Using any content on Modrinth is at your own risk. Please stay safe! 💚',
	},
	dontShowAgain: {
		id: 'search.photosensitivity-warning-modal.dont-show-again',
		defaultMessage: "Don't show this again",
	},
})

function show() {
	dontShowAgain.value = false
	pendingDontShowAgain = false
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

function handleHide() {
	pendingDontShowAgain = dontShowAgain.value
}

function handleAfterHide() {
	emit('dismiss', pendingDontShowAgain)
	dontShowAgain.value = false
	pendingDontShowAgain = false
}

function acknowledge() {
	pendingDontShowAgain = dontShowAgain.value
	modal.value?.hide()
}

defineExpose({ show, hide })
</script>
