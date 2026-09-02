<template>
	<VerifyPayoutModal
		ref="verifyModal"
		:payout="payout"
		:creator-amount="creatorAmount"
		:amount-received="amountReceivedValue"
		:adjustments="validAdjustments"
		:submitting="submitting"
		@submit="startDistribution"
	/>

	<div class="grid gap-6 xl:grid-cols-[minmax(0,1fr)_minmax(24rem,42rem)]">
		<div class="flex flex-col gap-6">
			<div
				class="flex flex-col gap-4 rounded-2xl border border-solid border-surface-4 bg-surface-3 p-6"
			>
				<h2 class="m-0 text-lg font-semibold text-contrast">Aditude Payment</h2>
				<div class="flex flex-col gap-2.5">
					<span class="text-base font-semibold text-contrast">Amount Received</span>
					<StyledInput
						v-model="amountReceived"
						type="number"
						inputmode="decimal"
						placeholder="0.00"
						:step="0.01"
						wrapper-class="w-full"
					/>
				</div>
			</div>

			<AdjustmentsCard v-model="adjustments" />
		</div>

		<div class="flex flex-col gap-6">
			<DistributeBreakdownCard
				:payout="payout"
				:amount-received="amountReceived"
				:adjustments="validAdjustments"
			/>

			<ButtonStyled color="green" size="large">
				<button
					class="w-full"
					:disabled="!canOpenVerification || submitting"
					@click="openVerifyModal"
				>
					Reconcile & Distribute Earnings
					<ChevronRightIcon aria-hidden="true" />
				</button>
			</ButtonStyled>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ChevronRightIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	injectModrinthClient,
	injectNotificationManager,
	StyledInput,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import {
	type DistributionAdjustment,
	formatCurrency,
	getCreatorShare,
	getNetActualRevenue,
	roundCurrency,
} from '../utils'
import AdjustmentsCard from './AdjustmentsCard.vue'
import DistributeBreakdownCard from './DistributeBreakdownCard.vue'
import VerifyPayoutModal from './VerifyPayoutModal.vue'

defineProps<{
	payout: Labrinth.Payouts.Internal.HistoryItem
}>()

const client = injectModrinthClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()

const verifyModal = ref<InstanceType<typeof VerifyPayoutModal> | null>(null)
const amountReceived = ref<number | undefined>()
const adjustments = ref<DistributionAdjustment[]>([])
const submitting = ref(false)

const amountReceivedValue = computed(() => roundCurrency(Number(amountReceived.value ?? 0)))
const validAdjustments = computed(() =>
	adjustments.value
		.filter((adjustment) => adjustment.description.trim() || adjustment.amount !== 0)
		.map((adjustment) => ({
			description: adjustment.description.trim(),
			amount: roundCurrency(Number(adjustment.amount)),
		})),
)
const adjustmentsAreValid = computed(() =>
	validAdjustments.value.every(
		(adjustment) => adjustment.description.length > 0 && Number.isFinite(adjustment.amount),
	),
)
const netActualRevenue = computed(() =>
	getNetActualRevenue(amountReceivedValue.value, validAdjustments.value),
)
const creatorAmount = computed(() => getCreatorShare(netActualRevenue.value))
const canOpenVerification = computed(
	() => amountReceivedValue.value > 0 && adjustmentsAreValid.value && creatorAmount.value > 0,
)

function openVerifyModal() {
	if (!canOpenVerification.value) {
		return
	}

	verifyModal.value?.show()
}

async function startDistribution(request: Labrinth.Payouts.Internal.StartDistributionRequest) {
	if (submitting.value) {
		return
	}

	submitting.value = true

	try {
		await client.labrinth.payouts_internal.startDistribution(request)
		verifyModal.value?.hide()
		await Promise.all([
			queryClient.invalidateQueries({ queryKey: ['creator-payouts-history'] }),
			queryClient.invalidateQueries({ queryKey: ['creator-payouts-distribution'] }),
		])
		addNotification({
			title: 'Payout initiated',
			text: `${formatCurrency(creatorAmount.value, { cents: true })} will be distributed to creators.`,
			type: 'success',
		})
		await navigateTo('/admin/creator-payouts')
	} catch (error) {
		addNotification({
			title: 'Failed to initiate payout',
			text: error instanceof Error ? error.message : String(error),
			type: 'error',
		})
	} finally {
		submitting.value = false
	}
}
</script>
