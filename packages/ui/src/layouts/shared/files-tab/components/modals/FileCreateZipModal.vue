<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" max-width="500px" no-padding>
		<form class="px-6 pt-6 md:min-w-[400px]" @submit.prevent="handleSubmit">
			<label class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.nameLabel) }}</span>
				<Input
					ref="targetInput"
					v-model="target"
					placeholder="archive.zip"
					wrapper-class="w-full"
				/>
				<div class="min-h-5 text-sm text-red" aria-live="polite">{{ displayedError }}</div>
			</label>
		</form>
		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="outlined" @click="hide">
					<XIcon class="h-5 w-5" /> {{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button type="colored" color="brand" :disabled="!!displayedError" @click="handleSubmit">
					<FolderArchiveIcon class="h-5 w-5" /> {{ formatMessage(messages.createButton) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { type Kyros, ModrinthApiError } from '@modrinth/api-client'
import { FolderArchiveIcon, XIcon } from '@modrinth/assets'
import { computed, nextTick, ref, watch } from 'vue'

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
	alreadyExists: {
		id: 'files.create-zip-modal.already-exists',
		defaultMessage: 'A file with this name already exists.',
	},
	checkFailed: {
		id: 'files.create-zip-modal.check-failed',
		defaultMessage: 'Could not check whether this file already exists.',
	},
})
const props = defineProps<{
	parent: string
	statFile?: (path: string) => Promise<Kyros.Files.v1.FileStatResponse>
}>()
const emit = defineEmits<{ create: [target: string] }>()
const modal = ref<InstanceType<typeof NewModal>>()
const targetInput = ref<HTMLInputElement | null>(null)
const target = ref('archive.zip')
const submitted = ref(false)
const availabilityError = ref('')
let checkGeneration = 0
let submitting = false

const normalizedTarget = computed(() =>
	target.value.toLowerCase().endsWith('.zip') ? target.value : `${target.value}.zip`,
)
const localError = computed(() => {
	if (!target.value) return formatMessage(messages.required)
	if (target.value === '.' || target.value === '..' || /[\\/]/.test(target.value)) {
		return formatMessage(messages.invalid)
	}
	return ''
})
const displayedError = computed(
	() => (submitted.value ? localError.value : '') || availabilityError.value,
)

function getTargetPath(fileName: string) {
	return `${props.parent}/${fileName}`.replace('//', '/')
}

async function fileExists(fileName: string) {
	if (!props.statFile) throw new Error('File stat is unavailable')
	try {
		await props.statFile(getTargetPath(fileName))
		return true
	} catch (error) {
		if (error instanceof ModrinthApiError && error.statusCode === 404) return false
		throw error
	}
}

function checkAvailability() {
	const generation = ++checkGeneration
	availabilityError.value = ''
	if (localError.value) return

	const fileName = normalizedTarget.value
	void fileExists(fileName)
		.then((exists) => {
			if (generation !== checkGeneration || fileName !== normalizedTarget.value) return
			availabilityError.value = exists ? formatMessage(messages.alreadyExists) : ''
		})
		.catch(() => {
			if (generation !== checkGeneration || fileName !== normalizedTarget.value) return
			availabilityError.value = formatMessage(messages.checkFailed)
		})
}

async function handleSubmit() {
	submitted.value = true
	availabilityError.value = ''
	if (localError.value || submitting) return

	const generation = ++checkGeneration
	const fileName = normalizedTarget.value
	submitting = true
	try {
		const exists = await fileExists(fileName)
		if (generation !== checkGeneration || fileName !== normalizedTarget.value) return
		if (exists) {
			availabilityError.value = formatMessage(messages.alreadyExists)
			return
		}
		emit('create', fileName)
		hide()
	} catch {
		if (generation !== checkGeneration || fileName !== normalizedTarget.value) return
		availabilityError.value = formatMessage(messages.checkFailed)
	} finally {
		submitting = false
	}
}
function show() {
	const targetUnchanged = target.value === 'archive.zip'
	target.value = 'archive.zip'
	submitted.value = false
	availabilityError.value = ''
	submitting = false
	modal.value?.show()
	if (targetUnchanged) checkAvailability()
	nextTick(() => targetInput.value?.focus())
}
function hide() {
	checkGeneration++
	availabilityError.value = ''
	modal.value?.hide()
}

watch(target, checkAvailability)

defineExpose({ show, hide })
</script>
