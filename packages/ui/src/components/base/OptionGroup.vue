<template>
	<div class="option-group" role="radiogroup" :aria-label="ariaLabel">
		<label v-for="(option, index) in options" :key="`option-group-${index}`">
			<input
				v-model="modelValue"
				class="option-group__input"
				type="radio"
				:name="radioName"
				:value="option"
			/>
			<slot :option="option" :selected="modelValue === option" />
		</label>
	</div>
</template>

<script setup lang="ts" generic="T">
import { computed, useId } from 'vue'

const modelValue = defineModel<T>({ required: true })

const props = defineProps<{
	options: T[]
	/** Shared `name` for the radio group; defaults to a unique id. */
	name?: string
	/** Accessible name for the radiogroup (recommended when the group has no visible legend). */
	ariaLabel?: string
}>()

const autoName = useId()
const radioName = computed(() => props.name ?? autoName)
</script>

<style scoped>
.option-group {
	display: flex;
	flex-direction: column;
	gap: 0.25em;
}

.option-group label {
	display: inline-flex;
	align-items: center;
	gap: 0.4em;
	cursor: pointer;
}

/*
 * Frontend global styles apply text-field rules to all inputs (appearance: none, padding, min-height).
 * Restore the platform radio control so it stays visible everywhere (site + Storybook).
 */
.option-group__input[type='radio'] {
	appearance: auto !important;
	-webkit-appearance: radio !important;
	-moz-appearance: auto !important;
	box-sizing: border-box !important;
	width: auto !important;
	height: auto !important;
	min-width: 0 !important;
	min-height: 0 !important;
	max-height: none !important;
	max-width: none !important;
	margin: 0 !important;
	padding: 0 !important;
	box-shadow: none !important;
	font: inherit !important;
	font-weight: inherit !important;
	line-height: normal !important;
	color: inherit !important;
	outline: revert-layer !important;
	accent-color: var(--color-brand);
}
</style>
