<template>
	<button
		:id="id"
		type="button"
		role="switch"
		:aria-checked="modelValue"
		:disabled="disabled"
		class="group inline-flex shrink-0 items-center rounded-full m-0 p-1 transition-all duration-200 cursor-pointer border-none"
		:class="[
			small ? 'h-5 !w-[40px]' : 'h-6 !w-[48px]',
			modelValue ? 'bg-brand' : 'bg-button-bg',
			disabled ? 'opacity-50 cursor-not-allowed' : '',
		]"
		@click="toggle"
	>
		<span
			class="rounded-full transition-all duration-200"
			:class="[
				small ? 'w-3 h-3' : 'w-4 h-4',
				modelValue
					? small
						? 'translate-x-[20px] bg-black/90'
						: 'translate-x-[24px] bg-black/90'
					: 'bg-gray',
				disabled
					? ''
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
}>()

const modelValue = defineModel<boolean>()

function toggle() {
	if (!props.disabled) {
		modelValue.value = !modelValue.value
	}
}
</script>
