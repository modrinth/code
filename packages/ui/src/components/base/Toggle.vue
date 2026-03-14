<template>
	<button
		:id="id"
		type="button"
		role="switch"
		:aria-checked="modelValue"
		:disabled="disabled"
		class="relative inline-flex shrink-0 rounded-full m-0 transition-all duration-200 cursor-pointer border-none"
		:class="[
			small ? 'h-5 !w-[40px]' : 'h-6 !w-[48px]',
			modelValue ? 'bg-brand' : 'bg-button-bg',
			disabled ? 'opacity-50 cursor-not-allowed' : 'btn-wrapper',
		]"
		@click="toggle"
	>
		<span
			class="absolute rounded-full transition-all duration-200"
			:class="[
				small ? 'w-4 h-4 top-0.5 left-0.5' : 'w-5 h-5 top-[2px] left-[2px]',
				modelValue
					? small
						? 'translate-x-5 bg-black/90'
						: 'translate-x-6 bg-black/90'
					: 'bg-gray',
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
