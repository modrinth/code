<template>
	<div class="flex w-full items-center gap-4">
		<span class="shrink-0 whitespace-nowrap py-2 text-sm leading-5 text-secondary">
			{{ min }}
		</span>

		<div class="relative h-10 min-w-0 flex-1" :class="disabled ? 'opacity-50' : ''">
			<div
				class="pointer-events-none absolute inset-x-0 top-1/2 h-1 -translate-y-1/2 rounded-full bg-surface-5"
			>
				<div class="h-full rounded-full bg-brand" :style="{ width: `${currentPercentage}%` }" />
			</div>

			<div
				v-if="visibleSnapPoints.length"
				class="pointer-events-none absolute inset-x-0 top-1/2 h-6 -translate-y-1/2"
			>
				<span
					v-for="snapPoint in visibleSnapPoints"
					:key="snapPoint"
					class="absolute top-0 h-6 w-1 -translate-x-1/2 rounded-full"
					:class="snapPoint <= currentValue ? 'bg-brand' : 'bg-surface-5'"
					:style="{ left: `${getPercentage(snapPoint)}%` }"
				/>
			</div>

			<input
				ref="input"
				v-model="currentValue"
				type="range"
				:min="min"
				:max="max"
				:step="step"
				class="slider absolute top-0 h-10 min-h-0 appearance-none border-0 bg-transparent p-0 shadow-none outline-none"
				:class="disabled ? 'cursor-not-allowed' : 'cursor-pointer'"
				:disabled="disabled"
				@input="onInputWithSnap(($event.target as HTMLInputElement).value)"
			/>
		</div>

		<span class="shrink-0 whitespace-nowrap py-2 text-sm leading-5 text-secondary">
			{{ formatValue(max) }}
		</span>

		<Input
			:model-value="String(currentValue)"
			type="number"
			size="medium"
			wrapper-class="slider-value shrink-0"
			input-class="!font-semibold"
			:style="{ width: valueInputWidth }"
			:disabled="disabled"
			:min="min"
			:max="max"
			:step="step"
			@change="onInput(($event.target as HTMLInputElement).value)"
		/>
	</div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'

import Input from './inputs/Input.vue'

const emit = defineEmits<{ 'update:modelValue': [number] }>()

interface Props {
	modelValue?: number
	min: number
	max: number
	step?: number
	forceStep?: boolean
	snapPoints?: number[]
	snapRange?: number
	disabled?: boolean
	unit?: string
}

const props = withDefaults(defineProps<Props>(), {
	modelValue: 0,
	min: 0,
	max: 100,
	step: 10,
	forceStep: true,
	snapPoints: () => [],
	snapRange: 100,
	disabled: false,
	unit: '',
})

const currentValue = ref(clampValue(props.modelValue))
const currentPercentage = computed(() => getPercentage(currentValue.value))
const valueInputWidth = computed(
	() => `calc(${Math.max(String(currentValue.value).length, 1)}ch + 2.125rem)`,
)
const visibleSnapPoints = computed(() =>
	props.snapPoints.filter((snapPoint) => snapPoint >= props.min && snapPoint <= props.max),
)

watch(
	() => props.modelValue,
	(newValue) => {
		currentValue.value = clampValue(newValue ?? props.min)
	},
)

function clampValue(value: number) {
	return Math.max(props.min, Math.min(value, props.max))
}

function getPercentage(value: number) {
	const range = props.max - props.min
	if (range <= 0) return 0

	return Math.max(0, Math.min(((value - props.min) / range) * 100, 100))
}

function formatValue(value: number) {
	return props.unit ? `${value} ${props.unit}` : String(value)
}

function inputValueValid(inputValue: number) {
	if (Number.isNaN(inputValue)) return

	let newValue = inputValue
	if (props.forceStep && props.step > 0) {
		newValue -= newValue % props.step
	}

	currentValue.value = clampValue(newValue)
	emit('update:modelValue', currentValue.value)
}

function onInputWithSnap(value: string) {
	let parsedValue = Number.parseFloat(value)

	for (const snapPoint of props.snapPoints) {
		const distance = Math.abs(snapPoint - parsedValue)
		if (distance < props.snapRange) parsedValue = snapPoint
	}

	inputValueValid(parsedValue)
}

function onInput(value: string) {
	inputValueValid(Number.parseFloat(value))
}
</script>

<style lang="scss" scoped>
.slider {
	left: -0.625rem;
	width: calc(100% + 1.25rem);

	&::-webkit-slider-runnable-track {
		height: 0.25rem;
		background: transparent;
	}

	&::-moz-range-track,
	&::-moz-range-progress {
		height: 0.25rem;
		background: transparent;
	}

	&::-webkit-slider-thumb {
		-webkit-appearance: none;
		appearance: none;
		width: 1.25rem;
		height: 1.25rem;
		margin-top: -0.5rem;
		border: 0;
		border-radius: 9999px;
		background: var(--color-text-default);
		box-shadow:
			0 0 0 2px var(--surface-3),
			0 0 0 4px var(--color-brand);
	}

	&::-moz-range-thumb {
		width: 1.25rem;
		height: 1.25rem;
		border: 0;
		border-radius: 9999px;
		background: var(--color-text-default);
		box-shadow:
			0 0 0 2px var(--surface-3),
			0 0 0 4px var(--color-brand);
	}

	&:focus-visible::-webkit-slider-thumb {
		box-shadow:
			0 0 0 2px var(--surface-3),
			0 0 0 4px var(--color-brand),
			0 0 0 8px var(--color-brand-highlight);
	}

	&:focus-visible::-moz-range-thumb {
		box-shadow:
			0 0 0 2px var(--surface-3),
			0 0 0 4px var(--color-brand),
			0 0 0 8px var(--color-brand-highlight);
	}

	&:disabled {
		pointer-events: none;
		opacity: 1;
	}
}

.slider-value :deep(input[type='number']) {
	-moz-appearance: textfield;

	&::-webkit-inner-spin-button,
	&::-webkit-outer-spin-button {
		margin: 0;
		-webkit-appearance: none;
	}
}
</style>
