<template>
	<NewModal ref="modal" max-width="450px">
		<template #title>
			<span class="text-lg font-extrabold text-contrast">
				{{ title }}
			</span>
		</template>
		<div class="flex flex-col gap-4">
			<p class="m-0 text-secondary">
				{{ description }}
			</p>
			<StyledInput
				v-model="folderName"
				type="text"
				:placeholder="formatMessage(messages.folderNamePlaceholder)"
				autofocus
				autocomplete="off"
				@keydown.enter="handleConfirm"
			/>
			<p v-if="nameError" class="m-0 text-sm text-red">
				{{ nameError }}
			</p>
			<div class="flex justify-end gap-2">
				<ButtonStyled>
					<button @click="hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button :disabled="!folderName.trim() || !!nameError" @click="handleConfirm">
						<CheckIcon />
						{{ confirmLabel }}
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import { CheckIcon, XIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	commonMessages,
	defineMessages,
	NewModal,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	folderNamePlaceholder: {
		id: 'app.instance.mods.folder-name-placeholder',
		defaultMessage: 'Folder name...',
	},
	nameTakenError: {
		id: 'app.instance.mods.folder-name-taken',
		defaultMessage: 'A folder with this name already exists.',
	},
})

const props = withDefaults(
	defineProps<{
		title?: string
		description?: string
		confirmLabel?: string
		existingNames?: string[]
		excludeName?: string
	}>(),
	{
		title: 'New folder',
		description: 'Create a folder to organize your mods.',
		confirmLabel: 'Create',
		existingNames: () => [],
		excludeName: '',
	},
)

const emit = defineEmits<{
	confirm: [name: string]
}>()

const modal = ref<InstanceType<typeof NewModal> | null>(null)
const folderName = ref('')

const nameError = computed(() => {
	const name = folderName.value.trim()
	if (!name) return ''
	if (name === props.excludeName) return ''
	if (props.existingNames.some((n) => n === name)) {
		return formatMessage(messages.nameTakenError)
	}
	return ''
})

function show(defaultName?: string) {
	folderName.value = defaultName ?? ''
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
	folderName.value = ''
}

function handleConfirm() {
	const name = folderName.value.trim()
	if (!name || nameError.value) return
	emit('confirm', name)
	hide()
}

defineExpose({ show, hide })
</script>
