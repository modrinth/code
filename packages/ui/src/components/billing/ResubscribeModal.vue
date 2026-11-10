<template>
	<NewModal ref="modal" @hide="handleHide">
		<template #title>
			<div class="text-2xl font-semibold text-contrast">{{ formatMessage(messages.title) }}</div>
		</template>

		<div class="flex w-[40rem] max-w-full flex-col">
			<template v-if="modalData">
				<p class="m-0 mb-4 text-secondary leading-relaxed">
					<IntlFormatted
						:message-id="messages.description"
						:values="{ serverName: modalData.serverName }"
					>
						<template #server-name="{ children }">
							<span class="font-semibold text-contrast"><component :is="() => children" /></span>
						</template>
					</IntlFormatted>
				</p>

				<div class="grid gap-4" :class="{ 'sm:grid-cols-[3fr_2fr]': region }">
					<div class="bg-surface-2 p-4 rounded-2xl">
						<div class="flex items-center gap-2 mb-3">
							<ModrinthServersIcon class="flex h-5 w-fit" />
							<TagItem>{{ planName }}</TagItem>
						</div>
						<ServersSpecs
							v-if="planSpecs"
							class="!grid sm:grid-cols-2"
							:ram="planSpecs.ram"
							:storage="planSpecs.storage"
							:cpus="planSpecs.cpu"
						/>
					</div>
					<div
						v-if="region"
						class="bg-surface-2 p-4 rounded-2xl flex flex-col gap-2 items-center justify-center"
					>
						<img
							v-if="regionFlag"
							class="aspect-[16/10] max-w-12 w-full object-cover rounded-md border-1 border-button-border border-solid"
							:src="regionFlag"
							alt=""
							aria-hidden="true"
						/>
						<span class="font-semibold">{{ regionTitle }}</span>
						<span v-if="regionSubtitle" class="text-xs text-secondary font-medium">
							{{ regionSubtitle }}
						</span>
					</div>
				</div>

				<div
					v-if="previewError"
					class="mt-4 p-4 rounded-2xl bg-surface-2 text-sm text-red leading-relaxed"
				>
					{{ formatMessage(messages.previewError) }}
				</div>
				<ExpandableInvoiceTotal
					v-else
					class="mt-4"
					:period="period"
					:currency="currency"
					:loading="loadingPreview"
					:total="preview?.total ?? -1"
					:billing-items="
						preview
							? [
									{
										title: formatMessage(messages.planItem, { plan: planName }),
										amount: preview.subtotal,
									},
									{ title: formatMessage(messages.taxItem), amount: preview.tax },
								]
							: []
					"
				/>

				<div class="mt-2 flex items-center pl-4 pr-2 py-3 bg-bg rounded-2xl gap-2 text-secondary">
					<template v-if="loadingPreview && !paymentMethod">
						<SpinnerIcon class="animate-spin" />
					</template>
					<FormattedPaymentMethod v-else-if="paymentMethod" :method="paymentMethod" />
					<div v-else class="flex items-center gap-2 text-red">
						<XIcon />
						{{ formatMessage(messages.noPaymentMethod) }}
					</div>
					<Button
						v-if="savedPaymentMethods.length > 0 || onAddPaymentMethod"
						type="quiet"
						size="xs"
						class="ml-auto !h-6"
						@click="choosingPaymentMethod = !choosingPaymentMethod"
					>
						<EditIcon /> {{ formatMessage(messages.changePaymentMethod) }}
					</Button>
				</div>

				<div
					v-if="choosingPaymentMethod"
					class="mt-2 flex flex-col gap-1 p-2 rounded-2xl bg-surface-2"
				>
					<PaymentMethodOption
						v-for="method in savedPaymentMethods"
						:key="method.id"
						:item="method"
						:selected="method.id === paymentMethod?.id"
						:loading="method.id === settingPaymentMethodId"
						@select="selectPaymentMethod(method)"
					/>
					<Button v-if="onAddPaymentMethod" type="quiet" @click="addPaymentMethod">
						<PlusIcon /> {{ formatMessage(messages.addPaymentMethod) }}
					</Button>
				</div>

				<p v-if="preview" class="m-0 mt-4 text-sm text-secondary leading-relaxed">
					<template v-if="chargedNow">
						{{
							formatMessage(messages.chargedNow, {
								total: formatPrice(preview.total, currency),
								period,
							})
						}}
					</template>
					<template v-else>
						<IntlFormatted
							:message-id="messages.chargedOnDate"
							:values="{
								total: formatPrice(preview.total, currency),
								period,
								date: formattedDueDate,
							}"
						>
							<template #charge-date="{ children }">
								<span class="font-semibold text-contrast"><component :is="() => children" /></span>
							</template>
						</IntlFormatted>
					</template>
					<br />
					<span class="font-semibold">{{ formatMessage(messages.recurringNotice) }}</span>
				</p>
			</template>
			<template v-else>
				<p class="m-0 text-secondary">{{ formatMessage(messages.failedLoad) }}</p>
			</template>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="outlined" @click="handleCancel">
					<XIcon />
					{{ formatMessage(messages.cancelButton) }}
				</Button>
				<Button type="colored" color="brand" :disabled="!canResubscribe" @click="handleResubscribe">
					<RotateCounterClockwiseIcon />
					{{ formatMessage(messages.resubscribeButton) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import type { Archon, Labrinth } from '@modrinth/api-client'
import {
	EditIcon,
	PlusIcon,
	RotateCounterClockwiseIcon,
	SpinnerIcon,
	XIcon,
} from '@modrinth/assets'
import { useQueryClient } from '@tanstack/vue-query'
import type Stripe from 'stripe'
import { computed, ref, useTemplateRef } from 'vue'

import { Button } from '#ui/components/base/buttons'
import { injectModrinthClient } from '#ui/providers/api-client'
import { injectNotificationManager } from '#ui/providers/web-notifications.ts'

import { useFormatDateTime, useFormatPrice } from '../../composables'
import { defineMessages, useVIntl } from '../../composables/i18n'
import { regionOverrides } from '../../utils/regions'
import IntlFormatted from '../base/IntlFormatted.vue'
import TagItem from '../base/TagItem.vue'
import { NewModal } from '../index'
import ModrinthServersIcon from '../servers/ModrinthServersIcon.vue'
import ExpandableInvoiceTotal from './ExpandableInvoiceTotal.vue'
import FormattedPaymentMethod from './FormattedPaymentMethod.vue'
import PaymentMethodOption from './PaymentMethodOption.vue'
import ServersSpecs from './ServersSpecs.vue'

const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const { labrinth } = injectModrinthClient()
const queryClient = useQueryClient()

const props = defineProps<{
	paymentMethods?: Stripe.PaymentMethod[]
	regions?: Archon.Servers.v1.Region[]
	onAddPaymentMethod?: () => void
}>()

export type ResubscribeModalPayload = {
	subscriptionId: string
	wasSuspended: boolean
	serverName: string
	plan?: Labrinth.Billing.Internal.Product | null
	regionShortcode?: string
}

const emit = defineEmits<{
	(e: 'cancel'): void
	(e: 'resubscribe', payload: { subscriptionId: string; wasSuspended: boolean }): void
}>()

const formatDate = useFormatDateTime({ dateStyle: 'long' })
const formatPrice = useFormatPrice()

const modal = useTemplateRef<InstanceType<typeof NewModal>>('modal')

const modalData = ref<ResubscribeModalPayload | null>(null)
const preview = ref<Labrinth.Billing.Internal.ResubscribePreview | null>(null)
const loadingPreview = ref(false)
const previewError = ref(false)
const choosingPaymentMethod = ref(false)
const settingPaymentMethodId = ref<string | null>(null)

const messages = defineMessages({
	title: {
		id: 'billing.resubscribe-modal.title',
		defaultMessage: 'Resubscribe to Modrinth Hosting',
	},
	description: {
		id: 'billing.resubscribe-modal.description',
		defaultMessage:
			'You are about to resubscribe to <server-name>{serverName}</server-name>. Your subscription will be reactivated and your server will continue running without interruption.',
	},
	planItem: {
		id: 'billing.resubscribe-modal.plan-item',
		defaultMessage: 'Modrinth Hosting ({plan})',
	},
	taxItem: { id: 'billing.resubscribe-modal.tax-item', defaultMessage: 'Tax' },
	planSmall: { id: 'billing.resubscribe-modal.plan.small', defaultMessage: 'Small' },
	planMedium: { id: 'billing.resubscribe-modal.plan.medium', defaultMessage: 'Medium' },
	planLarge: { id: 'billing.resubscribe-modal.plan.large', defaultMessage: 'Large' },
	planCustom: { id: 'billing.resubscribe-modal.plan.custom', defaultMessage: 'Custom' },
	period: {
		id: 'billing.resubscribe-modal.period',
		defaultMessage:
			'{interval, select, fiveDays {5 days} monthly {month} quarterly {3 months} yearly {year} other {{interval}}}',
	},
	previewError: {
		id: 'billing.resubscribe-modal.preview-error',
		defaultMessage:
			"We couldn't calculate the cost of resubscribing. Make sure your payment method has a billing address, then try again.",
	},
	noPaymentMethod: {
		id: 'billing.resubscribe-modal.no-payment-method',
		defaultMessage: 'No payment method selected',
	},
	changePaymentMethod: {
		id: 'billing.resubscribe-modal.change-payment-method',
		defaultMessage: 'Change',
	},
	addPaymentMethod: {
		id: 'billing.resubscribe-modal.add-payment-method',
		defaultMessage: 'Add payment method',
	},
	chargedNow: {
		id: 'billing.resubscribe-modal.charged-now',
		defaultMessage:
			"You'll be charged {total} today, and then every {period} until you cancel. You can cancel anytime from your settings page.",
	},
	chargedOnDate: {
		id: 'billing.resubscribe-modal.charged-on-date',
		defaultMessage:
			"You'll be charged {total} on <charge-date>{date}</charge-date>, and then every {period} until you cancel. You can cancel anytime from your settings page.",
	},
	recurringNotice: {
		id: 'billing.resubscribe-modal.recurring-notice',
		defaultMessage: 'By clicking "Resubscribe", you are resuming a recurring subscription.',
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
	errorTitle: { id: 'billing.resubscribe-modal.error.title', defaultMessage: 'Error' },
	errorText: {
		id: 'billing.resubscribe-modal.error.text',
		defaultMessage: 'Cannot resubscribe, failed to load subscription details.',
	},
	paymentMethodErrorText: {
		id: 'billing.resubscribe-modal.error.payment-method',
		defaultMessage: 'Failed to change your payment method.',
	},
})

const savedPaymentMethods = computed(() => props.paymentMethods ?? [])

const paymentMethod = computed(
	() => (preview.value?.payment_method as Stripe.PaymentMethod | undefined) ?? undefined,
)

const currency = computed(() => preview.value?.currency_code ?? 'USD')

const period = computed(() => {
	const interval = preview.value?.interval
	if (!interval) return undefined
	return formatMessage(messages.period, {
		interval: interval === 'five-days' ? 'fiveDays' : interval,
	})
})

const dueDate = computed(() => {
	if (!preview.value) return null
	const date = new Date(preview.value.due)
	return Number.isNaN(date.getTime()) ? null : date
})

const chargedNow = computed(() => !dueDate.value || dueDate.value.getTime() <= Date.now())

const formattedDueDate = computed(() => (dueDate.value ? formatDate(dueDate.value) : ''))

const canResubscribe = computed(
	() =>
		!!modalData.value?.subscriptionId &&
		!!preview.value &&
		!!paymentMethod.value &&
		!loadingPreview.value &&
		!settingPaymentMethodId.value,
)

const planName = computed(() => {
	const metadata = modalData.value?.plan?.metadata
	if (!metadata || (metadata.type !== 'pyro' && metadata.type !== 'medal')) {
		return formatMessage(messages.planCustom)
	}
	switch (metadata.ram) {
		case 4096:
			return formatMessage(messages.planSmall)
		case 6144:
			return formatMessage(messages.planMedium)
		case 8192:
			return formatMessage(messages.planLarge)
		default:
			return formatMessage(messages.planCustom)
	}
})

const planSpecs = computed(() => {
	const metadata = modalData.value?.plan?.metadata
	if (metadata?.type === 'pyro' || metadata?.type === 'medal') {
		return { ram: metadata.ram, storage: metadata.storage, cpu: metadata.cpu }
	}
	return null
})

const region = computed(() =>
	modalData.value?.regionShortcode
		? props.regions?.find((r) => r.shortcode === modalData.value?.regionShortcode)
		: undefined,
)
const regionFlag = computed(() =>
	region.value
		? (regionOverrides[region.value.shortcode]?.flag ??
			`https://flagcdn.com/${region.value.country_code}.svg`)
		: undefined,
)
const regionOverrideTitle = computed(() =>
	region.value ? regionOverrides[region.value.shortcode]?.name : undefined,
)
const regionTitle = computed(() =>
	regionOverrideTitle.value ? formatMessage(regionOverrideTitle.value) : region.value?.display_name,
)
const regionSubtitle = computed(() =>
	regionOverrideTitle.value ? region.value?.display_name : undefined,
)

async function loadPreview() {
	if (!modalData.value) return
	const subscriptionId = modalData.value.subscriptionId

	loadingPreview.value = true
	previewError.value = false
	try {
		const result = await labrinth.billing_internal.previewResubscribe(subscriptionId)
		if (modalData.value?.subscriptionId === subscriptionId) {
			preview.value = result
		}
	} catch {
		if (modalData.value?.subscriptionId === subscriptionId) {
			preview.value = null
			previewError.value = true
		}
	} finally {
		loadingPreview.value = false
	}
}

async function selectPaymentMethod(method: Stripe.PaymentMethod) {
	if (method.id === paymentMethod.value?.id || settingPaymentMethodId.value) return

	settingPaymentMethodId.value = method.id
	try {
		await labrinth.billing_internal.editPaymentMethod(method.id, { primary: true })
		await queryClient.invalidateQueries({ queryKey: ['billing', 'customer'] })
		await loadPreview()
		choosingPaymentMethod.value = false
	} catch {
		addNotification({
			type: 'error',
			title: formatMessage(messages.errorTitle),
			text: formatMessage(messages.paymentMethodErrorText),
		})
	} finally {
		settingPaymentMethodId.value = null
	}
}

function addPaymentMethod() {
	hide()
	props.onAddPaymentMethod?.()
}

function show(payload: ResubscribeModalPayload) {
	if (!payload) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.errorTitle),
			text: formatMessage(messages.errorText),
		})
		return
	}

	modalData.value = { ...payload, serverName: payload.serverName.trim() }
	preview.value = null
	choosingPaymentMethod.value = false
	modal.value?.show()
	void loadPreview()
}

function hide() {
	modal.value?.hide()
}

function handleHide() {
	choosingPaymentMethod.value = false
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
