<template>
	<NewModal ref="modal" :header="formatMessage(messages.header, { type })" max-width="500px">
		<form class="space-y-6 md:min-w-[400px]" @submit.prevent="handleSubmit">
			<label class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{
					formatMessage(fileValidationMessages.nameLabel)
				}}</span>
				<StyledInput
					ref="createInput"
					v-model="itemName"
					:placeholder="
						formatMessage(
							type === 'file' ? messages.placeholderFile : messages.placeholderDirectory,
						)
					"
					wrapper-class="w-full"
				/>
				<div v-if="submitted && error" class="text-sm text-red">{{ error }}</div>
			</label>
		</form>
		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button class="!border !border-surface-4" @click="hide">
						<XIcon class="h-5 w-5" />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button :disabled="!!error && submitted" @click="handleSubmit">
						<PlusIcon class="h-5 w-5" />
						{{ formatMessage(messages.createButton, { type }) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { PlusIcon, XIcon } from '@modrinth/assets'
import { computed, nextTick, ref } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import { fileValidationMessages } from './file-validation-messages'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'files.create-modal.header',
		defaultMessage: 'Create a {type, select, directory {folder} other {file}}',
	},
	placeholderFile: {
		id: 'files.create-modal.placeholder-file',
		defaultMessage: 'e.g. config.yml',
	},
	placeholderDirectory: {
		id: 'files.create-modal.placeholder-directory',
		defaultMessage: 'e.g. plugins',
	},
	createButton: {
		id: 'files.create-modal.create-button',
		defaultMessage: 'Create {type, select, directory {folder} other {file}}',
	},
})

const props = defineProps<{
	type: 'file' | 'directory'
}>()

const emit = defineEmits<{
	create: [name: string]
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const createInput = ref<HTMLInputElement | null>(null)
const itemName = ref('')
const submitted = ref(false)

const error = computed(() => {
	if (!itemName.value) {
		return formatMessage(fileValidationMessages.nameRequired)
	}
	if (props.type === 'file') {
		const validPattern = /^[a-zA-Z0-9-_.\s]+$/
		if (!validPattern.test(itemName.value)) {
			return formatMessage(fileValidationMessages.nameInvalidFile)
		}
	} else {
		const validPattern = /^[a-zA-Z0-9-_\s]+$/
		if (!validPattern.test(itemName.value)) {
			return formatMessage(fileValidationMessages.nameInvalidDirectory)
		}
	}
	return ''
})

const handleSubmit = () => {
	submitted.value = true
	if (!error.value) {
		emit('create', itemName.value)
		hide()
	}
}

const show = () => {
	itemName.value = ''
	submitted.value = false
	modal.value?.show()
	nextTick(() => {
		setTimeout(() => {
			createInput.value?.focus()
		}, 100)
	})
}

const hide = () => {
	modal.value?.hide()
}

defineExpose({ show, hide })
</script>
