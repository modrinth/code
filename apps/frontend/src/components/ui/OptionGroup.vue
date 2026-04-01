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
	name?: string
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
</style>
