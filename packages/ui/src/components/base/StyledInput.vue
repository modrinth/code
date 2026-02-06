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
		<!-- Left icon (filled variant, single-line only) -->
		<component
			:is="icon"
			v-if="icon && variant === 'filled' && !multiline"
			class="absolute left-3 h-5 w-5 z-[1] pointer-events-none transition-colors"
			:class="[isFocused ? 'opacity-100 text-contrast' : 'opacity-60 text-secondary']"
			aria-hidden="true"
		/>

		<!-- Multiline textarea -->
		<textarea
			v-if="multiline"
			ref="inputRef"
			:id="id"
			:value="model"
			:placeholder="placeholder"
			:disabled="disabled"
			:readonly="readonly"
			:name="name"
			:autocomplete="autocomplete"
			:maxlength="maxlength"
			:rows="rows"
			class="w-full text-primary placeholder:text-secondary focus:text-contrast font-medium transition-[shadow,color] appearance-none shadow-none focus:ring-4 focus:ring-brand-shadow bg-surface-4 border-none rounded-xl"
			:class="[
				inputClass,
				'pl-3 pr-3 py-2 text-base',
				error ? 'outline outline-2 outline-red bg-warning-bg' : 'outline-none',
				disabled ? 'cursor-not-allowed' : '',
				resizeClass,
			]"
			@input="onInput"
			@focus="isFocused = true"
			@blur="isFocused = false"
		/>

		<!-- Single-line input -->
		<input
			v-else
			ref="inputRef"
			:id="id"
			:type="type"
			:value="model"
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
			class="w-full text-primary placeholder:text-secondary focus:text-contrast font-medium transition-[shadow,color] appearance-none shadow-none focus:ring-4 focus:ring-brand-shadow"
			:class="[
				inputClass,
				variant === 'filled' && icon ? 'pl-10' : 'pl-3',
				clearable && model && variant === 'filled' ? 'pr-8' : 'pr-3',
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

		<!-- Clear button (right side, filled variant, single-line only) -->
		<button
			v-if="!multiline && clearable && model && !disabled && !readonly && variant === 'filled'"
			type="button"
			class="absolute right-0.5 z-[1] p-2 bg-transparent border-none text-secondary hover:text-contrast transition-colors cursor-pointer select-none"
			aria-label="Clear input"
			@click="clear"
		>
			<XIcon class="h-5 w-5" />
		</button>

		<!-- Right icon button (outlined variant, single-line only) -->
		<button
			v-if="!multiline && variant === 'outlined'"
			type="button"
			class="flex items-center justify-center px-2 bg-transparent border border-solid border-button-bg rounded-r-xl text-secondary hover:text-contrast transition-colors shrink-0"
			:aria-label="clearable && model ? 'Clear input' : 'Search'"
			:tabindex="clearable && model ? undefined : -1"
			@click="clearable && model ? clear() : undefined"
		>
			<XIcon v-if="clearable && model" class="h-4 w-4" />
			<component :is="icon" v-else-if="icon" class="h-4 w-4" />
			<SearchIcon v-else class="h-4 w-4" />
		</button>

		<!-- Custom rightside slot -->
		<slot name="right" />
	</div>
</template>

<script setup lang="ts">
import { SearchIcon, XIcon } from '@modrinth/assets'
import { type Component, computed, ref } from 'vue'

const model = defineModel<string | number | undefined>()

const props = withDefaults(
	defineProps<{
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
	clear: []
}>()

const inputRef = ref<HTMLInputElement | HTMLTextAreaElement>()
const isFocused = ref(false)
const resizeClass = computed(
	() => ({ none: 'resize-none', vertical: 'resize-y', both: 'resize' })[props.resize ?? 'none'],
)

defineExpose({ focus: () => inputRef.value?.focus() })

function onInput(event: Event) {
	const target = event.target as HTMLInputElement | HTMLTextAreaElement
	model.value =
		props.type === 'number' && !props.multiline
			? target.value === ''
				? undefined
				: Number(target.value)
			: target.value
}

function clear() {
	model.value = props.type === 'number' && !props.multiline ? undefined : ''
	emit('clear')
}
</script>
