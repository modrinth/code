<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { computed, ref, useTemplateRef } from 'vue'

import { ChevronRightIcon } from '@modrinth/assets'
import NewModal from '../modal/NewModal.vue'
import type { ServerBillingInterval } from './ModrinthServersPurchaseModal.vue'
import PlanSelector from './ServersPurchase0Plan.vue'

const props = defineProps<{
	availableProducts: Labrinth.Billing.Internal.Product[]
	currency: string
}>()

const emit = defineEmits<{
	(e: 'continue', payload: { interval: ServerBillingInterval; planId: string | null }): void
}>()

const modal = useTemplateRef<InstanceType<typeof NewModal>>('modal')

const selectedPlan = ref<Labrinth.Billing.Internal.Product>()
const selectedInterval = ref<ServerBillingInterval>('quarterly')

const defaultPlan = computed<Labrinth.Billing.Internal.Product | undefined>(() => {
	return (
		props.availableProducts.find((p) => p?.metadata?.type === 'pyro' && p.metadata.ram === 6144) ??
		props.availableProducts.find((p) => p?.metadata?.type === 'pyro') ??
		props.availableProducts[0]
	)
})

function continueWithSelection() {
	emit('continue', {
		interval: selectedInterval.value,
		planId: selectedPlan.value?.id ?? null,
	})
	modal.value?.hide()
}

function chooseCustom() {
	selectedPlan.value = undefined
}

function show(initialInterval?: ServerBillingInterval, initialPlanId?: string | null) {
	selectedInterval.value = initialInterval ?? 'quarterly'
	if (initialPlanId === null) {
		selectedPlan.value = undefined
	} else if (initialPlanId) {
		selectedPlan.value = props.availableProducts.find((product) => product.id === initialPlanId)
	} else {
		selectedPlan.value = defaultPlan.value
	}
	modal.value?.show()
}

defineExpose({
	show,
})
</script>

<template>
	<NewModal ref="modal">
		<template #title>
			<div class="flex items-center gap-1 font-bold text-secondary">
				<span class="text-contrast">Plan</span>
				<ChevronRightIcon class="h-5 w-5 text-secondary" stroke-width="3" />
				<span class=""> Region </span>
				<ChevronRightIcon class="h-5 w-5 text-secondary" stroke-width="3" />
				<span class=""> Payment method </span>
				<ChevronRightIcon class="h-5 w-5 text-secondary" stroke-width="3" />
				<span class=""> Review </span>
			</div>
		</template>
		<div class="w-[56rem] max-w-full">
			<PlanSelector
				v-model:plan="selectedPlan"
				v-model:interval="selectedInterval"
				:available-products="availableProducts"
				:currency="currency"
				@choose-custom="chooseCustom"
				@proceed="continueWithSelection"
			/>
		</div>
	</NewModal>
</template>
