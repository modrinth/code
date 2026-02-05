<template>
	<div
		class="relative"
		:class="[
			wrapperClass,
			multiline ? 'flex' : 'inline-flex',
			{ 'opacity-50 cursor-not-allowed': disabled },
			!multiline && variant === 'outlined' ? 'items-stretch' : 'items-center',
		]"
	>
		<!-- Left Icon (filled variant, single-line only) -->
		<component
			:is="icon"
			v-if="icon && variant === 'filled' && !multiline"
			class="absolute left-3 h-5 w-5 z-[1] pointer-events-none transition-colors"
			:class="[isFocused ? 'opacity-100 text-contrast' : 'opacity-60 text-secondary']"
			aria-hidden="true"
		/>

		<!-- Multiline Textarea -->
		<textarea
			v-if="multiline"
			:id="id"
			:value="modelValue"
			:placeholder="placeholder"
			:disabled="disabled"
			:readonly="readonly"
			:name="name"
			:autocomplete="autocomplete"
			:maxlength="maxlength"
			:rows="rows"
			class="w-full focus:text-contrast font-medium transition-[shadow,color] appearance-none shadow-none focus:ring-4 focus:ring-brand-shadow bg-surface-4 border-none rounded-xl"
			:class="[
				inputClass,
				'pl-3 pr-3 py-2 text-base',
				error ? 'outline outline-2 outline-red bg-warning-bg' : 'outline-none',
				disabled ? 'cursor-not-allowed' : '',
				resize === 'none' ? 'resize-none' : resize === 'vertical' ? 'resize-y' : 'resize',
			]"
			@input="onInput"
			@focus="isFocused = true"
			@blur="isFocused = false"
		/>

		<!-- Single-line Input -->
		<input
			v-else
			:id="id"
			:type="type"
			:value="modelValue"
			:placeholder="placeholder"
			:disabled="disabled"
			:readonly="readonly"
			:name="name"
			:autocomplete="autocomplete"
			:inputmode="inputmode"
			:maxlength="maxlength"
			:min="min"
			:max="max"
			:step="step"
			class="w-full focus:text-contrast font-medium transition-[shadow,color] appearance-none shadow-none focus:ring-4 focus:ring-brand-shadow"
			:class="[
				inputClass,
				variant === 'filled' && icon ? 'pl-10' : variant === 'outlined' ? 'pl-3' : 'pl-3',
				clearable && modelValue && variant === 'filled' ? 'pr-8' : 'pr-3',
				size === 'small' ? 'h-8 py-1.5 text-sm' : 'h-9 py-2 text-base',
				error ? 'outline outline-2 outline-red bg-warning-bg' : 'outline-none',
				disabled ? 'cursor-not-allowed' : '',
				variant === 'outlined'
					? 'bg-transparent border border-solid border-button-bg rounded-l-xl border-r-0'
					: 'bg-surface-4 border-none rounded-xl',
			]"
			@input="onInput"
			@focus="isFocused = true"
			@blur="isFocused = false"
		/>

		<!-- Clear Button (right side, filled variant, single-line only) -->
		<button
			v-if="!multiline && clearable && modelValue && !disabled && !readonly && variant === 'filled'"
			type="button"
			class="absolute right-0.5 z-[1] p-2 bg-transparent border-none text-secondary hover:text-contrast transition-colors cursor-pointer"
			aria-label="Clear input"
			@click="clear"
		>
			<XIcon class="h-5 w-5" />
		</button>

		<!-- Right Icon Button (outlined variant, single-line only) -->
		<button
			v-if="!multiline && variant === 'outlined'"
			type="button"
			class="flex items-center justify-center px-2 bg-transparent border border-solid border-button-bg rounded-r-xl text-secondary hover:text-contrast transition-colors shrink-0"
			:aria-label="clearable && modelValue ? 'Clear input' : undefined"
			@click="clearable && modelValue ? clear() : undefined"
		>
			<XIcon v-if="clearable && modelValue" class="h-4 w-4" />
			<component :is="icon" v-else-if="icon" class="h-4 w-4" />
			<SearchIcon v-else class="h-4 w-4" />
		</button>

		<!-- Custom Right Slot -->
		<slot name="right" />
	</div>
</template>

<script setup lang="ts">
import { SearchIcon, XIcon } from '@modrinth/assets'
import { type Component, ref } from 'vue'

const props = withDefaults(
	defineProps<{
		modelValue?: string | number
		icon?: Component
		type?: 'text' | 'email' | 'password' | 'number' | 'url' | 'search' | 'date' | 'datetime-local'
		placeholder?: string
		id?: string
		name?: string
		autocomplete?: string
		inputmode?: 'none' | 'text' | 'decimal' | 'numeric' | 'tel' | 'search' | 'email' | 'url'
		maxlength?: number
		min?: number
		max?: number
		step?: number
		disabled?: boolean
		readonly?: boolean
		error?: boolean
		size?: 'standard' | 'small'
		variant?: 'filled' | 'outlined'
		clearable?: boolean
		multiline?: boolean
		rows?: number
		resize?: 'none' | 'vertical' | 'both'
		inputClass?: string
		wrapperClass?: string
	}>(),
	{
		type: 'text',
		size: 'standard',
		variant: 'filled',
		disabled: false,
		readonly: false,
		error: false,
		clearable: false,
		multiline: false,
		rows: 3,
		resize: 'none',
	},
)

const emit = defineEmits<{
	'update:modelValue': [value: string | number]
	clear: []
}>()

const isFocused = ref(false)

function onInput(event: Event) {
	const target = event.target as HTMLInputElement | HTMLTextAreaElement
	const value = props.type === 'number' && !props.multiline ? Number(target.value) : target.value
	emit('update:modelValue', value)
}

function clear() {
	emit('update:modelValue', '')
	emit('clear')
}
</script>
