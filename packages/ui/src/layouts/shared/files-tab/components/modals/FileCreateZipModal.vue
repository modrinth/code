<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" max-width="500px">
		<form class="space-y-6 md:min-w-[400px]" @submit.prevent="handleSubmit">
			<label class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.nameLabel) }}</span>
				<Input
					ref="targetInput"
					v-model="target"
					placeholder="archive.zip"
					wrapper-class="w-full"
				/>
				<div v-if="submitted && error" class="text-sm text-red">{{ error }}</div>
			</label>
		</form>
		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="outlined" @click="hide">
					<XIcon class="h-5 w-5" /> {{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button type="colored" color="brand" :disabled="!!error && submitted" @click="handleSubmit">
					<FolderArchiveIcon class="h-5 w-5" /> {{ formatMessage(messages.createButton) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { FolderArchiveIcon, XIcon } from '@modrinth/assets'
import { computed, nextTick, ref } from 'vue'

import { Button } from '#ui/components/base/buttons'
import Input from '#ui/components/base/inputs/Input.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

const { formatMessage } = useVIntl()
const messages = defineMessages({
	header: { id: 'files.create-zip-modal.header', defaultMessage: 'Create ZIP' },
	nameLabel: { id: 'files.create-zip-modal.name-label', defaultMessage: 'Archive name' },
	createButton: { id: 'files.create-zip-modal.create-button', defaultMessage: 'Create ZIP' },
	required: { id: 'files.create-zip-modal.required', defaultMessage: 'Enter an archive name.' },
	invalid: {
		id: 'files.create-zip-modal.invalid',
		defaultMessage: 'The archive name must be a single file name.',
	},
})
const emit = defineEmits<{ create: [target: string] }>()
const modal = ref<InstanceType<typeof NewModal>>()
const targetInput = ref<HTMLInputElement | null>(null)
const target = ref('archive.zip')
const submitted = ref(false)
const error = computed(() => {
	if (!target.value) return formatMessage(messages.required)
	if (target.value === '.' || target.value === '..' || /[\\/]/.test(target.value)) {
		return formatMessage(messages.invalid)
	}
	return ''
})
function handleSubmit() {
	submitted.value = true
	if (error.value) return
	emit('create', target.value)
	hide()
}
function show() {
	target.value = 'archive.zip'
	submitted.value = false
	modal.value?.show()
	nextTick(() => targetInput.value?.focus())
}
function hide() {
	modal.value?.hide()
}
defineExpose({ show, hide })
</script>
