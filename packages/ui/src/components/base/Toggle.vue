<template>
	<button
		:id="id"
		type="button"
		role="switch"
		:aria-checked="indeterminate ? 'mixed' : modelValue"
		:disabled="disabled"
		class="group inline-flex shrink-0 touch-manipulation items-center rounded-full m-0 p-1 transition-all duration-200 cursor-pointer border border-solid border-surface-5"
		:class="[
			small ? 'h-5 !w-[40px]' : 'h-6 !w-[48px]',
			indeterminate || modelValue ? 'bg-brand' : 'bg-button-bg',
			disabled ? 'opacity-50 cursor-not-allowed' : '',
		]"
		@click="toggle"
	>
		<span
			class="rounded-full transition-all duration-200"
			:class="[
				indeterminate
					? small
						? 'h-0.5 w-2.5 translate-x-[11px] bg-black/90'
						: 'h-0.5 w-3 translate-x-[14px] bg-black/90'
					: modelValue
						? small
							? 'h-3 w-3 translate-x-[20px] bg-black/90'
							: 'h-4 w-4 translate-x-[24px] bg-black/90'
						: small
							? 'h-3 w-3 bg-gray'
							: 'h-4 w-4 bg-gray',
				disabled
					? ''
					: indeterminate
						? 'group-hover:scale-x-110 group-active:scale-x-75'
						: small
							? 'group-hover:w-[14px] group-hover:h-[14px] group-hover:m-[-1px] group-active:w-[10px] group-active:h-[10px] group-active:m-[1px]'
							: 'group-hover:w-[18px] group-hover:h-[18px] group-hover:m-[-1px] group-active:w-[14px] group-active:h-[14px] group-active:m-[1px]',
			]"
		/>
	</button>
</template>

<script setup lang="ts">
const props = defineProps<{
	id?: string
	disabled?: boolean
	small?: boolean
	indeterminate?: boolean
}>()

const modelValue = defineModel<boolean>()

function toggle() {
	if (!props.disabled) {
		modelValue.value = !modelValue.value
	}
}
</script>
