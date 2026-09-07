<template>
	<div
		:class="[wrapperClass, attrs.class]"
		:style="attrs.style"
		class="flex w-full min-w-0 max-w-[18.375rem] flex-col gap-3"
	>
		<Input
			v-if="backupCode"
			v-bind="controlAttrs()"
			:id="controlId"
			ref="backupInput"
			:model-value="model"
			:appearance="appearance"
			:disabled="disabled"
			:readonly="readonly"
			:error="error"
			:size="size"
			:name="name"
			:aria-label="accessibleLabel"
			:placeholder="formatMessage(messages.backupCodePlaceholder)"
			autocomplete="off"
			autocapitalize="none"
			autocorrect="off"
			:spellcheck="false"
			inputmode="text"
			dir="ltr"
			@update:model-value="model = String($event ?? '').trim()"
		/>
		<div v-else class="relative isolate w-full max-w-[18.375rem]" dir="ltr">
			<div class="grid grid-cols-6 gap-1.5" aria-hidden="true">
				<InputFrame
					v-for="index in codeLength"
					:key="index"
					:appearance="appearance"
					:disabled="disabled"
					:invalid="error"
					:size="size"
					class="relative justify-center !px-0 tabular-nums"
					:class="{ 'ring-4 ring-brand-shadow': focused && isSelected(index - 1) }"
				>
					<span
						:class="{
							'bg-brand-highlight':
								focused && isSelected(index - 1) && selectionEnd > selectionStart,
						}"
					>
						{{ model[index - 1] }}
					</span>
					<span
						v-if="focused && !readonly && selectionStart === selectionEnd && isSelected(index - 1)"
						class="absolute h-5 w-px bg-current"
					/>
				</InputFrame>
			</div>
			<input
				v-bind="controlAttrs()"
				:id="controlId"
				ref="codeInput"
				:value="model"
				:name="name"
				type="text"
				inputmode="numeric"
				pattern="[0-9]{6}"
				:maxlength="codeLength"
				:autocomplete="autocomplete"
				:disabled="disabled"
				:readonly="readonly"
				:aria-label="accessibleLabel"
				:aria-invalid="error || undefined"
				:spellcheck="false"
				class="absolute inset-0 z-10 h-full w-full cursor-text opacity-0 disabled:cursor-not-allowed"
				@focus="onFocus"
				@blur="focused = false"
				@select="updateSelection"
				@keyup="updateSelection"
				@input="onInput"
				@paste="onPaste"
				@pointerdown="onPointerDown"
			/>
		</div>
		<button
			v-if="allowBackupCode"
			type="button"
			class="min-h-6 self-center rounded-sm border-0 bg-transparent p-0 text-center text-sm font-medium text-link hover:underline focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-link disabled:cursor-not-allowed disabled:opacity-50"
			:disabled="disabled || readonly"
			:aria-controls="controlId"
			@click="switchMode"
		>
			{{ formatMessage(backupCode ? messages.useAuthenticationCode : messages.useBackupCode) }}
		</button>
	</div>
</template>

<script setup lang="ts">
import {
	defineMessages,
	Input,
	type InputAppearance,
	InputFrame,
	type InputSize,
	useVIntl,
} from '@modrinth/ui'
import { computed, nextTick, onMounted, ref, useAttrs, useId, watch } from 'vue'

defineOptions({ inheritAttrs: false })

const model = defineModel<string>({ default: '' })
const emit = defineEmits<{
	complete: [code: string]
}>()
const props = withDefaults(
	defineProps<{
		allowBackupCode?: boolean
		appearance?: InputAppearance
		autocomplete?: string
		autofocus?: boolean
		disabled?: boolean
		error?: boolean
		id?: string
		name?: string
		readonly?: boolean
		size?: InputSize
		wrapperClass?: string
	}>(),
	{
		allowBackupCode: false,
		appearance: 'surface',
		autocomplete: 'one-time-code',
		autofocus: false,
		disabled: false,
		error: false,
		readonly: false,
		size: 'large',
	},
)

