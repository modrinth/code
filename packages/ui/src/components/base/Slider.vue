<template>
	<div class="flex w-full items-center gap-4">
		<span
			v-if="currentValue !== null"
			class="shrink-0 whitespace-nowrap py-2 text-sm leading-5 text-secondary"
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
				:class="
					disabled
						? 'cursor-not-allowed'
						: currentValue === min
							? 'cursor-e-resize'
							: currentValue === max
								? 'cursor-w-resize'
								: 'cursor-ew-resize'
				"
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
				class="snap-points pointer-events-none absolute inset-x-0 top-1/2 h-[18px] -translate-y-1/2"
			>
				<span
					v-for="snapPoint in visibleSnapPoints"
					:key="snapPoint"
					class="absolute top-0 h-[18px] w-1.5 -translate-x-1/2 rounded-full"
					:class="snapPoint <= currentValue ? 'bg-brand brightness-on-hover' : 'bg-surface-5'"
					:style="{ left: `${getPercentage(snapPoint)}%` }"
				/>
			</div>
		</div>

		<span
			v-if="currentValue !== null"
			class="shrink-0 whitespace-nowrap py-2 text-sm leading-5 text-secondary"
		>
			{{ maxLabel ?? formatValue(max) }}
		</span>

		<Input
			:model-value="currentValue ?? undefined"
			type="number"
			:size="size"
			wrapper-class="slider-value shrink-0"
			:class="currentValue === null ? 'w-full' : undefined"
			:style="currentValue === null ? undefined : { '--value-chars': valueFieldChars }"
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
import { computed, ref, useTemplateRef, watch } from 'vue'

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
const valueFieldChars = computed(() => {
	const decimals = (String(props.step).split('.')[1] ?? '').length

	return Math.max(props.min.toFixed(decimals).length, props.max.toFixed(decimals).length, 2)
})
const input = useTemplateRef<HTMLInputElement>('input')
const currentValue = ref(props.modelValue === null ? null : normalizeValue(props.modelValue))
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
	const parsedValue = Number.parseFloat(value)

	let snappedValue = parsedValue
	let closestDistance = props.snapRange

	for (const snapPoint of props.snapPoints) {
		const distance = Math.abs(snapPoint - parsedValue)
		if (distance < closestDistance) {
			closestDistance = distance
			snappedValue = snapPoint
		}
	}

	inputValueValid(snappedValue)

	if (input.value && currentValue.value !== null) {
		input.value.value = String(currentValue.value)
	}
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

	&:hover,
	&:focus-visible {
		& + .slider-track .filled-slider-track,
		& ~ .snap-points .brightness-on-hover {
			filter: brightness(var(--hover-brightness));
		}
	}
}

.slider-value :deep(input[type='number']) {
	-moz-appearance: textfield;
	flex: none;
	width: calc(var(--value-chars) * 1ch);

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
