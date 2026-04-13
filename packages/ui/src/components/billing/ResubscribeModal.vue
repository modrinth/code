<template>
	<NewModal ref="modal" max-width="550px">
		<template #title>
			<div class="text-2xl font-semibold text-contrast">{{ formatMessage(messages.title) }}</div>
		</template>

		<div class="flex w-[44rem] max-w-full flex-col gap-6">
			<template v-if="modalData">
				<p class="m-0 text-secondary leading-relaxed">
					<IntlFormatted
						:message-id="messages.description"
						:values="{ serverName: modalData.serverName }"
					>
						<template #server-name="{ children }">
							<span class="font-semibold text-contrast"><component :is="() => children" /></span>
						</template>
					</IntlFormatted>
				</p>

				<div v-if="formattedPrice" class="flex flex-col gap-2.5">
					<span class="text-contrast font-semibold">{{ formatMessage(messages.planLabel) }}</span>
					<div
						class="flex items-center justify-between gap-4 rounded-2xl border border-solid border-surface-5 bg-surface-2 p-5"
					>
						<div class="flex flex-col gap-1">
							<div class="truncate font-semibold text-contrast">{{ modalData.planName }}</div>
							<div
								v-if="
									modalData.ramGb != null ||
									modalData.storageGb != null ||
									modalData.sharedCpus != null
								"
								class="text-secondary flex gap-1.5 font-medium text-sm items-center"
							>
								<template v-if="modalData.ramGb != null">
									{{ formatMessage(messages.ramLabel, { ramGb: modalData.ramGb }) }}
								</template>
								<template v-if="modalData.storageGb != null">
									<div
										v-if="modalData.ramGb != null"
										class="h-1.5 w-1.5 bg-button-border rounded-full"
									></div>
									{{ formatMessage(messages.storageLabel, { storageGb: modalData.storageGb }) }}
								</template>
								<template v-if="modalData.sharedCpus != null">
									<div
										v-if="modalData.ramGb != null || modalData.storageGb != null"
										class="h-1.5 w-1.5 bg-button-border rounded-full"
									></div>
									{{ formatMessage(messages.cpusLabel, { sharedCpus: modalData.sharedCpus }) }}
								</template>
							</div>
						</div>
						<div class="flex flex-col gap-1 items-end">
							<div class="font-semibold text-contrast">
								{{ formattedPrice }}
							</div>
							<div v-if="intervalLabel" class="text-secondary">{{ intervalLabel }}</div>
						</div>
					</div>
				</div>

				<p v-if="formattedNextChargeDate" class="m-0 text-primary">
					<IntlFormatted
						:message-id="messages.nextCharge"
						:values="{ date: formattedNextChargeDate }"
					>
						<template #charge-date="{ children }">
							<span class="font-semibold text-contrast"><component :is="() => children" /></span>
						</template>
					</IntlFormatted>
				</p>
			</template>
			<template v-else>
				<p class="m-0 text-secondary">{{ formatMessage(messages.failedLoad) }}</p>
			</template>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled type="outlined">
					<button class="!border !border-surface-5" @click="handleCancel">
						<XIcon />
						{{ formatMessage(messages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button :disabled="!canResubscribe" @click="handleResubscribe">
						<RotateCounterClockwiseIcon />
						{{ formatMessage(messages.resubscribeButton) }}
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
import { defineMessages, useVIntl } from '../../composables/i18n'
import IntlFormatted from '../base/IntlFormatted.vue'
import { ButtonStyled, NewModal } from '../index'

const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

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

const messages = defineMessages({
	title: { id: 'billing.resubscribe-modal.title', defaultMessage: 'Resubscribe to Server' },
	description: {
		id: 'billing.resubscribe-modal.description',
		defaultMessage:
			'You are about to resubscribe to <server-name>{serverName}</server-name>. Your subscription will be reactivated and your server will continue running without interruption.',
	},
	planLabel: { id: 'billing.resubscribe-modal.plan-label', defaultMessage: 'Plan' },
	ramLabel: { id: 'billing.resubscribe-modal.ram', defaultMessage: '{ramGb} GB RAM' },
	storageLabel: {
		id: 'billing.resubscribe-modal.storage',
		defaultMessage: '{storageGb} GB Storage',
	},
	cpusLabel: {
		id: 'billing.resubscribe-modal.cpus',
		defaultMessage: '{sharedCpus} Shared CPUs',
	},
	nextCharge: {
		id: 'billing.resubscribe-modal.next-charge',
		defaultMessage: 'Your next charge will be on <charge-date>{date}</charge-date>.',
	},
	failedLoad: {
		id: 'billing.resubscribe-modal.failed-load',
		defaultMessage: 'Failed to load subscription details.',
	},
	cancelButton: { id: 'billing.resubscribe-modal.cancel', defaultMessage: 'Cancel' },
	resubscribeButton: {
		id: 'billing.resubscribe-modal.resubscribe',
		defaultMessage: 'Resubscribe',
	},
	intervalMonthly: { id: 'billing.resubscribe-modal.interval.monthly', defaultMessage: '/month' },
	intervalQuarterly: {
		id: 'billing.resubscribe-modal.interval.quarterly',
		defaultMessage: '/quarter',
	},
	intervalYearly: { id: 'billing.resubscribe-modal.interval.yearly', defaultMessage: '/year' },
	intervalFiveDays: {
		id: 'billing.resubscribe-modal.interval.five-days',
		defaultMessage: '/5 days',
	},
	errorTitle: { id: 'billing.resubscribe-modal.error.title', defaultMessage: 'Error' },
	errorText: {
		id: 'billing.resubscribe-modal.error.text',
		defaultMessage: 'Cannot resubscribe, failed to load subscription details.',
	},
})

const canResubscribe = computed(() => !!modalData.value?.subscriptionId)

const intervalLabel = computed(() => {
	switch (modalData.value?.interval) {
		case 'monthly':
			return formatMessage(messages.intervalMonthly)
		case 'quarterly':
			return formatMessage(messages.intervalQuarterly)
		case 'yearly':
			return formatMessage(messages.intervalYearly)
		case 'five-days':
			return formatMessage(messages.intervalFiveDays)
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
			title: formatMessage(messages.errorTitle),
			text: formatMessage(messages.errorText),
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