const attrs = useAttrs()
const generatedId = useId()
const controlId = computed(() => props.id ?? `two-factor-code-${generatedId}`)
const codeLength = 6
const backupCode = ref(false)
const codeInput = ref<HTMLInputElement>()
const backupInput = ref<InstanceType<typeof Input>>()
const focused = ref(false)
const selectionStart = ref(0)
const selectionEnd = ref(0)
const { formatMessage } = useVIntl()
const messages = defineMessages({
	authenticationCode: {
		id: 'ui.two-factor-code.authentication-code',
		defaultMessage: 'Six-digit authentication code',
	},
	backupCode: {
		id: 'ui.two-factor-code.backup-code',
		defaultMessage: 'Backup code',
	},
	backupCodePlaceholder: {
		id: 'ui.two-factor-code.backup-code.placeholder',
		defaultMessage: 'Enter a backup code',
	},
	useBackupCode: {
		id: 'ui.two-factor-code.use-backup-code',
		defaultMessage: 'Use a backup code',
	},
	useAuthenticationCode: {
		id: 'ui.two-factor-code.use-authentication-code',
		defaultMessage: 'Use an authenticator code',
	},
})
const accessibleLabel = computed(
	() =>
		(attrs['aria-label'] as string | undefined) ??
		formatMessage(backupCode.value ? messages.backupCode : messages.authenticationCode),
)

function controlAttrs() {
	const { class: _class, style: _style, ...rest } = attrs
	return rest
}

function sanitizeCode(value: string) {
	return value.replace(/\D/g, '').slice(0, codeLength)
}

function updateSelection() {
	selectionStart.value = codeInput.value?.selectionStart ?? 0
	selectionEnd.value = codeInput.value?.selectionEnd ?? 0
}

function isSelected(index: number) {
	if (selectionStart.value === selectionEnd.value) {
		return index === Math.min(selectionStart.value, codeLength - 1)
	}
	return index >= selectionStart.value && index < selectionEnd.value
}

function onFocus() {
	focused.value = true
	updateSelection()
}

function onInput(event: Event) {
	const input = event.target as HTMLInputElement
	const cursor = sanitizeCode(input.value.slice(0, input.selectionStart ?? 0)).length
	const value = sanitizeCode(input.value)
	model.value = value
	input.value = value
	input.setSelectionRange(cursor, cursor)
	updateSelection()
}

function onPaste(event: ClipboardEvent) {
	if (props.disabled || props.readonly) return
	const value = sanitizeCode(event.clipboardData?.getData('text/plain') ?? '')
	if (!value) return
	event.preventDefault()
	model.value = value
	void nextTick(() => {
		codeInput.value?.setSelectionRange(value.length, value.length)
		updateSelection()
	})
}

function onPointerDown(event: PointerEvent) {
	if (props.disabled || event.button !== 0) return
	event.preventDefault()
	const input = codeInput.value
	if (!input) return
	const bounds = input.getBoundingClientRect()
	const index = Math.max(
		0,
		Math.min(
			Math.floor(((event.clientX - bounds.left) / bounds.width) * codeLength),
			model.value.length,
			codeLength - 1,
		),
	)
	input.focus()
	input.setSelectionRange(index, Math.min(index + 1, model.value.length))
	updateSelection()
}

function focus() {
	if (backupCode.value) {
		backupInput.value?.focus()
	} else {
		codeInput.value?.focus()
		codeInput.value?.setSelectionRange(model.value.length, model.value.length)
		updateSelection()
	}
}

async function switchMode() {
	backupCode.value = !backupCode.value
	model.value = ''
	await nextTick()
	focus()
}

function clear() {
	model.value = ''
	backupCode.value = false
}

onMounted(() => {
	if (props.autofocus) focus()
})

watch(model, (value) => {
	if (props.disabled || props.readonly) return
	const complete = backupCode.value ? /^[A-Za-z0-9]{11}$/.test(value) : /^\d{6}$/.test(value)
	if (complete) emit('complete', value)
})

watch(
	() => props.allowBackupCode,
	(allowed) => {
		if (!allowed && backupCode.value) clear()
	},
)

defineExpose({ clear, focus })
</script>
