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
			class="pointer-events-none absolute left-2 z-[1] h-4 w-4 text-gray-600"
			aria-hidden="true"
		/>

		<!-- Multiline textarea -->
		<textarea
			v-if="multiline"
			:id="id"
			ref="inputRef"
			:value="model"
			:placeholder="placeholder"
			:disabled="disabled"
			:readonly="readonly"
			:name="name"
			:autocomplete="autocomplete"
			:maxlength="maxlength"
			:rows="rows"
			class="retro-field retro-field-textarea w-full box-border"
			:class="[
				inputClass,
				'pl-2 pr-2 py-1.5 text-sm',
				error && 'retro-field--error',
				disabled ? 'cursor-not-allowed' : '',
				resizeClass,
			]"
			@input="onInput"
		/>

		<!-- Single-line input -->
		<input
			v-else
			:id="id"
			ref="inputRef"
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
			class="retro-field w-full box-border"
			:class="[
				inputClass,
				variant === 'filled' && icon ? 'pl-8' : 'pl-2',
				clearable && model && variant === 'filled' ? 'pr-7' : 'pr-2',
				size === 'small' ? 'h-7 py-0.5 text-xs' : 'h-8 py-1 text-sm',
				error && 'retro-field--error',
				disabled ? 'cursor-not-allowed' : '',
				variant === 'outlined'
					? 'retro-field--outlined-left rounded-none border-r-0'
					: 'rounded-none',
			]"
			@input="onInput"
		/>

		<!-- Clear button (right side, filled variant, single-line only) -->
		<button
			v-if="!multiline && clearable && model && !disabled && !readonly && variant === 'filled'"
			type="button"
			class="retro-outset-btn absolute right-0.5 top-1/2 z-[1] -translate-y-1/2 cursor-pointer p-0.5 select-none"
			aria-label="Clear input"
			@click="clear"
		>
			<XIcon class="h-3.5 w-3.5 text-black" />
		</button>

		<!-- Right icon button (outlined variant, single-line only) -->
		<button
			v-if="!multiline && variant === 'outlined'"
			type="button"
			class="retro-outset-btn flex shrink-0 items-center justify-center px-2 text-black"
			:class="size === 'small' ? 'h-7 min-w-[1.75rem]' : 'h-8 min-w-[2rem]'"
			:aria-label="clearable && model ? 'Clear input' : 'Search'"
			:tabindex="clearable && model ? undefined : -1"
			@click="clearable && model ? clear() : undefined"
		>
			<XIcon v-if="clearable && model" class="h-3.5 w-3.5" />
			<component :is="icon" v-else-if="icon" class="h-3.5 w-3.5" />
			<SearchIcon v-else class="h-3.5 w-3.5" />
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

<style scoped>
.retro-field {
	font-family: Tahoma, Verdana, 'MS Sans Serif', Arial, sans-serif;
	color: black;
	background: white;
	border: 2px solid;
	border-color: #838383;
	appearance: none;
}

.retro-field::placeholder {
	color: #6d6d6d;
}

.retro-field:focus {
	border-color: #838383;
	outline: 1px dotted #000;
	outline-offset: -3px;
}

.retro-field:disabled {
	color: #acacac;
	background: #e8e8e8;
	box-shadow: none;
}

.retro-field-textarea {
	min-height: 4.5rem;
	line-height: 1.35;
}

.retro-field--outlined-left {
	border-top-right-radius: 0;
	border-bottom-right-radius: 0;
}
</style>
