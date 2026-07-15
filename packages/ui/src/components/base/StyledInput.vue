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
			:id="id"
			v-bind="inputAttrs"
			ref="inputRef"
			:value="model"
			:placeholder="placeholder"
			:disabled="disabled"
			:readonly="readonly"
			:name="name"
			:autocomplete="autocomplete"
			:autocorrect="autocorrect"
			:autocapitalize="autocapitalize"
			:spellcheck="spellcheck"
			:maxlength="maxlength"
			:rows="rows"
			class="w-full touch-manipulation text-primary placeholder:text-secondary focus:text-contrast font-medium transition-[shadow,color] appearance-none shadow-none focus:ring-4 focus:ring-brand-shadow border border-solid rounded-xl"
			:class="[
				inputClass,
				'pl-3 pr-3 py-2 text-base',
				error
					? 'outline-none border-red bg-highlight-red'
					: 'outline-none border-surface-5 bg-surface-4',
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
			:id="id"
			v-bind="inputAttrs"
			ref="inputRef"
			:type="type"
			:value="model"
			:placeholder="placeholder"
			:disabled="disabled"
			:readonly="readonly"
			:name="name"
			:autocomplete="autocomplete"
			:autocorrect="autocorrect"
			:autocapitalize="autocapitalize"
			:spellcheck="spellcheck"
			:inputmode="inputmode"
			:maxlength="maxlength"
			:min="min"
			:max="max"
			:step="step"
			class="w-full touch-manipulation text-primary placeholder:text-secondary focus:text-contrast font-medium transition-[shadow,color] appearance-none shadow-none focus:ring-4 focus:ring-brand-shadow"
			:class="[
				inputClass,
				variant === 'filled' && icon ? 'pl-10' : 'pl-3',
				showClearButton || isDateInput ? 'pr-10' : 'pr-3',
				isDateInput ? '[&::-webkit-calendar-picker-indicator]:opacity-0' : '',
				size === 'small' ? 'h-8 py-1.5 text-sm' : 'h-9 py-2 text-base',
				'outline-none',
				disabled ? 'cursor-not-allowed' : '',
				error ? 'bg-highlight-red' : variant === 'outlined' ? 'bg-transparent' : 'bg-surface-4',
				error ? 'border-red' : variant === 'outlined' ? 'border-button-bg' : 'border-surface-5',
				variant === 'outlined'
					? 'border border-solid rounded-l-xl border-r-0'
					: 'border border-solid rounded-xl',
			]"
			@input="onInput"
			@focus="isFocused = true"
			@blur="isFocused = false"
		/>

		<CalendarIcon
			v-if="isDateInput && !showClearButton"
			class="pointer-events-none absolute right-3 top-1/2 z-[1] h-5 w-5 -translate-y-1/2 transition-colors"
			:class="[isFocused ? 'opacity-100 text-contrast' : 'opacity-60 text-secondary']"
			aria-hidden="true"
		/>

		<!-- Clear button (right side, filled variant, single-line only) -->
		<button
			v-if="showClearButton"
			type="button"
			class="absolute right-0.5 top-1/2 z-[1] flex h-full w-10 -translate-y-1/2 touch-manipulation items-center justify-center border-none bg-transparent p-0 transition-colors cursor-pointer select-none hover:opacity-100 hover:text-contrast"
			:class="[isFocused ? 'opacity-100 text-contrast' : 'opacity-60 text-secondary']"
			aria-label="Clear input"
			@click="clear"
		>
			<XIcon class="h-5 w-5" />
		</button>

		<!-- Right icon button (outlined variant, single-line only) -->
		<button
			v-if="!multiline && variant === 'outlined'"
			type="button"
			class="flex touch-manipulation items-center justify-center px-2 bg-transparent border border-solid rounded-r-xl text-secondary hover:text-contrast transition-colors shrink-0"
			:class="error ? 'border-red' : 'border-button-bg'"
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
import { CalendarIcon, SearchIcon, XIcon } from '@modrinth/assets'
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
		autocorrect?: 'on' | 'off'
		autocapitalize?: 'none' | 'off' | 'sentences' | 'words' | 'characters'
		spellcheck?: boolean
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
		inputAttrs?: Record<string, string | number | boolean | undefined>
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
const isDateInput = computed(
	() => !props.multiline && (props.type === 'date' || props.type === 'datetime-local'),
)
const showClearButton = computed(
	() =>
		!props.multiline &&
		props.clearable &&
		Boolean(model.value) &&
		!props.disabled &&
		!props.readonly &&
		props.variant === 'filled',
)
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
