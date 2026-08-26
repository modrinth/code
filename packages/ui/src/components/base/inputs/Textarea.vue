<template>
	<InputFrame
		:appearance="appearance"
		:class="[wrapperClass, attrs.class]"
		:disabled="disabled"
		:invalid="error"
		multiline
		:style="attrs.style"
	>
		<template v-if="$slots.leading" #leading>
			<span
				class="flex size-5 shrink-0 items-center justify-center text-secondary opacity-60 transition-colors [&>svg]:size-5 group-focus-within/input:text-contrast group-focus-within/input:opacity-100"
				aria-hidden="true"
			>
				<slot name="leading" />
			</span>
		</template>

		<textarea
			v-bind="mergeProps(controlAttrs(), inputAttrs ?? {})"
			:id="id"
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
			:aria-invalid="error || undefined"
			class="min-h-0 min-w-0 w-full flex-1 self-stretch appearance-none border-0 bg-transparent p-0 font-medium text-base text-primary shadow-none outline-none placeholder:text-secondary focus:text-contrast focus:ring-0"
			:class="[inputClass, controlClass, resizeClass, disabled ? 'cursor-not-allowed' : '']"
			@input="onInput"
		/>

		<template v-if="$slots.trailing || $slots.right" #trailing>
			<span class="flex shrink-0 items-start gap-2">
				<slot name="trailing" />
				<slot name="right" />
			</span>
		</template>
	</InputFrame>
</template>

<script setup lang="ts">
import { computed, mergeProps, ref, type TextareaHTMLAttributes, useAttrs } from 'vue'

import InputFrame from './InputFrame.vue'
import type { InputAppearance } from './types'

defineOptions({ inheritAttrs: false })

const model = defineModel<string | undefined>()
const attrs = useAttrs()

const props = withDefaults(
	defineProps<{
		appearance?: InputAppearance
		autocomplete?: string
		autocapitalize?: 'none' | 'off' | 'sentences' | 'words' | 'characters'
		autocorrect?: 'on' | 'off'
		controlClass?: string
		disabled?: boolean
		error?: boolean
		id?: string
		inputAttrs?: TextareaHTMLAttributes
		inputClass?: string
		maxlength?: number
		name?: string
		placeholder?: string
		readonly?: boolean
		resize?: 'none' | 'vertical' | 'both'
		rows?: number
		spellcheck?: boolean
		wrapperClass?: string
	}>(),
	{
		appearance: 'surface',
		disabled: false,
		error: false,
		readonly: false,
		resize: 'none',
		rows: 3,
	},
)

const emit = defineEmits<{
	input: [event: Event]
}>()

const inputRef = ref<HTMLTextAreaElement>()
const resizeClass = computed(
	() => ({ none: 'resize-none', vertical: 'resize-y', both: 'resize' })[props.resize],
)

defineExpose({ focus: () => inputRef.value?.focus() })

function controlAttrs() {
	const { class: _class, style: _style, ...rest } = attrs
	return rest
}

function onInput(event: Event) {
	model.value = (event.target as HTMLTextAreaElement).value
	emit('input', event)
}
</script>
