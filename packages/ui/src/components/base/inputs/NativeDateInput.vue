<template>
	<InputFrame
		:appearance="appearance"
		:class="[wrapperClass, attrs.class]"
		:disabled="disabled"
		:invalid="error"
		:size="size"
		:style="attrs.style"
	>
		<input
			v-bind="mergeProps(controlAttrs(), inputAttrs ?? {})"
			:id="id"
			ref="inputRef"
			:type="type"
			:value="model"
			:placeholder="placeholder"
			:disabled="disabled"
			:readonly="readonly"
			:name="name"
			:autocomplete="autocomplete"
			:min="min"
			:max="max"
			:step="step"
			:aria-invalid="error || undefined"
			class="min-w-0 w-full flex-1 self-stretch appearance-none border-0 bg-transparent p-0 font-medium text-primary shadow-none outline-none placeholder:text-secondary focus:text-contrast focus:ring-0 [&::-webkit-calendar-picker-indicator]:hidden"
			:class="[
				inputClass,
				controlClass,
				size === 'small' ? 'text-sm' : 'text-base',
				disabled ? 'cursor-not-allowed' : '',
			]"
			@input="onInput"
		/>

		<template #trailing>
			<span class="flex shrink-0 items-center gap-2">
				<InputClearButton
					v-if="showClearButton"
					:label="clearLabel"
					@click="clear"
				/>
				<button
					type="button"
					class="relative flex size-5 shrink-0 touch-manipulation cursor-pointer select-none items-center justify-center border-none bg-transparent p-0 text-secondary opacity-60 transition-colors before:absolute before:-inset-2 hover:text-contrast hover:opacity-100 focus-visible:text-contrast focus-visible:opacity-100 group-focus-within/input:text-contrast group-focus-within/input:opacity-100 disabled:pointer-events-none"
					:aria-label="pickerLabel"
					:disabled="disabled || readonly"
					@click="openPicker"
				>
					<CalendarIcon class="size-5" aria-hidden="true" />
				</button>
			</span>
		</template>
	</InputFrame>
</template>

<script setup lang="ts">
import { CalendarIcon } from '@modrinth/assets'
import { computed, type InputHTMLAttributes, mergeProps, ref, useAttrs } from 'vue'

import InputClearButton from './InputClearButton.vue'
import InputFrame from './InputFrame.vue'
import type { InputAppearance, InputSize } from './types'

defineOptions({ inheritAttrs: false })

const model = defineModel<string | undefined>()
const attrs = useAttrs()

const props = withDefaults(
	defineProps<{
		appearance?: InputAppearance
		autocomplete?: string
		clearLabel?: string
		clearable?: boolean
		controlClass?: string
		disabled?: boolean
		error?: boolean
		id?: string
		inputAttrs?: InputHTMLAttributes
		inputClass?: string
		max?: string
		min?: string
		name?: string
		pickerLabel?: string
		placeholder?: string
		readonly?: boolean
		size?: InputSize
		step?: number
		type?: 'date' | 'datetime-local'
		wrapperClass?: string
	}>(),
	{
		appearance: 'surface',
		clearLabel: 'Clear date',
		clearable: false,
		disabled: false,
		error: false,
		pickerLabel: 'Open date picker',
		readonly: false,
		size: 'standard',
		type: 'date',
	},
)

const emit = defineEmits<{
	clear: []
	input: [event: Event]
}>()

const inputRef = ref<HTMLInputElement>()
const hasValue = computed(() => model.value !== undefined && model.value !== '')
const showClearButton = computed(
	() => props.clearable && hasValue.value && !props.disabled && !props.readonly,
)

defineExpose({ focus: () => inputRef.value?.focus() })

function controlAttrs() {
	const { class: _class, style: _style, ...rest } = attrs
	return rest
}

function onInput(event: Event) {
	model.value = (event.target as HTMLInputElement).value
	emit('input', event)
}

function clear() {
	model.value = ''
	emit('clear')
}

function openPicker() {
	if (props.disabled || props.readonly) return

	const input = inputRef.value
	if (!input) return

	input.focus()
	if (input.showPicker) {
		input.showPicker()
	} else {
		input.click()
	}
}
</script>
