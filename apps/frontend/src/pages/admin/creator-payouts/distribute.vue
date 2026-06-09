<template>
	<div v-if="selectedPayout" class="normal-page no-sidebar !mb-20">
		<div class="normal-page__content flex flex-col gap-8">
			<NuxtLink
				to="/admin/creator-payouts"
				class="mt-6 inline-flex w-fit items-center gap-2 text-base font-medium text-secondary hover:text-contrast"
			>
				<ArrowLeftIcon class="size-5" aria-hidden="true" />
				Back to Overview
			</NuxtLink>

			<h1 class="m-0 text-3xl font-semibold text-contrast">
				{{ formatMonthYear(selectedPayout.payouts_date) }} Earnings
			</h1>

			<DistributeEarnings :payout="selectedPayout" />
		</div>
	</div>
</template>

<script setup lang="ts">
import { ArrowLeftIcon } from '@modrinth/assets'
import { injectModrinthClient, injectNotificationManager } from '@modrinth/ui'
import { isAdmin } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import { computed, watch } from 'vue'

import DistributeEarnings from '~/components/ui/creator-payouts/distribute-earnings/index.vue'
import { formatMonthYear, isYearMonth } from '~/components/ui/creator-payouts/utils'

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

const route = useRoute()
const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const requestedPayoutDate = computed(() => route.query.payouts_date)

const { data: payoutHistory, error } = useQuery({
	queryKey: ['creator-payouts-history'],
	queryFn: () => client.labrinth.payouts_internal.getHistory(),
	retry: false,
})

const requestedPayout = computed(() => {
	if (!isYearMonth(requestedPayoutDate.value)) {
		return null
	}

	return (
		payoutHistory.value?.find((payout) => payout.payouts_date === requestedPayoutDate.value) ?? null
	)
})
const selectedPayout = computed(() =>
	requestedPayout.value?.status === 'review' ? requestedPayout.value : null,
)

watch(
	[payoutHistory, error],
	async () => {
		if (error.value) {
			addNotification({
				title: 'Invalid month to distribute',
				text: error.value.message,
				type: 'error',
			})
			await navigateTo('/admin/creator-payouts')
			return
		}

		if (!payoutHistory.value) {
			return
		}

		if (!isYearMonth(requestedPayoutDate.value) || !requestedPayout.value) {
			addNotification({
				title: 'Invalid month to distribute',
				type: 'error',
			})
			await navigateTo('/admin/creator-payouts')
			return
		}

		if (requestedPayout.value.status !== 'review') {
			addNotification({
				title: 'Invalid month to distribute',
				text: 'That payout period is not in review.',
				type: 'error',
			})
			await navigateTo('/admin/creator-payouts')
		}
	},
	{ immediate: true },
)
</script>
