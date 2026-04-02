<template>
	<NewModal ref="modal" max-width="550px">
		<template #title>
			<div class="text-2xl font-semibold text-contrast">Resubscribe to Server</div>
		</template>

		<div class="flex w-[44rem] max-w-full flex-col gap-6">
			<template v-if="modalData">
				<p class="m-0 text-secondary leading-relaxed">
					You are about to resubscribe to
					<span class="font-semibold text-contrast">{{ modalData.serverName }}</span
					>. Your subscription will be reactivated and your server will continue running without
					interruption.
				</p>

				<div v-if="formattedPrice" class="flex flex-col gap-2.5">
					<span class="text-contrast font-semibold">Plan</span>
					<div
						class="flex items-center justify-between gap-4 rounded-2xl border border-solid border-surface-5 bg-surface-2 p-5"
					>
						<div class="flex flex-col gap-1">
							<div class="truncate font-semibold text-contrast">{{ modalData.planName }}</div>
							<div
								v-if="modalData.ramGb != null || modalData.storageGb != null || modalData.sharedCpus != null"
								class="text-secondary flex gap-1.5 font-medium text-sm items-center"
							>
								<template v-if="modalData.ramGb != null">
									{{ modalData.ramGb }} GB RAM
								</template>
								<template v-if="modalData.storageGb != null">
									<div v-if="modalData.ramGb != null" class="h-1.5 w-1.5 bg-button-border rounded-full"></div>
									{{ modalData.storageGb }} GB Storage
								</template>
								<template v-if="modalData.sharedCpus != null">
									<div v-if="modalData.ramGb != null || modalData.storageGb != null" class="h-1.5 w-1.5 bg-button-border rounded-full"></div>
									{{ modalData.sharedCpus }} Shared CPUs
								</template>
							</div>
						</div>
						<div class="flex flex-col gap-1 items-end">
							<div class="font-semibold text-contrast">
								{{ formattedPrice }}
							</div>
							<div v-if="intervalLabel" class="text-secondary">/{{ intervalLabel }}</div>
						</div>
					</div>
				</div>

				<p v-if="formattedNextChargeDate" class="m-0 text-primary">
					Your next charge will be on
					<span class="font-semibold text-contrast">{{ formattedNextChargeDate }}</span
					>.
				</p>
			</template>
			<template v-else>
				<p class="m-0 text-secondary">Failed to load subscription details.</p>
			</template>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled type="outlined">
					<button class="!border !border-surface-5" @click="handleCancel">
						<XIcon />
						Cancel
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button :disabled="!canResubscribe" @click="handleResubscribe">
						<RotateCounterClockwiseIcon />
						Resubscribe
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { RotateCounterClockwiseIcon, XIcon } from '@modrinth/assets'
import { computed, ref, useTemplateRef } from 'vue'

import { injectNotificationManager } from '#ui/providers/web-notifications.ts'
import { useFormatDateTime, useFormatPrice } from '../../composables'
import { ButtonStyled, NewModal } from '../index'

const { addNotification } = injectNotificationManager()

type BillingInterval = Labrinth.Billing.Internal.PriceDuration

export type ResubscribeModalPayload = {
	subscriptionId: string
	wasSuspended: boolean
	serverName: string
	planName: string
	ramGb?: number
	storageGb?: number
	sharedCpus?: number
	priceCents?: number
	currencyCode?: string
	interval: BillingInterval
	nextChargeDate?: string | number | Date
}

type ResubscribeModalState = {
	subscriptionId: string
	wasSuspended: boolean
	serverName: string
	planName: string
	ramGb?: number
	storageGb?: number
	sharedCpus?: number
	priceCents?: number
	currencyCode?: string
	interval: BillingInterval
	nextChargeDate?: string | number | Date
}

const emit = defineEmits<{
	(e: 'cancel'): void
	(e: 'resubscribe', payload: { subscriptionId: string; wasSuspended: boolean }): void
}>()

const formatDate = useFormatDateTime({ dateStyle: 'long' })
const formatPrice = useFormatPrice()

const modal = useTemplateRef<InstanceType<typeof NewModal>>('modal')

const modalData = ref<ResubscribeModalState | null>(null)

const canResubscribe = computed(() => !!modalData.value?.subscriptionId)

const intervalLabel = computed(() => {
	switch (modalData.value?.interval) {
		case 'monthly':
			return 'month'
		case 'quarterly':
			return 'quarter'
		case 'yearly':
			return 'year'
		case 'five-days':
			return '5 days'
		default:
			return null
	}
})

const formattedPrice = computed(() => {
	const { priceCents, currencyCode } = modalData.value ?? {}
	if (priceCents == null || currencyCode == null) return ''
	return formatPrice(priceCents, currencyCode)
})

const normalizedNextChargeDate = computed(() => {
	if (!modalData.value?.nextChargeDate) return null
	const date = new Date(modalData.value.nextChargeDate)
	if (Number.isNaN(date.getTime())) {
		return null
	}
	return date
})

const formattedNextChargeDate = computed(() =>
	normalizedNextChargeDate.value ? formatDate(normalizedNextChargeDate.value) : '',
)

function show(payload: ResubscribeModalPayload) {
	if (!payload) {
		addNotification({
			type: 'error',
			title: 'Error',
			text: 'Cannot resubscribe, failed to load subscription details.',
		})
		return
	}

	modalData.value = {
		subscriptionId: payload.subscriptionId,
		wasSuspended: payload.wasSuspended,
		serverName: payload.serverName.trim(),
		planName: payload.planName,
		ramGb: payload.ramGb,
		storageGb: payload.storageGb,
		sharedCpus: payload.sharedCpus,
		priceCents: payload.priceCents,
		currencyCode: payload.currencyCode,
		interval: payload.interval,
		nextChargeDate: payload.nextChargeDate,
	}
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

function handleCancel() {
	hide()
	emit('cancel')
}

function handleResubscribe() {
	if (!canResubscribe.value || !modalData.value?.subscriptionId) return
	hide()

	emit('resubscribe', {
		subscriptionId: modalData.value.subscriptionId,
		wasSuspended: modalData.value.wasSuspended,
	})
}

defineExpose({
	show,
	hide,
})
</script>
