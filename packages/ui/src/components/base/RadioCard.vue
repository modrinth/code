<script setup lang="ts" generic="T extends string | number | boolean">
import { RadioButtonCheckedIcon, RadioButtonIcon } from '@modrinth/assets'
import { type Component, computed, useId } from 'vue'

const props = withDefaults(
	defineProps<{
		modelValue: T
		value: T
		name: string
		label: string
		description?: string
		icon?: Component
		disabled?: boolean
		maxWidth?: string
	}>(),
	{ disabled: false },
)

const emit = defineEmits<{
	'update:modelValue': [value: T]
}>()
const id = useId()
const checked = computed(() => props.modelValue === props.value)
</script>

<template>
	<label
		class="group relative block min-w-0"
		:class="disabled ? 'cursor-not-allowed opacity-50' : 'cursor-pointer'"
		:style="{ maxWidth }"
	>
		<input
			class="peer sr-only"
			type="radio"
			:name="name"
			:value="value"
			:checked="checked"
			:disabled="disabled"
			:aria-labelledby="`${id}-label`"
			:aria-describedby="description ? `${id}-description` : undefined"
			@change="emit('update:modelValue', value)"
		/>
		<span
			class="radio-card border-surface-4 bg-surface-2 peer-checked:border-brand peer-focus-visible:outline-brand box-border flex h-full flex-col gap-1 rounded-xl border border-solid p-3.5 transition-[border-color,background-color,filter] group-hover:peer-[:enabled:not(:checked)]:border-surface-4 group-hover:peer-[:enabled:not(:checked)]:brightness-125 peer-focus-visible:outline peer-focus-visible:outline-2 peer-focus-visible:outline-offset-2 motion-reduce:transition-none"
		>
			<span class="flex items-center gap-2">
				<component
					:is="icon"
					v-if="icon"
					aria-hidden="true"
					class="size-4 shrink-0"
					:class="checked ? 'text-brand' : 'text-secondary'"
				/>
				<span :id="`${id}-label`" class="text-contrast min-w-0 flex-1 text-sm font-semibold">{{
					label
				}}</span>
				<component
					:is="checked ? RadioButtonCheckedIcon : RadioButtonIcon"
					aria-hidden="true"
					class="size-4 shrink-0"
					:class="checked ? 'text-brand' : 'text-secondary'"
				/>
			</span>
			<span
				v-if="description"
				:id="`${id}-description`"
				class="text-secondary text-xs leading-relaxed"
				>{{ description }}</span
			>
		</span>
	</label>
</template>

<style scoped>
.peer:checked + .radio-card {
	border-color: var(--color-brand);
	background-color: color-mix(in srgb, var(--color-brand) 8%, var(--surface-2));
}
</style>
