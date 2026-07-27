<script setup lang="ts">
import { EditIcon } from '@modrinth/assets'
import { computed, nextTick, onUnmounted, ref, watch } from 'vue'

const model = defineModel<string>({ default: '' })

const props = withDefaults(
	defineProps<{
		placeholder?: string
		defaultValue?: string
		maxWidth?: string
		maxLength?: number
		editLabel?: string
		activationMode?: 'text' | 'icon'
		validate?: (value: string) => boolean
		onChange?: (value: string) => boolean | void | Promise<boolean | void>
	}>(),
	{
		placeholder: '',
		defaultValue: '',
		maxWidth: '100%',
		maxLength: undefined,
		editLabel: 'Edit',
		activationMode: 'text',
		validate: undefined,
		onChange: undefined,
	},
)

const isEditing = ref(false)
const isSaving = ref(false)
const isInvalid = ref(false)
const input = ref<HTMLInputElement>()
const isEditIconVisible = ref(false)
const draft = ref(model.value)
const originalValue = ref('')
let editIconHideTimeout: ReturnType<typeof setTimeout> | undefined

const displayValue = computed(() => model.value || props.defaultValue || props.placeholder)
const sizingValue = computed(() => {
	if (isEditing.value) {
		return draft.value || props.placeholder || ' '
	}

	return displayValue.value || ' '
})

watch(model, (value) => {
	if (!isEditing.value) {
		draft.value = value
	}
})

function clearEditIconHideTimeout() {
	if (editIconHideTimeout) {
		clearTimeout(editIconHideTimeout)
		editIconHideTimeout = undefined
	}
}

function showEditIcon() {
	isEditIconVisible.value = true
}

function keepEditIconVisible() {
	clearEditIconHideTimeout()
	isEditIconVisible.value = true
}

function hideEditIcon() {
	if (!editIconHideTimeout) {
		isEditIconVisible.value = false
	}
}

function handleEditIconMouseleave() {
	clearEditIconHideTimeout()
	editIconHideTimeout = setTimeout(() => {
		isEditIconVisible.value = false
		editIconHideTimeout = undefined
	}, 1000)
}

async function startEditing() {
	if (isEditing.value) return

	clearEditIconHideTimeout()
	originalValue.value = model.value
	draft.value = model.value
	isInvalid.value = false
	isEditing.value = true
	await nextTick()
	input.value?.focus()
	input.value?.select()
}

async function applyValue() {
	if (!isEditing.value || isSaving.value) return

	const nextValue = (draft.value || props.defaultValue).trim()

	if (nextValue === originalValue.value) {
		model.value = nextValue
		isEditing.value = false
		isInvalid.value = false
		return
	}

	if (props.validate && !props.validate(nextValue)) {
		isInvalid.value = true
		await nextTick()
		input.value?.focus()
		return
	}

	isSaving.value = true
	isInvalid.value = false

	try {
		const accepted = await props.onChange?.(nextValue)
		if (accepted === false) {
			isInvalid.value = true
			isSaving.value = false
			await nextTick()
			input.value?.focus()
			return
		}

		model.value = nextValue
		isEditing.value = false
	} catch {
		isInvalid.value = true
		isSaving.value = false
		await nextTick()
		input.value?.focus()
	} finally {
		isSaving.value = false
	}
}

function cancelEditing() {
	if (isSaving.value) return

	draft.value = originalValue.value
	isInvalid.value = false
	isEditing.value = false
}

function handleKeydown(event: KeyboardEvent) {
	if (event.key === 'Enter') {
		event.preventDefault()
		void applyValue()
		return
	}

	if (event.key === 'Escape') {
		event.preventDefault()
		event.stopPropagation()
		cancelEditing()
	}
}

onUnmounted(clearEditIconHideTimeout)

defineExpose({
	isEditing,
	startEditing,
})
</script>

<template>
	<div
		:data-value="sizingValue"
		class="relative flex h-6 min-w-3 min-h-0 max-w-full flex-col justify-center border-b font-medium"
		:class="
			isEditing
				? [
						'after:invisible after:block after:w-full after:whitespace-pre after:content-[attr(data-value)]',
						isInvalid ? 'border-red' : 'border-contrast',
					]
				: 'border-transparent'
		"
		:style="{ maxWidth }"
	>
		<input
			v-if="isEditing"
			ref="input"
			v-model="draft"
			type="text"
			:aria-invalid="isInvalid"
			:aria-label="editLabel"
			:disabled="isSaving"
			:maxlength="maxLength"
			:placeholder="placeholder"
			class="absolute inset-0 top-px w-full !h-full !min-h-0 min-w-0 truncate bg-transparent !p-0 text-inherit !border-b-2 border-0 !border-brand !border-solid !shadow-none [font:inherit] !outline-none"
			@blur="applyValue"
			@click.stop
			@keydown="handleKeydown"
		/>
		<div
			v-else-if="activationMode === 'icon'"
			class="group/edit-icon flex min-w-0 max-w-full items-center"
			@mouseenter="showEditIcon"
			@mouseleave="hideEditIcon"
		>
			<span class="min-w-0 truncate select-text" :title="model || displayValue">
				{{ displayValue }}
			</span>
			<span
				class="flex shrink-0 overflow-hidden transition-all duration-150"
				:class="
					isEditIconVisible
						? 'max-w-6 translate-x-0 opacity-100'
						: 'max-w-0 translate-x-1 opacity-0'
				"
				@mouseenter="keepEditIconVisible"
				@mouseleave="handleEditIconMouseleave"
			>
				<button
					type="button"
					class="ml-1 flex size-5 cursor-pointer items-center justify-center border-0 bg-transparent p-0 text-secondary transition-colors hover:text-brand focus-visible:text-contrast"
					:aria-label="`${editLabel}: ${displayValue}`"
					@click.stop="startEditing"
				>
					<EditIcon class="size-5" aria-hidden="true" />
				</button>
			</span>
		</div>
		<button
			v-else
			type="button"
			class="flex w-full max-w-full items-center gap-2 truncate border-0 bg-transparent p-0 text-left text-inherit transition-colors hover:text-brand focus-visible:text-contrast [font:inherit]"
			:aria-label="`${editLabel}: ${displayValue}`"
			:title="model || displayValue"
			@click="startEditing"
		>
			<span class="min-w-0 truncate">{{ displayValue }}</span>
		</button>
	</div>
</template>
