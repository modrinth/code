<template>
	<div class="flex w-full items-center gap-4">
		<span
			v-if="currentValue !== null"
			class="min-w-10 shrink-0 whitespace-nowrap py-2 text-right text-sm leading-5 text-secondary"
		>
			{{ minLabel ?? min }}
		</span>

		<div
			v-if="currentValue !== null"
			class="relative mx-2 min-w-0 flex-1"
			:class="[heightClass, disabled ? 'opacity-50' : '']"
		>
			<input
				ref="input"
				:value="currentValue"
				type="range"
				:min="min"
				:max="max"
				:step="step"
				class="slider absolute top-0 h-full min-h-0 appearance-none overflow-visible border-0 bg-transparent p-0 shadow-none outline-none"
				:class="disabled ? 'cursor-not-allowed' : 'cursor-pointer'"
				:disabled="disabled"
				:aria-label="ariaLabel"
				@input="onInputWithSnap(($event.target as HTMLInputElement).value)"
			/>
			<div
				class="slider-track pointer-events-none absolute inset-x-0 top-1/2 h-[6px] -translate-y-1/2 rounded-full bg-surface-5"
			>
				<div
					class="filled-slider-track h-full rounded-full bg-brand relative"
					:style="{ width: `${currentPercentage}%` }"
				>
					<div
						class="slider-thumb absolute h-8 w-[10px] rounded-full bg-brand top-[-13px] right-[-5px]"
					></div>
				</div>
			</div>

			<div
				v-if="visibleSnapPoints.length"
				class="pointer-events-none absolute inset-x-0 top-1/2 h-[18px] -translate-y-1/2"
			>
				<span
					v-for="snapPoint in visibleSnapPoints"
					:key="snapPoint"
					class="absolute top-0 h-[18px] w-1.5 -translate-x-1/2 rounded-full"
					:class="snapPoint <= currentValue ? 'bg-brand' : 'bg-surface-5'"
					:style="{ left: `${getPercentage(snapPoint)}%` }"
				/>
			</div>
		</div>

		<span
			v-if="currentValue !== null"
			class="min-w-10 shrink-0 whitespace-nowrap py-2 text-left text-sm leading-5 text-secondary"
		>
			{{ maxLabel ?? formatValue(max) }}
		</span>

		<Input
			:model-value="currentValue ?? undefined"
			type="number"
			:size="size"
			wrapper-class="slider-value shrink-0"
			:style="{ width: currentValue === null ? '100%' : inputWidth }"
			:input-class="currentValue === null ? undefined : 'text-center'"
			:disabled="disabled"
			:placeholder="placeholder"
			:aria-label="ariaLabel"
			:min="min"
			:max="max"
			:step="step"
			@change="onInput"
		/>
	</div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'

import Input from './inputs/Input.vue'
import type { InputSize } from './inputs/types'

const emit = defineEmits<{ 'update:modelValue': [number] }>()

interface Props {
	size?: InputSize
	modelValue?: number | null
	min: number
	max: number
	step?: number
	forceStep?: boolean
	snapPoints?: number[]
	snapRange?: number
	disabled?: boolean
	unit?: string
	minLabel?: string
	maxLabel?: string
	placeholder?: string
	ariaLabel?: string
}

const props = withDefaults(defineProps<Props>(), {
	size: 'medium',
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

const heightClass = computed(
	() =>
		({
			small: 'h-8',
			standard: 'h-9',
			medium: 'h-10',
			large: 'h-12',
		})[props.size],
)
const currentValue = ref(props.modelValue === null ? null : normalizeValue(props.modelValue))
const inputWidth = computed(() => {
	const digits = Math.max(String(props.min).length, String(props.max).length)
	const padding = props.size === 'small' || props.size === 'standard' ? 1.5 : 2
	return `max(65px, calc(${digits}ch + ${padding}rem + 2px))`
})
const currentPercentage = computed(() => getPercentage(currentValue.value ?? props.min))
const visibleSnapPoints = computed(() =>
	props.snapPoints.filter((snapPoint) => snapPoint >= props.min && snapPoint <= props.max),
)

watch(
	() => props.modelValue,
	(newValue) => {
		currentValue.value = newValue === null ? null : normalizeValue(newValue ?? props.min)
	},
)

function normalizeValue(value: number) {
	if (!Number.isFinite(value)) return props.min

	if (props.forceStep && props.step > 0) {
		value = props.min + Math.round((value - props.min) / props.step) * props.step
		value = Number(value.toFixed(8))
	}

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
	if (!Number.isFinite(inputValue)) return

	currentValue.value = normalizeValue(inputValue)
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

function onInput(event: Event) {
	const target = event.target as HTMLInputElement
	inputValueValid(target.valueAsNumber)
	target.value = currentValue.value === null ? '' : String(currentValue.value)
}
</script>

<style lang="scss" scoped>
.slider {
	left: -0.625rem;
	width: calc(100% + 1.25rem);

	&:focus {
		box-shadow: none;
	}

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
		border: 0;
		background: transparent;
	}

	&::-moz-range-thumb {
		width: 1.25rem;
		height: 1.25rem;
		border: 0;
		background: transparent;
	}

	&:focus-visible::-webkit-slider-thumb {
		box-shadow: none;
	}

	&:focus-visible::-moz-range-thumb {
		box-shadow: none;
	}

	&:disabled {
		pointer-events: none;
		opacity: 1;
	}

	&:focus-visible + .slider-track .slider-thumb {
		outline: 3px solid var(--color-focus-ring);
		outline-offset: 3px;
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

.filled-slider-track {
	transition: width 0.25s var(--ease-out-expo);
	@media (prefers-reduced-motion) {
		transition: none;
	}
}
</style>
