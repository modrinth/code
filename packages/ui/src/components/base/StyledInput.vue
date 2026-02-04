<template>
	<div
		class="relative inline-flex items-center"
		:class="[wrapperClass, { 'opacity-50 cursor-not-allowed': disabled }]"
	>
		<!-- Left Icon -->
		<component
			:is="icon"
			v-if="icon"
			class="absolute left-3 h-5 w-5 z-[1] pointer-events-none transition-colors"
			:class="[isFocused ? 'opacity-100 text-contrast' : 'opacity-60 text-secondary']"
			aria-hidden="true"
		/>

		<!-- Input -->
		<input
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
			class="w-full rounded-xl bg-surface-4 text-contrast font-medium transition-shadow border-none appearance-none shadow-none"
			:class="[
				inputClass,
				icon ? 'pl-10' : 'pl-2',
				clearable && modelValue ? 'pr-8' : 'pr-2',
				size === 'small' ? 'min-h-8 py-1.5 text-sm' : 'min-h-9 py-2 text-base',
				error ? 'outline outline-2 outline-red bg-warning-bg' : 'outline-none',
				disabled ? 'cursor-not-allowed' : '',
			]"
			@input="onInput"
			@focus="isFocused = true"
			@blur="isFocused = false"
		/>

		<!-- Clear Button (right side) -->
		<button
			v-if="clearable && modelValue && !disabled && !readonly"
			type="button"
			class="absolute right-0.5 z-[1] p-2 bg-transparent border-none text-secondary hover:text-contrast transition-colors cursor-pointer"
			aria-label="Clear input"
			@click="clear"
		>
			<XIcon class="h-5 w-5" />
		</button>

		<!-- Custom Right Slot -->
		<slot name="right" />
	</div>
</template>

<script setup lang="ts">
import { XIcon } from '@modrinth/assets'
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
		clearable?: boolean
		inputClass?: string
		wrapperClass?: string
	}>(),
	{
		type: 'text',
		size: 'standard',
		disabled: false,
		readonly: false,
		error: false,
		clearable: false,
	},
)

const emit = defineEmits<{
	'update:modelValue': [value: string | number]
	clear: []
}>()

const isFocused = ref(false)

function onInput(event: Event) {
	const target = event.target as HTMLInputElement
	const value = props.type === 'number' ? Number(target.value) : target.value
	emit('update:modelValue', value)
}

function clear() {
	emit('update:modelValue', '')
	emit('clear')
}
</script>
