<script setup lang="ts">
defineProps<{
	modelValue?: boolean
	disabled?: boolean
	label: string
	onLabel: string
	offLabel: string
	placeholder?: string
}>()

const emit = defineEmits<{
	'update:model-value': [value: boolean]
}>()
</script>

<template>
	<div
		v-tooltip="modelValue === undefined ? placeholder : undefined"
		role="group"
		:aria-label="label"
		class="inline-flex h-9 items-center gap-0.5 rounded-xl border border-solid border-surface-5 bg-transparent p-px shadow-sm"
		:class="disabled ? 'opacity-50' : ''"
	>
		<button
			type="button"
			:aria-pressed="modelValue === true"
			:disabled="disabled"
			class="flex h-8 cursor-pointer items-center rounded-[10px] border border-solid px-3 text-base font-medium leading-5 text-secondary transition-all active:scale-[0.97] focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-brand-shadow disabled:cursor-not-allowed"
			:class="
				modelValue === true
					? 'border-green bg-highlight-green !text-green'
					: 'border-transparent bg-transparent hover:bg-surface-4'
			"
			@click="emit('update:model-value', true)"
		>
			{{ onLabel }}
		</button>
		<button
			type="button"
			:aria-pressed="modelValue === false"
			:disabled="disabled"
			class="flex h-8 cursor-pointer items-center rounded-[10px] border border-solid px-3 text-base font-medium leading-5 text-secondary transition-all active:scale-[0.97] focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-brand-shadow disabled:cursor-not-allowed"
			:class="
				modelValue === false
					? 'border-surface-5 bg-surface-4'
					: 'border-transparent bg-transparent hover:bg-surface-4'
			"
			@click="emit('update:model-value', false)"
		>
			{{ offLabel }}
		</button>
	</div>
</template>
