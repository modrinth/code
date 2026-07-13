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
			<div v-if="showColor" class="flex flex-col gap-2">
				<span class="text-sm font-semibold text-secondary">{{ formatMessage(messages.colorLabel) }}</span>
				<div class="flex flex-wrap gap-2">
					<button
						class="flex h-7 w-7 items-center justify-center rounded-full border-2 transition-all"
						:class="
							!selectedColor
								? 'border-brand scale-110'
								: 'border-surface-5 hover:border-surface-4'
						"
						:title="formatMessage(messages.noColor)"
						@click="selectedColor = undefined"
					>
						<span class="block h-4 w-4 rounded-full bg-surface-5" />
						<span class="absolute h-0.5 w-5 rotate-45 bg-red" />
					</button>
					<button
						v-for="color in FOLDER_COLORS"
						:key="color"
						class="flex h-7 w-7 items-center justify-center rounded-full border-2 transition-all"
						:class="
							selectedColor === color
								? 'border-brand scale-110'
								: 'border-transparent hover:scale-105'
						"
						:style="{ backgroundColor: color }"
						:title="color"
						@click="selectedColor = color"
					>
						<CheckIcon v-if="selectedColor === color" class="size-4 text-white" />
					</button>
				</div>
			</div>
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

import { FOLDER_COLORS } from '@/helpers/mod-folders'

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
	colorLabel: {
		id: 'app.instance.mods.folder-color-label',
		defaultMessage: 'Color',
	},
	noColor: {
		id: 'app.instance.mods.folder-no-color',
		defaultMessage: 'No color',
	},
})

const props = withDefaults(
	defineProps<{
		title?: string
		description?: string
		confirmLabel?: string
		existingNames?: string[]
		excludeName?: string
		showColor?: boolean
	}>(),
	{
		title: 'New folder',
		description: 'Create a folder to organize your mods.',
		confirmLabel: 'Create',
		existingNames: () => [],
		excludeName: '',
		showColor: true,
	},
)

const emit = defineEmits<{
	confirm: [name: string, color?: string]
}>()

const modal = ref<InstanceType<typeof NewModal> | null>(null)
const folderName = ref('')
const selectedColor = ref<string | undefined>()

const nameError = computed(() => {
	const name = folderName.value.trim()
	if (!name) return ''
	if (name === props.excludeName) return ''
	if (props.existingNames.some((n) => n === name)) {
		return formatMessage(messages.nameTakenError)
	}
	return ''
})

function show(defaultName?: string, defaultColor?: string) {
	folderName.value = defaultName ?? ''
	selectedColor.value = defaultColor
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
	folderName.value = ''
	selectedColor.value = undefined
}

function handleConfirm() {
	const name = folderName.value.trim()
	if (!name || nameError.value) return
	try {
		emit('confirm', name, selectedColor.value)
	} finally {
		hide()
	}
}

defineExpose({ show, hide })
</script>
