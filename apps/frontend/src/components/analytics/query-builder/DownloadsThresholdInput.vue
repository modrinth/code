<template>
	<div class="flex flex-wrap items-center gap-3">
		<span class="shrink-0 whitespace-nowrap text-sm font-semibold text-primary">
			{{ props.label }}
		</span>
		<input
			v-model="inputValue"
			type="text"
			inputmode="numeric"
			placeholder="0"
			class="h-8 rounded-lg border border-solid border-surface-5 bg-surface-3 px-2 text-center text-sm font-semibold text-primary outline-none transition-[box-shadow,color] focus:text-contrast focus:ring-4 focus:ring-brand-shadow"
			:class="props.inputWidthClass"
			:aria-label="props.inputAriaLabel"
			@blur="formatInput"
		/>
		<span class="shrink-0 text-sm font-semibold text-primary">{{ props.suffix }}</span>
	</div>
</template>

<script setup lang="ts">
const props = withDefaults(
	defineProps<{
		label: string
		inputAriaLabel: string
		threshold?: number | null
		suffix?: string
		inputWidthClass?: string
	}>(),
	{
		suffix: 'downloads',
		inputWidthClass: 'w-20',
	},
)

const emit = defineEmits<{
	'update:threshold': [threshold: number | null]
}>()

const inputValue = ref('')
let isSyncingThreshold = false
let hasPendingEmittedThreshold = false
let pendingEmittedThreshold: number | null = null

function parseDownloadsThreshold(value: string): number | null | undefined {
	const normalizedValue = value.trim().toLowerCase().replace(/,/g, '')
	if (!normalizedValue) {
		return null
	}

	const match = normalizedValue.match(/^(\d+(?:\.\d+)?)([kmb])?$/)
	if (!match) {
		return undefined
	}

	const amount = Number.parseFloat(match[1])
	if (!Number.isFinite(amount)) {
		return undefined
	}

	const multiplierBySuffix: Record<string, number> = {
		k: 1_000,
		m: 1_000_000,
		b: 1_000_000_000,
	}

	const multiplier = match[2] ? multiplierBySuffix[match[2]] : 1
	return Math.max(0, Math.floor(amount * multiplier))
}

function formatCompactNumber(value: number): string {
	const formatWithSuffix = (divisor: number, suffix: string) => {
		const dividedValue = value / divisor
		const fractionDigits = Number.isInteger(dividedValue) ? 0 : 1
		return `${dividedValue.toFixed(fractionDigits).replace(/\.0$/, '')}${suffix}`
	}

	if (value >= 1_000_000_000) return formatWithSuffix(1_000_000_000, 'B')
	if (value >= 1_000_000) return formatWithSuffix(1_000_000, 'M')
	if (value >= 1_000) return formatWithSuffix(1_000, 'k')
	return String(value)
}

function formatInput() {
	const threshold = parseDownloadsThreshold(inputValue.value)
	if (threshold === undefined || threshold === null) {
		return
	}

	inputValue.value = formatCompactNumber(threshold)
}

watch(inputValue, (value) => {
	if (isSyncingThreshold) {
		return
	}

	const threshold = parseDownloadsThreshold(value)
	if (threshold === undefined) {
		return
	}

	hasPendingEmittedThreshold = true
	pendingEmittedThreshold = threshold
	emit('update:threshold', threshold)
	nextTick(() => {
		if (hasPendingEmittedThreshold && pendingEmittedThreshold === threshold) {
			hasPendingEmittedThreshold = false
		}
	})
})

watch(
	() => props.threshold,
	(threshold) => {
		if (hasPendingEmittedThreshold && threshold === pendingEmittedThreshold) {
			hasPendingEmittedThreshold = false
			return
		}

		isSyncingThreshold = true
		inputValue.value =
			threshold === null || threshold === undefined ? '' : formatCompactNumber(threshold)
		nextTick(() => {
			isSyncingThreshold = false
		})
	},
	{ immediate: true },
)
</script>
