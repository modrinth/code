<template>
	<div class="rounded-2xl border border-solid border-surface-4 bg-surface-2 p-7">
		<h2 class="m-0 text-lg font-semibold text-contrast">Adjustments</h2>

		<div v-if="adjustments.length > 0" class="mt-5 flex flex-col gap-2.5">
			<div
				v-for="(adjustment, index) in adjustments"
				:key="index"
				class="grid grid-cols-[minmax(0,1fr)_11rem_auto] gap-1.5"
			>
				<StyledInput
					:model-value="adjustment.description"
					placeholder="Description"
					autocomplete="off"
					wrapper-class="w-full"
					@update:model-value="updateAdjustment(index, 'description', String($event ?? ''))"
				/>
				<StyledInput
					:model-value="adjustment.amount"
					type="number"
					inputmode="decimal"
					placeholder="0.00"
					:step="0.01"
					wrapper-class="w-full"
					@update:model-value="updateAdjustment(index, 'amount', Number($event ?? 0))"
				/>
				<ButtonStyled circular type="outlined" color="red">
					<button :aria-label="`Remove adjustment ${index + 1}`" @click="removeAdjustment(index)">
						<TrashIcon aria-hidden="true" />
					</button>
				</ButtonStyled>
			</div>
		</div>

		<button
			class="mt-5 flex h-14 w-full items-center justify-center gap-3 rounded-xl border-2 border-dashed border-surface-4 bg-transparent text-base font-semibold text-secondary transition-colors hover:text-contrast"
			@click="addAdjustment"
		>
			<PlusIcon class="size-5" aria-hidden="true" />
			Add Adjustment
		</button>
	</div>
</template>

<script setup lang="ts">
import { PlusIcon, TrashIcon } from '@modrinth/assets'
import { ButtonStyled, StyledInput } from '@modrinth/ui'

import type { DistributionAdjustment } from '../utils'

const adjustments = defineModel<DistributionAdjustment[]>({ required: true })

function addAdjustment() {
	adjustments.value = [...adjustments.value, { description: '', amount: 0 }]
}

function removeAdjustment(index: number) {
	adjustments.value = adjustments.value.filter((_, adjustmentIndex) => adjustmentIndex !== index)
}

function updateAdjustment(
	index: number,
	field: keyof DistributionAdjustment,
	value: string | number,
) {
	adjustments.value = adjustments.value.map((adjustment, adjustmentIndex) =>
		adjustmentIndex === index ? { ...adjustment, [field]: value } : adjustment,
	)
}
</script>
