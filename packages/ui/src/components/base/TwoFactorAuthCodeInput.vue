<template>
	<div
		class="flex w-fit flex-wrap gap-1.5"
		:class="[wrapperClass, { 'opacity-50': disabled }]"
		role="group"
		aria-label="Two-factor authentication code"
		@pointerdown="handlePointerDown"
	>
		<input
			v-for="index in codeLength"
			:key="index"
			:ref="(element) => setCodeInput(element, index - 1)"
			:value="digits[index - 1]"
			type="text"
			inputmode="numeric"
			pattern="[0-9]*"
			maxlength="1"
			:autocomplete="index === 1 ? autocomplete : undefined"
			:disabled="disabled"
			:readonly="readonly"
			:aria-label="`Code digit ${index}`"
			class="h-12 w-11 appearance-none rounded-xl border-none bg-surface-4 p-1 text-center text-base font-medium text-primary focus:text-primary focus:ring-4 focus:ring-brand-shadow disabled:cursor-not-allowed"
			:class="[inputClass, 'outline-none']"
			@focus="handleFocus($event)"
			@input="handleInput($event, index - 1)"
			@keydown="handleKeydown($event, index - 1)"
			@paste.prevent="handlePaste"
		/>
	</div>
</template>

<script setup lang="ts">
import { nextTick, ref, watch, type ComponentPublicInstance } from 'vue'

const model = defineModel<string>({ default: '' })

const props = withDefaults(
	defineProps<{
		autocomplete?: string
		disabled?: boolean
		readonly?: boolean
		inputClass?: string
		wrapperClass?: string
	}>(),
	{
		autocomplete: 'one-time-code',
		disabled: false,
		readonly: false,
	},
)

const codeInputs = ref<HTMLInputElement[]>([])
const digits = ref<string[]>([])
const codeLength = 6

watch(
	() => model.value,
	(value) => {
		const sanitizedValue = sanitizeCode(value)
		if (sanitizedValue !== digits.value.join('') || digits.value.length !== codeLength) {
			digits.value = Array.from({ length: codeLength }, (_, index) => sanitizedValue[index] ?? '')
		}

		if (value !== sanitizedValue) {
			model.value = sanitizedValue
		}
	},
	{ immediate: true },
)

function sanitizeCode(value: string) {
	return value.replace(/\D/g, '').slice(0, codeLength)
}

function updateModel() {
	model.value = digits.value.join('')
}

function setCodeInput(element: Element | ComponentPublicInstance | null, index: number) {
	if (element instanceof HTMLInputElement) {
		codeInputs.value[index] = element
	}
}

function focusInput(index: number) {
	const input = codeInputs.value[index]
	input?.focus()
	selectInputValue(input)
}

function focusFirstUnfilledCodeInput() {
	const firstUnfilledIndex = digits.value.findIndex((digit) => !digit)
	const inputIndex = firstUnfilledIndex === -1 ? digits.value.length - 1 : firstUnfilledIndex
	focusInput(inputIndex)
}

function handlePointerDown(event: PointerEvent) {
	if (props.disabled) {
		return
	}

	if (event.target instanceof HTMLInputElement && event.target.value) {
		event.preventDefault()
		event.target.focus()
		selectInputValue(event.target)
		return
	}

	event.preventDefault()
	focusFirstUnfilledCodeInput()
}

function handleFocus(event: FocusEvent) {
	if (props.disabled) {
		return
	}

	selectInputValue(event.target as HTMLInputElement)
}

function handleInput(event: Event, index: number) {
	if (disabledOrReadonly()) {
		return
	}

	const input = event.target as HTMLInputElement
	const inputDigits = input.value.replace(/\D/g, '')

	if (!inputDigits) {
		digits.value[index] = ''
		input.value = ''
		updateModel()
		return
	}

	if (inputDigits.length === 1) {
		const digit = inputDigits.slice(-1)
		digits.value[index] = digit
		input.value = digit
		updateModel()

		if (index < codeLength - 1) {
			focusInput(index + 1)
		}
		return
	}

	const pastedDigits = inputDigits.slice(0, codeLength - index)
	for (const [offset, digit] of Array.from(pastedDigits).entries()) {
		digits.value[index + offset] = digit
	}
	updateModel()
	input.value = digits.value[index] ?? ''
	void nextTick(focusFirstUnfilledCodeInput)
}

function handleKeydown(event: KeyboardEvent, index: number) {
	if (disabledOrReadonly()) {
		return
	}

	if (event.key === 'Backspace' && !digits.value[index] && index > 0) {
		focusInput(index - 1)
	} else if (event.key === 'ArrowLeft' && index > 0) {
		event.preventDefault()
		focusInput(index - 1)
	} else if (event.key === 'ArrowRight' && index < codeLength - 1) {
		event.preventDefault()
		focusInput(index + 1)
	}
}

function handlePaste(event: ClipboardEvent) {
	if (disabledOrReadonly()) {
		return
	}

	const clipboardText = event.clipboardData?.getData('text') ?? ''
	const pastedCode = sanitizeCode(clipboardText)
	if (!pastedCode) {
		return
	}

	digits.value = Array.from({ length: codeLength }, (_, index) => pastedCode[index] ?? '')
	updateModel()
	void nextTick(focusFirstUnfilledCodeInput)
}

function disabledOrReadonly() {
	return props.disabled || props.readonly
}

function selectInputValue(input?: HTMLInputElement) {
	if (!input) {
		return
	}

	if (input.value) {
		input.select()
	} else {
		input.setSelectionRange(input.value.length, input.value.length)
	}
}

function clear() {
	digits.value = Array.from({ length: codeLength }, () => '')
	updateModel()
}

defineExpose({
	clear,
	focus: focusFirstUnfilledCodeInput,
})
</script>
