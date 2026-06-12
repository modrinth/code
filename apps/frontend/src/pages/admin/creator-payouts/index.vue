<template>
	<div class="normal-page no-sidebar !mb-20">
		<div class="normal-page__content flex flex-col gap-6">
			<h1 class="m-0 mt-6 text-3xl font-semibold text-contrast">Creator Payouts</h1>

			<PayoutInitiatedCard
				v-if="activeDistribution"
				:distribution="activeDistribution"
				:cancelling="cancellingDistribution"
				@cancel="cancelDistribution"
			/>

			<div class="grid gap-5 empty:hidden lg:grid-cols-3">
				<DistributeMonthCard v-if="reviewPayout && !activeDistribution" :payout="reviewPayout" />
				<DistributeMonthCard
					v-for="payout in pendingPayouts"
					:key="payout.payouts_date"
					:payout="payout"
				/>
			</div>

			<PayoutsTable :payouts="payoutHistory ?? []" />
		</div>
	</div>
</template>

<script setup lang="ts">
import { injectModrinthClient, injectNotificationManager } from '@modrinth/ui'
import { isAdmin } from '@modrinth/utils'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

import DistributeMonthCard from '~/components/ui/creator-payouts/distribute-month-card/index.vue'
import PayoutInitiatedCard from '~/components/ui/creator-payouts/payout-initiated-card/index.vue'
import PayoutsTable from '~/components/ui/creator-payouts/payouts-table/index.vue'

definePageMeta({
	middleware: [
		'auth',
		async () => {
			const auth = await useAuth()

			if (!auth.value.user || !isAdmin(auth.value.user)) {
				throw createError({
					fatal: true,
					statusCode: 401,
					statusMessage: 'Unauthorized',
				})
			}
		},
	],
})

const client = injectModrinthClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()
const cancellingDistribution = ref(false)

const { data: payoutHistory, error: historyError } = useQuery({
	queryKey: ['creator-payouts-history'],
	queryFn: () => client.labrinth.payouts_internal.getHistory(),
	placeholderData: [],
	retry: false,
})

const { data: activeDistribution, error: distributionError } = useQuery({
	queryKey: ['creator-payouts-distribution'],
	queryFn: () => client.labrinth.payouts_internal.getDistribution(),
	retry: false,
})

const sortedPayouts = computed(() =>
	[...(payoutHistory.value ?? [])].sort((left, right) =>
		right.payouts_date.localeCompare(left.payouts_date),
	),
)
const reviewPayout = computed(() =>
	sortedPayouts.value.find((payout) => payout.status === 'review'),
)
const pendingPayouts = computed(() =>
	sortedPayouts.value.filter((payout) => payout.status === 'pending').slice(0, 2),
)

watch(historyError, (error) => {
	if (!error) {
		return
	}

	addNotification({
		title: 'Failed to load creator payouts',
		text: error.message,
		type: 'error',
	})
})

watch(distributionError, (error) => {
	if (!error) {
		return
	}

	addNotification({
		title: 'Failed to load active payout',
		text: error.message,
		type: 'error',
	})
})

async function cancelDistribution() {
	if (cancellingDistribution.value) {
		return
	}

	cancellingDistribution.value = true

	try {
		await client.labrinth.payouts_internal.cancelDistribution()
		await Promise.all([
			queryClient.invalidateQueries({ queryKey: ['creator-payouts-history'] }),
			queryClient.invalidateQueries({ queryKey: ['creator-payouts-distribution'] }),
		])
		addNotification({
			title: 'Payout cancelled',
			type: 'success',
		})
	} catch (error) {
		addNotification({
			title: 'Failed to cancel payout',
			text: error instanceof Error ? error.message : String(error),
			type: 'error',
		})
	} finally {
		cancellingDistribution.value = false
	}
}
</script>
