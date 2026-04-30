<template>
	<div class="flex items-center gap-3 px-4 py-3">
		<span class="shrink-0 text-sm font-semibold text-primary">In the last</span>
		<div
			class="flex h-8 shrink-0 items-center overflow-hidden rounded-lg border border-solid border-surface-5 bg-surface-3"
		>
			<button
				type="button"
				class="flex h-8 w-8 cursor-pointer items-center justify-center border-0 border-r border-solid border-surface-5 bg-transparent p-0 text-secondary transition-colors hover:text-contrast"
				aria-label="Decrease timeframe amount"
				@click.stop="decrementAmount"
			>
				<MinusIcon class="size-4" />
			</button>
			<input
				v-model.number="amount"
				type="number"
				min="1"
				step="1"
				class="h-8 w-12 border-0 bg-transparent px-1 text-center text-sm font-semibold text-primary outline-none"
				aria-label="Timeframe amount"
				@focus="activate"
				@input="sanitizeAmount"
				@blur="sanitizeAmount"
			/>
			<button
				type="button"
				class="flex h-8 w-8 cursor-pointer items-center justify-center border-0 border-l border-solid border-surface-5 bg-transparent p-0 text-secondary transition-colors hover:text-contrast"
				aria-label="Increase timeframe amount"
				@click.stop="incrementAmount"
			>
				<PlusIcon class="size-4" />
			</button>
		</div>
		<select
			v-model="unit"
			class="h-8 rounded-lg border border-solid border-surface-5 bg-surface-3 px-2 text-sm font-semibold text-primary outline-none transition-[box-shadow,color] focus:text-contrast focus:ring-4 focus:ring-brand-shadow"
			aria-label="Timeframe unit"
			@change="activate"
		>
			<option v-for="option in unitOptions" :key="option.value" :value="option.value">
				{{ option.label }}
			</option>
		</select>
	</div>
</template>

<script setup lang="ts">
import { MinusIcon, PlusIcon } from '@modrinth/assets'

import type { AnalyticsLastTimeframeUnit } from '~/providers/analytics/analytics'

defineProps<{
	unitOptions: Array<{
		value: AnalyticsLastTimeframeUnit
		label: string
	}>
}>()

const amount = defineModel<number>('amount', { required: true })
const unit = defineModel<AnalyticsLastTimeframeUnit>('unit', { required: true })

const emit = defineEmits<{
	activate: []
}>()

function activate() {
	emit('activate')
}

function sanitizeAmount() {
	const nextAmount = Number(amount.value)
	amount.value = Number.isFinite(nextAmount) ? Math.max(1, Math.floor(nextAmount)) : 1
	activate()
}

function incrementAmount() {
	sanitizeAmount()
	amount.value += 1
	activate()
}

function decrementAmount() {
	sanitizeAmount()
	amount.value = Math.max(1, amount.value - 1)
	activate()
}
</script>
