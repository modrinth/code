<template>
	<ConfirmModal
		ref="deleteAdjustmentModal"
		title="Delete adjustment?"
		:description="deleteAdjustmentDescription"
		proceed-label="Delete"
		:markdown="false"
		width="36rem"
		@proceed="confirmRemoveAdjustment"
	/>

	<div
		class="flex flex-col gap-5 rounded-2xl border border-solid border-surface-4 bg-surface-3 p-6"
	>
		<h2 class="m-0 text-lg font-semibold text-contrast">Adjustments</h2>

		<div v-if="adjustments.length > 0" class="flex flex-col gap-2.5">
			<div
				v-for="(adjustment, index) in adjustments"
				:key="index"
				class="grid grid-cols-[minmax(0,1fr)_6.5rem_auto] gap-1.5"
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
					<button
						:aria-label="`Remove adjustment ${index + 1}`"
						@click="openDeleteAdjustmentModal(index)"
					>
						<TrashIcon aria-hidden="true" />
					</button>
				</ButtonStyled>
			</div>
		</div>

		<ButtonStyled type="outlined">
			<button class="w-full" @click="addAdjustment">
				<PlusIcon aria-hidden="true" />
				Add Adjustment
			</button>
		</ButtonStyled>
	</div>
</template>

<script setup lang="ts">
import { PlusIcon, TrashIcon } from '@modrinth/assets'
import { ButtonStyled, ConfirmModal, StyledInput } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { formatCurrency, type DistributionAdjustment } from '../utils'

const adjustments = defineModel<DistributionAdjustment[]>({ required: true })
const pendingDeleteIndex = ref<number | null>(null)
const deleteAdjustmentModal = ref<InstanceType<typeof ConfirmModal> | null>(null)

const pendingDeleteAdjustment = computed(() =>
	pendingDeleteIndex.value === null ? null : adjustments.value[pendingDeleteIndex.value],
)
const deleteAdjustmentDescription = computed(() => {
	const adjustment = pendingDeleteAdjustment.value

	if (!adjustment) {
		return ''
	}

	if (!adjustment.description) {
		return `Delete adjustment for ${formatAdjustmentAmount(adjustment.amount)}`
	}

	return `Delete adjustment "${adjustment.description}" for ${formatAdjustmentAmount(adjustment.amount)}`
})

function addAdjustment() {
	adjustments.value = [...adjustments.value, { description: '', amount: 0 }]
}

function openDeleteAdjustmentModal(index: number) {
	pendingDeleteIndex.value = index
	deleteAdjustmentModal.value?.show()
}

function confirmRemoveAdjustment() {
	if (pendingDeleteIndex.value === null) {
		return
	}

	adjustments.value = adjustments.value.filter(
		(_, adjustmentIndex) => adjustmentIndex !== pendingDeleteIndex.value,
	)
	pendingDeleteIndex.value = null
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

function formatAdjustmentAmount(amount: number): string {
	const formatted = formatCurrency(Math.abs(amount), { cents: true })
	return amount < 0 ? `-${formatted}` : formatted
}
</script>
