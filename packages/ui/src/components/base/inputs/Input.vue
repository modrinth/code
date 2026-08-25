<template>
	<InputFrame
		:appearance="appearance"
		:class="[wrapperClass, attrs.class]"
		:disabled="disabled"
		:invalid="error"
		:size="size"
		:style="attrs.style"
	>
		<template v-if="icon || $slots.leading" #leading>
			<span
				class="flex size-5 shrink-0 items-center justify-center text-secondary opacity-60 transition-colors [&>svg]:size-5 group-focus-within/input:text-contrast group-focus-within/input:opacity-100"
				aria-hidden="true"
			>
				<slot name="leading">
					<component :is="icon" />
				</slot>
			</span>
		</template>

		<div class="flex min-w-0 flex-1 self-stretch items-center" @click="inputRef?.focus()">
			<div class="flex min-w-0 flex-1 items-baseline">
				<span
					v-if="$slots.prefix"
					class="shrink-0 font-medium text-secondary opacity-60 transition-colors group-focus-within/input:text-contrast group-focus-within/input:opacity-100"
					:class="size === 'small' ? 'text-sm' : 'text-base'"
				>
					<slot name="prefix" />
				</span>

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
					:autocorrect="autocorrect"
					:autocapitalize="autocapitalize"
					:spellcheck="spellcheck"
					:inputmode="inputmode"
					:maxlength="maxlength"
					:min="min"
					:max="max"
					:step="step"
					:aria-invalid="error || undefined"
					class="min-w-0 w-full flex-1 appearance-none !min-h-0 !border-0 !bg-transparent !p-0 font-medium text-primary !shadow-none !outline-none placeholder:text-secondary focus:text-contrast focus:ring-0"
					:class="[
						inputClass,
						controlClass,
						size === 'small' ? 'text-sm' : 'text-base',
						disabled ? 'cursor-not-allowed' : '',
					]"
					@input="onInput"
				/>
			</div>
		</div>

		<template v-if="showClearButton || $slots.trailing || $slots.right" #trailing>
			<span class="flex shrink-0 items-center gap-2">
				<InputClearButton v-if="showClearButton" :label="clearLabel" @click="clear" />
				<slot name="trailing" />
				<slot name="right" />
			</span>
		</template>
	</InputFrame>
</template>

<script setup lang="ts">
import { type Component, computed, type InputHTMLAttributes, mergeProps, ref, useAttrs } from 'vue'

import InputClearButton from './InputClearButton.vue'
import InputFrame from './InputFrame.vue'
import type { InputAppearance, InputSize } from './types'

defineOptions({ inheritAttrs: false })

const model = defineModel<string | number | undefined>()
const attrs = useAttrs()

const props = withDefaults(
	defineProps<{
		appearance?: InputAppearance
		autocomplete?: string
		autocapitalize?: 'none' | 'off' | 'sentences' | 'words' | 'characters'
		autocorrect?: 'on' | 'off'
		clearLabel?: string
		clearable?: boolean
		clamp?: boolean
		controlClass?: string
		disabled?: boolean
		error?: boolean
		icon?: Component
		id?: string
		inputAttrs?: InputHTMLAttributes
		inputClass?: string
		inputmode?: 'none' | 'text' | 'decimal' | 'numeric' | 'tel' | 'search' | 'email' | 'url'
		max?: number
		maxlength?: number
		min?: number
		name?: string
		placeholder?: string
		readonly?: boolean
		size?: InputSize
		spellcheck?: boolean
		step?: number
		type?: 'text' | 'email' | 'password' | 'number' | 'url' | 'search'
		wrapperClass?: string
	}>(),
	{
		appearance: 'surface',
		clearLabel: 'Clear input',
		clearable: false,
		clamp: false,
		disabled: false,
		error: false,
		readonly: false,
		size: 'standard',
		type: 'text',
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

defineExpose({
	focus: () => inputRef.value?.focus(),
	select: () => inputRef.value?.select(),
})

function controlAttrs() {
	const { class: _class, style: _style, ...rest } = attrs
	return rest
}

function onInput(event: Event) {
	const target = event.target as HTMLInputElement
	if (props.type !== 'number') {
		model.value = target.value
	} else if (target.value === '') {
		model.value = undefined
	} else {
		let value = target.valueAsNumber
		if (props.clamp) {
			if (props.min !== undefined) value = Math.max(props.min, value)
			if (props.max !== undefined) value = Math.min(props.max, value)
			target.value = String(value)
		}
		model.value = value
	}

	emit('input', event)
}

function clear() {
	model.value = props.type === 'number' ? undefined : ''
	emit('clear')
}
</script>
