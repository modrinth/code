<template>
	<ServersUpgradeModalWrapper ref="upgradeModal" />
	<section class="universal-card experimental-styles-within">
		<h2>{{ formatMessage(messages.subscriptionTitle) }}</h2>
		<p>{{ formatMessage(messages.subscriptionDescription) }}</p>
		<div class="universal-card recessed">
			<ConfirmModal
				ref="modalCancel"
				:title="formatMessage(cancelModalMessages.title)"
				:description="formatMessage(cancelModalMessages.description)"
				:proceed-label="formatMessage(cancelModalMessages.action)"
				@proceed="cancelSubscription(cancelSubscriptionId, true)"
			/>
			<div class="flex flex-wrap justify-between gap-4">
				<div class="flex flex-col gap-4">
					<template v-if="midasCharge">
						<span v-if="midasCharge.status === 'open'">
							{{ formatMessage(messages.midasStatusOpen) }}
						</span>
						<span v-else-if="midasCharge.status === 'processing'" class="text-orange">
							{{ formatMessage(messages.midasStatusProcessing) }}
						</span>
						<span v-else-if="midasCharge.status === 'cancelled'">
							{{ formatMessage(messages.midasStatusCancelledLine1) }} <br />
							{{ formatMessage(messages.midasStatusCancelledLine2) }}
						</span>
						<span v-else-if="midasCharge.status === 'failed'" class="text-red">
							{{ formatMessage(messages.midasStatusFailed) }}
						</span>
					</template>

					<span v-else>{{ formatMessage(messages.midasUpsell) }}</span>
					<ModrinthPlusIcon class="h-8 w-min" />
					<div class="flex flex-col gap-2">
						<span class="font-bold">{{ formatMessage(messages.midasBenefitsTitle) }}</span>
						<div class="flex items-center gap-2">
							<CheckCircleIcon class="h-5 w-5 shrink-0 text-brand" />
							<span>{{ formatMessage(messages.midasBenefitAdFree) }}</span>
						</div>
						<div class="flex items-center gap-2">
							<CheckCircleIcon class="h-5 w-5 shrink-0 text-brand" />
							<span>{{ formatMessage(messages.midasBenefitBadge) }}</span>
						</div>
						<div class="flex items-center gap-2">
							<CheckCircleIcon class="h-5 w-5 shrink-0 text-brand" />
							<span>{{ formatMessage(messages.midasBenefitSupport) }}</span>
						</div>
					</div>
				</div>
				<div class="flex w-full flex-wrap justify-between gap-4 xl:w-auto xl:flex-col">
					<div class="flex flex-col gap-1 xl:ml-auto xl:text-right">
						<span class="text-2xl font-bold text-dark">
							<template v-if="midasCharge">
								{{
									formatMessage(messages.pricePerInterval, {
										price: formatPrice(
											vintl.locale,
											midasSubscriptionPrice.prices.intervals[midasSubscription.interval],
											midasSubscriptionPrice.currency_code,
										),
										interval: getIntervalNounLabel(midasSubscription.interval),
									})
								}}
							</template>
							<template v-else>
								{{
									formatMessage(messages.pricePerInterval, {
										price: formatPrice(
											vintl.locale,
											price.prices.intervals.monthly,
											price.currency_code,
										),
										interval: formatMessage(messages.intervalMonth),
									})
								}}
							</template>
						</span>
						<!-- Next charge preview for Midas when interval is changing -->
						<div
							v-if="
								midasCharge &&
								midasCharge.status === 'open' &&
								midasSubscription &&
								midasSubscription.interval &&
								midasCharge.subscription_interval !== midasSubscription.interval
							"
							class="-mt-1 flex items-baseline gap-2 text-sm text-secondary"
						>
							<span class="opacity-70">{{ formatMessage(messages.nextLabel) }}</span>
							<span class="font-semibold text-contrast">
								{{ formatPrice(vintl.locale, midasCharge.amount, midasCharge.currency_code) }}
							</span>
							<span>
								{{
									formatMessage(messages.slashInterval, {
										interval: getIntervalNounLabel(midasCharge.subscription_interval),
									})
								}}
							</span>
						</div>
						<template v-if="midasCharge">
							<span
								v-if="
									midasCharge.status === 'open' &&
									midasCharge.subscription_interval === 'monthly' &&
									oppositePrice != null
								"
								class="text-sm text-purple"
							>
								{{
									formatMessage(messages.savePerYearBySwitchingToYearly, {
										amount: formatPrice(
											vintl.locale,
											midasCharge.amount * 12 - oppositePrice,
											midasCharge.currency_code,
										),
									})
								}}
							</span>
							<span class="text-sm text-secondary">
								{{
									formatMessage(messages.sinceDate, {
										date: $dayjs(midasSubscription.created).format('MMMM D, YYYY'),
									})
								}}
							</span>
							<span v-if="midasCharge.status === 'open'" class="text-sm text-secondary">
								{{
									formatMessage(messages.renewsDate, {
										date: $dayjs(midasCharge.due).format('MMMM D, YYYY'),
									})
								}}
							</span>
							<span v-else-if="midasCharge.status === 'cancelled'" class="text-sm text-secondary">
								{{
									formatMessage(messages.expiresDate, {
										date: $dayjs(midasCharge.due).format('MMMM D, YYYY'),
									})
								}}
							</span>
							<span
								v-if="
									midasCharge.status === 'open' &&
									midasSubscription &&
									midasSubscription.interval &&
									midasCharge.subscription_interval !== midasSubscription.interval
								"
								class="text-sm text-secondary"
							>
								{{
									formatMessage(messages.switchesToBillingOn, {
										interval: getIntervalAdjectiveLabel(midasCharge.subscription_interval),
										date: $dayjs(midasCharge.due).format('MMMM D, YYYY'),
									})
								}}
							</span>
						</template>

						<span v-else class="text-sm text-secondary">
							{{
								formatMessage(messages.orYearlySave, {
									price: formatPrice(
										vintl.locale,
										price.prices.intervals.yearly,
										price.currency_code,
									),
									percent: calculateSavings(
										price.prices.intervals.monthly,
										price.prices.intervals.yearly,
									),
								})
							}}
						</span>
					</div>
					<div
						v-if="midasCharge && midasCharge.status === 'failed'"
						class="ml-auto flex flex-row-reverse items-center gap-2"
					>
						<ButtonStyled v-if="midasCharge && midasCharge.status === 'failed'">
							<button
								@click="
									() => {
										$refs.midasPurchaseModal.show()
									}
								"
							>
								<UpdatedIcon />
								{{ formatMessage(messages.updateMethod) }}
							</button>
						</ButtonStyled>
						<ButtonStyled type="transparent" circular>
							<OverflowMenu
								:dropdown-id="`${baseId}-cancel-midas`"
								:options="[
									{
										id: 'cancel',
										action: () => {
											cancelSubscriptionId = midasSubscription.id
											$refs.modalCancel.show()
										},
									},
								]"
							>
								<MoreVerticalIcon />
								<template #cancel
									><XIcon /> {{ formatMessage(commonMessages.cancelButton) }}</template
								>
							</OverflowMenu>
						</ButtonStyled>
					</div>
					<div
						v-else-if="midasCharge && midasCharge.status !== 'cancelled'"
						class="ml-auto flex gap-2"
					>
						<ButtonStyled>
							<button
								:disabled="changingInterval"
								@click="
									() => {
										cancelSubscriptionId = midasSubscription.id
										$refs.modalCancel.show()
									}
								"
							>
								<XIcon /> {{ formatMessage(commonMessages.cancelButton) }}
							</button>
						</ButtonStyled>
						<ButtonStyled
							:color="midasCharge.subscription_interval === 'yearly' ? 'standard' : 'purple'"
							color-fill="text"
						>
							<button
								v-tooltip="
									midasCharge.subscription_interval === 'yearly'
										? formatMessage(messages.monthlyBillingAdditionalPerYearTooltip, {
												amount: formatPrice(
													vintl.locale,
													oppositePrice * 12 - midasCharge.amount,
													midasCharge.currency_code,
												),
											})
										: undefined
								"
								:disabled="changingInterval"
								@click="switchMidasInterval(oppositeInterval)"
							>
								<SpinnerIcon v-if="changingInterval" class="animate-spin" />
								<TransferIcon v-else />
								{{
									changingInterval
										? formatMessage(messages.switchingToInterval, {
												interval: getIntervalAdjectiveLabel(oppositeInterval),
											})
										: formatMessage(messages.switchToInterval, {
												interval: getIntervalAdjectiveLabel(oppositeInterval),
											})
								}}
							</button>
						</ButtonStyled>
					</div>
					<ButtonStyled
						v-else-if="midasCharge && midasCharge.status === 'cancelled'"
						color="purple"
					>
						<button class="ml-auto" @click="cancelSubscription(midasSubscription.id, false)">
							{{ formatMessage(messages.resubscribe) }} <RightArrowIcon />
						</button>
					</ButtonStyled>
					<ButtonStyled v-else color="purple" size="large">
						<button
							class="ml-auto"
							@click="
								() => {
									$refs.midasPurchaseModal.show()
								}
							"
						>
							{{ formatMessage(messages.subscribe) }} <RightArrowIcon />
						</button>
					</ButtonStyled>
				</div>
			</div>
		</div>
		<div v-if="pyroSubscriptions.length > 0">
			<div
				v-for="(subscription, index) in pyroSubscriptions"
				:key="index"
				class="universal-card recessed mt-4"
			>
				<div class="flex flex-col justify-between gap-4">
					<div class="flex flex-col gap-4">
						<ModrinthServersIcon class="flex h-8 w-fit" />
						<div class="flex flex-col gap-2">
							<ServerListing
								v-if="subscription.serverInfo"
								v-bind="subscription.serverInfo"
								:pending-change="getPendingChange(subscription)"
							/>
							<div v-else class="w-fit">
								<p>
									{{ formatMessage(messages.pyroLinkedServerNotFound) }}
								</p>
								<div class="flex w-full flex-col gap-2">
									<CopyCode
										class="whitespace-nowrap"
										:text="
											formatMessage(messages.pyroServerIdLabel, {
												id: subscription.metadata.id,
											})
										"
									/>
									<CopyCode
										class="whitespace-nowrap"
										:text="
											formatMessage(messages.pyroStripeIdLabel, {
												id: subscription.id,
											})
										"
									/>
								</div>
							</div>
							<h3 class="m-0 mt-4 text-xl font-semibold leading-none text-contrast">
								{{
									formatMessage(messages.planTitle, {
										size: getProductSize(getPyroProduct(subscription)),
									})
								}}
							</h3>
							<div class="flex flex-row justify-between">
								<div class="mt-2 flex flex-col gap-2">
									<div class="flex items-center gap-2">
										<CheckCircleIcon class="h-5 w-5 text-brand" />
										<span>
											{{
												formatMessage(messages.pyroCpuLine, {
													shared: getPyroProduct(subscription)?.metadata?.cpu / 2,
													bursts: getPyroProduct(subscription)?.metadata?.cpu,
												})
											}}
										</span>
									</div>
									<div class="flex items-center gap-2">
										<CheckCircleIcon class="h-5 w-5 text-brand" />
										<span>
											{{
												getPyroProduct(subscription)?.metadata?.ram
													? formatMessage(messages.pyroRamLine, {
															gb: getPyroProduct(subscription).metadata.ram / 1024,
														})
													: ''
											}}
										</span>
									</div>
									<div class="flex items-center gap-2">
										<CheckCircleIcon class="h-5 w-5 text-brand" />
										<span>
											{{
												getPyroProduct(subscription)?.metadata?.swap
													? formatMessage(messages.pyroSwapLine, {
															gb: getPyroProduct(subscription).metadata.swap / 1024,
														})
													: ''
											}}
										</span>
									</div>
									<div class="flex items-center gap-2">
										<CheckCircleIcon class="h-5 w-5 text-brand" />
										<span>
											{{
												getPyroProduct(subscription)?.metadata?.storage
													? formatMessage(messages.pyroStorageLine, {
															gb: getPyroProduct(subscription).metadata.storage / 1024,
														})
													: ''
											}}
										</span>
									</div>
								</div>
								<div class="flex flex-col items-end justify-between">
									<div class="flex flex-col items-end gap-2">
										<div class="flex text-2xl font-bold text-contrast">
											<span class="text-contrast">
												{{
													formatPrice(
														vintl.locale,
														getProductPrice(getPyroProduct(subscription), subscription.interval)
															.prices.intervals[subscription.interval],
														getProductPrice(getPyroProduct(subscription), subscription.interval)
															.currency_code,
													)
												}}
											</span>
											<span>
												{{
													formatMessage(messages.slashInterval, {
														interval: getIntervalNounLabel(subscription.interval),
													})
												}}
											</span>
										</div>
										<div
											v-if="
												getPyroCharge(subscription) &&
												getPyroCharge(subscription).status === 'open' &&
												((getPyroCharge(subscription).price_id &&
													getPyroCharge(subscription).price_id !== subscription.price_id) ||
													(getPyroCharge(subscription).subscription_interval &&
														getPyroCharge(subscription).subscription_interval !==
															subscription.interval))
											"
											class="-mt-1 flex items-baseline gap-2 text-sm text-secondary"
										>
											<span class="opacity-70">{{ formatMessage(messages.nextLabel) }}</span>
											<span class="font-semibold text-contrast">
												{{
													formatPrice(
														vintl.locale,
														getPyroCharge(subscription).amount,
														getPyroCharge(subscription).currency_code,
													)
												}}
											</span>
											<span>
												{{
													formatMessage(messages.slashInterval, {
														interval: getIntervalNounLabel(
															getPyroCharge(subscription).subscription_interval ||
																subscription.interval,
														),
													})
												}}
											</span>
										</div>
										<div v-if="getPyroCharge(subscription)" class="mb-4 flex flex-col items-end">
											<span class="text-sm text-secondary">
												{{
													formatMessage(messages.sinceDate, {
														date: $dayjs(subscription.created).format('MMMM D, YYYY'),
													})
												}}
											</span>
											<span
												v-if="getPyroCharge(subscription).status === 'open'"
												class="text-sm text-secondary"
											>
												{{
													formatMessage(messages.renewsDate, {
														date: $dayjs(getPyroCharge(subscription).due).format('MMMM D, YYYY'),
													})
												}}
											</span>
											<span
												v-if="
													getPyroCharge(subscription).status === 'open' &&
													getPyroCharge(subscription).subscription_interval &&
													getPyroCharge(subscription).subscription_interval !==
														subscription.interval
												"
												class="text-sm text-secondary"
											>
												{{
													formatMessage(messages.switchesToBillingOn, {
														interval: getIntervalAdjectiveLabel(
															getPyroCharge(subscription).subscription_interval,
														),
														date: $dayjs(getPyroCharge(subscription).due).format('MMMM D, YYYY'),
													})
												}}
											</span>
											<span
												v-else-if="getPyroCharge(subscription).status === 'processing'"
												class="text-sm text-orange"
											>
												{{ formatMessage(messages.pyroStatusProcessing) }}
											</span>
											<span
												v-else-if="getPyroCharge(subscription).status === 'cancelled'"
												class="text-sm text-secondary"
											>
												{{
													formatMessage(messages.expiresDate, {
														date: $dayjs(getPyroCharge(subscription).due).format('MMMM D, YYYY'),
													})
												}}
											</span>
											<span
												v-else-if="getPyroCharge(subscription).status === 'failed'"
												class="text-sm text-red"
											>
												{{ formatMessage(messages.pyroStatusFailed) }}
											</span>
										</div>
									</div>
									<div class="flex gap-2">
										<ButtonStyled
											v-if="
												getPyroCharge(subscription) &&
												getPyroCharge(subscription).status !== 'cancelled'
											"
										>
											<button @click="showCancellationSurvey(subscription)">
												<XIcon />
												{{ formatMessage(commonMessages.cancelButton) }}
											</button>
										</ButtonStyled>
										<ButtonStyled
											v-if="
												getPyroCharge(subscription) &&
												getPyroCharge(subscription).status !== 'cancelled' &&
												getPyroCharge(subscription).status !== 'failed'
											"
											color="green"
											color-fill="text"
										>
											<button @click="showPyroUpgradeModal(subscription)">
												<ArrowBigUpDashIcon />
												{{ formatMessage(messages.upgrade) }}
											</button>
										</ButtonStyled>
										<ButtonStyled
											v-else-if="
												getPyroCharge(subscription) &&
												(getPyroCharge(subscription).status === 'cancelled' ||
													getPyroCharge(subscription).status === 'failed')
											"
											color="green"
										>
											<button
												@click="
													resubscribePyro(
														subscription.id,
														$dayjs(getPyroCharge(subscription).due).isBefore($dayjs()),
													)
												"
											>
												{{ formatMessage(messages.resubscribe) }} <RightArrowIcon />
											</button>
										</ButtonStyled>
									</div>
								</div>
							</div>
						</div>
					</div>
				</div>
			</div>
		</div>
	</section>

	<section class="universal-card experimental-styles-within">
		<ConfirmModal
			ref="modal_confirm"
			:title="formatMessage(deleteModalMessages.title)"
			:description="formatMessage(deleteModalMessages.description)"
			:proceed-label="formatMessage(deleteModalMessages.action)"
			@proceed="removePaymentMethod(removePaymentMethodIndex)"
		/>
		<PurchaseModal
			ref="midasPurchaseModal"
			:product="midasProduct"
			:country="country"
			:publishable-key="config.public.stripePublishableKey"
			:send-billing-request="
				async (body) =>
					await useBaseFetch('billing/payment', { internal: true, method: 'POST', body })
			"
			:on-error="
				(err) =>
					addNotification({
						title: formatMessage(commonMessages.errorNotificationTitle),
						type: 'error',
						text: err.message ?? (err.data ? err.data.description : err),
					})
			"
			:customer="customer"
			:payment-methods="paymentMethods"
			:return-url="`${config.public.siteUrl}/settings/billing`"
		/>
		<AddPaymentMethodModal
			ref="addPaymentMethodModal"
			:publishable-key="config.public.stripePublishableKey"
			:return-url="`${config.public.siteUrl}/settings/billing`"
			:create-setup-intent="createSetupIntent"
			:on-error="handleError"
		/>
		<div class="header__row">
			<div class="header__title">
				<h2 class="text-2xl">{{ formatMessage(messages.paymentMethodTitle) }}</h2>
			</div>
			<nuxt-link class="btn" to="/settings/billing/charges">
				<HistoryIcon /> {{ formatMessage(messages.paymentMethodHistory) }}
			</nuxt-link>
			<button class="btn" @click="addPaymentMethod">
				<PlusIcon /> {{ formatMessage(messages.paymentMethodAdd) }}
			</button>
		</div>
		<div
			v-if="!paymentMethods || paymentMethods.length === 0"
			class="universal-card recessed !mb-0"
		>
			{{ formatMessage(messages.paymentMethodNone) }}
		</div>
		<div v-else class="flex flex-col gap-4">
			<div
				v-for="(method, index) in paymentMethods"
				:key="index"
				class="universal-card recessed !mb-0 flex items-center justify-between"
			>
				<div class="flex gap-2">
					<component :is="getPaymentMethodIcon(method.type)" class="h-8 w-8" />
					<div class="flex flex-col">
						<div class="flex items-center gap-2">
							<div class="font-bold text-contrast">
								<template v-if="method.type === 'card'">
									{{
										formatMessage(paymentMethodMessages.paymentMethodCardDisplay, {
											card_brand:
												formatMessage(paymentMethodMessages[method.card.brand]) ??
												formatMessage(paymentMethodMessages.unknown),
											last_four: method.card.last4,
										})
									}}
								</template>
								<template v-else>
									{{
										formatMessage(paymentMethodMessages[method.type]) ??
										formatMessage(paymentMethodMessages.unknown)
									}}
								</template>
							</div>
							<div
								v-if="primaryPaymentMethodId === method.id"
								class="border-r-ma rounded-full bg-button-bg px-2 py-0.5 text-sm font-bold text-secondary"
							>
								{{ formatMessage(messages.paymentMethodPrimary) }}
							</div>
						</div>
						<div v-if="method.type === 'card'" class="text-secondary">
							{{
								formatMessage(messages.paymentMethodCardExpiry, {
									month: method.card.exp_month,
									year: method.card.exp_year,
								})
							}}
						</div>
						<div v-else-if="method.type === 'cashapp'" class="text-secondary">
							{{ method.cashapp.cashtag }}
						</div>
						<div v-else-if="method.type === 'paypal'" class="text-secondary">
							{{ method.paypal.payer_email }}
						</div>
					</div>
				</div>
				<OverflowMenu
					:dropdown-id="`${baseId}-payment-method-overflow-${index}`"
					class="btn icon-only transparent"
					:options="
						[
							{
								id: 'primary',
								action: () => editPaymentMethod(index, true),
							},
							{
								id: 'remove',
								action: () => {
									removePaymentMethodIndex = index
									$refs.modal_confirm.show()
								},
								color: 'red',
								hoverOnly: true,
							},
						].slice(primaryPaymentMethodId === method.id ? 1 : 0, 2)
					"
				>
					<MoreVerticalIcon />
					<template #primary>
						<StarIcon />
						{{ formatMessage(messages.paymentMethodMakePrimary) }}
					</template>
					<template #edit>
						<EditIcon />
						{{ formatMessage(commonMessages.editButton) }}
					</template>
					<template #remove>
						<TrashIcon />
						{{ formatMessage(commonMessages.deleteLabel) }}
					</template>
				</OverflowMenu>
			</div>
		</div>
	</section>
</template>

<script setup>
import {
	ArrowBigUpDashIcon,
	CheckCircleIcon,
	EditIcon,
	HistoryIcon,
	ModrinthPlusIcon,
	MoreVerticalIcon,
	PlusIcon,
	RightArrowIcon,
	SpinnerIcon,
	StarIcon,
	TransferIcon,
	TrashIcon,
	UpdatedIcon,
	XIcon,
} from '@modrinth/assets'
import {
	AddPaymentMethodModal,
	ButtonStyled,
	commonMessages,
	ConfirmModal,
	CopyCode,
	defineMessages,
	getPaymentMethodIcon,
	injectNotificationManager,
	OverflowMenu,
	paymentMethodMessages,
	PurchaseModal,
	ServerListing,
	useVIntl,
} from '@modrinth/ui'
import { calculateSavings, formatPrice, getCurrency } from '@modrinth/utils'
import { computed, ref } from 'vue'

import { useBaseFetch } from '@/composables/fetch.js'
import ModrinthServersIcon from '~/components/ui/servers/ModrinthServersIcon.vue'
import ServersUpgradeModalWrapper from '~/components/ui/servers/ServersUpgradeModalWrapper.vue'
import { useServersFetch } from '~/composables/servers/servers-fetch.ts'
import { products } from '~/generated/state.json'

const { addNotification, handleError } = injectNotificationManager()
definePageMeta({
	middleware: 'auth',
})

const auth = await useAuth()
const baseId = useId()

useHead({
	script: [
		{
			src: 'https://js.stripe.com/v3/',
			defer: true,
			async: true,
		},
	],
})

const config = useRuntimeConfig()

const vintl = useVIntl()
const { formatMessage } = vintl

const deleteModalMessages = defineMessages({
	title: {
		id: 'settings.billing.modal.delete.title',
		defaultMessage: 'Are you sure you want to remove this payment method?',
	},
	description: {
		id: 'settings.billing.modal.delete.description',
		defaultMessage: 'This will remove this payment method forever (like really forever).',
	},
	action: {
		id: 'settings.billing.modal.delete.action',
		defaultMessage: 'Remove this payment method',
	},
})

const cancelModalMessages = defineMessages({
	title: {
		id: 'settings.billing.modal.cancel.title',
		defaultMessage: 'Are you sure you want to cancel your subscription?',
	},
	description: {
		id: 'settings.billing.modal.cancel.description',
		defaultMessage:
			'This will cancel your subscription. You will retain your perks until the end of the current billing cycle.',
	},
	action: {
		id: 'settings.billing.modal.cancel.action',
		defaultMessage: 'Cancel subscription',
	},
})

const messages = defineMessages({
	subscriptionTitle: {
		id: 'settings.billing.subscription.title',
		defaultMessage: 'Subscriptions',
	},
	subscriptionDescription: {
		id: 'settings.billing.subscription.description',
		defaultMessage: 'Manage your Modrinth subscriptions.',
	},
	paymentMethodTitle: {
		id: 'settings.billing.payment_method.title',
		defaultMessage: 'Payment methods',
	},
	paymentMethodNone: {
		id: 'settings.billing.payment_method.none',
		defaultMessage: 'You have not added any payment methods.',
	},
	paymentMethodHistory: {
		id: 'settings.billing.payment_method.action.history',
		defaultMessage: 'View past charges',
	},
	paymentMethodAdd: {
		id: 'settings.billing.payment_method.action.add',
		defaultMessage: 'Add payment method',
	},
	paymentMethodPrimary: {
		id: 'settings.billing.payment_method.primary',
		defaultMessage: 'Primary',
	},
	paymentMethodMakePrimary: {
		id: 'settings.billing.payment_method.action.primary',
		defaultMessage: 'Make primary',
	},
	paymentMethodCardExpiry: {
		id: 'settings.billing.payment_method.card_expiry',
		defaultMessage: 'Expires {month}/{year}',
	},
	pyroSubscriptionTitle: {
		id: 'settings.billing.pyro_subscription.title',
		defaultMessage: 'Modrinth Server Subscriptions',
	},
	pyroSubscriptionDescription: {
		id: 'settings.billing.pyro_subscription.description',
		defaultMessage: 'Manage your Modrinth Server subscriptions.',
	},
	intervalMonth: {
		id: 'settings.billing.interval.month',
		defaultMessage: 'month',
	},
	intervalYear: {
		id: 'settings.billing.interval.year',
		defaultMessage: 'year',
	},
	intervalMonthly: {
		id: 'settings.billing.interval.monthly',
		defaultMessage: 'monthly',
	},
	intervalYearly: {
		id: 'settings.billing.interval.yearly',
		defaultMessage: 'yearly',
	},
	pricePerInterval: {
		id: 'settings.billing.price.per-interval',
		defaultMessage: '{price} / {interval}',
	},
	slashInterval: {
		id: 'settings.billing.price.slash-interval',
		defaultMessage: '/{interval}',
	},
	nextLabel: {
		id: 'settings.billing.next',
		defaultMessage: 'Next:',
	},
	midasStatusOpen: {
		id: 'settings.billing.midas.status.open',
		defaultMessage: "You're currently subscribed to:",
	},
	midasStatusProcessing: {
		id: 'settings.billing.midas.status.processing',
		defaultMessage:
			'Your payment is being processed. Perks will activate once payment is complete.',
	},
	midasStatusCancelledLine1: {
		id: 'settings.billing.midas.status.cancelled.line1',
		defaultMessage: "You've cancelled your subscription.",
	},
	midasStatusCancelledLine2: {
		id: 'settings.billing.midas.status.cancelled.line2',
		defaultMessage: 'You will retain your perks until the end of the current billing cycle.',
	},
	midasStatusFailed: {
		id: 'settings.billing.midas.status.failed',
		defaultMessage: 'Your subscription payment failed. Please update your payment method.',
	},
	midasUpsell: {
		id: 'settings.billing.midas.upsell',
		defaultMessage: 'Become a subscriber to Modrinth Plus!',
	},
	midasBenefitsTitle: {
		id: 'settings.billing.midas.benefits.title',
		defaultMessage: 'Benefits',
	},
	midasBenefitAdFree: {
		id: 'settings.billing.midas.benefits.ad-free',
		defaultMessage: 'Ad-free browsing on modrinth.com and Modrinth App',
	},
	midasBenefitBadge: {
		id: 'settings.billing.midas.benefits.badge',
		defaultMessage: 'Modrinth+ badge on your profile',
	},
	midasBenefitSupport: {
		id: 'settings.billing.midas.benefits.support',
		defaultMessage: 'Support Modrinth and creators directly',
	},
	savePerYearBySwitchingToYearly: {
		id: 'settings.billing.midas.save-per-year',
		defaultMessage: 'Save {amount}/year by switching to yearly billing!',
	},
	sinceDate: {
		id: 'settings.billing.since',
		defaultMessage: 'Since {date}',
	},
	renewsDate: {
		id: 'settings.billing.renews',
		defaultMessage: 'Renews {date}',
	},
	expiresDate: {
		id: 'settings.billing.expires',
		defaultMessage: 'Expires {date}',
	},
	switchesToBillingOn: {
		id: 'settings.billing.switches-to-billing-on',
		defaultMessage: 'Switches to {interval} billing on {date}',
	},
	orYearlySave: {
		id: 'settings.billing.or-yearly-save',
		defaultMessage: 'Or {price} / year (save {percent}%)!',
	},
	updateMethod: {
		id: 'settings.billing.update-method',
		defaultMessage: 'Update method',
	},
	switchToInterval: {
		id: 'settings.billing.switch.to-interval',
		defaultMessage: 'Switch to {interval}',
	},
	switchingToInterval: {
		id: 'settings.billing.switch.switching-to-interval',
		defaultMessage: 'Switching to {interval}',
	},
	monthlyBillingAdditionalPerYearTooltip: {
		id: 'settings.billing.switch.tooltip.monthly-additional-per-year',
		defaultMessage: 'Monthly billing will cost you an additional {amount} per year',
	},
	resubscribe: {
		id: 'settings.billing.resubscribe',
		defaultMessage: 'Resubscribe',
	},
	subscribe: {
		id: 'settings.billing.subscribe',
		defaultMessage: 'Subscribe',
	},
	upgrade: {
		id: 'settings.billing.upgrade',
		defaultMessage: 'Upgrade',
	},
	pyroLinkedServerNotFound: {
		id: 'settings.billing.pyro.linked-server.not-found',
		defaultMessage:
			"A linked server couldn't be found for this subscription. There are a few possible explanations for this. If you just purchased your server, this is normal. It could take up to an hour for your server to be provisioned. Otherwise, if you purchased this server a while ago, it has likely since been suspended. If this is not what you were expecting, please contact Modrinth Support with the following information:",
	},
	pyroServerIdLabel: {
		id: 'settings.billing.pyro.linked-server.server-id',
		defaultMessage: 'Server ID: {id}',
	},
	pyroStripeIdLabel: {
		id: 'settings.billing.pyro.linked-server.stripe-id',
		defaultMessage: 'Stripe ID: {id}',
	},
	planTitle: {
		id: 'settings.billing.plan.title',
		defaultMessage: '{size} Plan',
	},
	productSizeUnknown: {
		id: 'settings.billing.plan.size.unknown',
		defaultMessage: 'Unknown',
	},
	productSizeSmall: {
		id: 'settings.billing.plan.size.small',
		defaultMessage: 'Small',
	},
	productSizeMedium: {
		id: 'settings.billing.plan.size.medium',
		defaultMessage: 'Medium',
	},
	productSizeLarge: {
		id: 'settings.billing.plan.size.large',
		defaultMessage: 'Large',
	},
	productSizeCustom: {
		id: 'settings.billing.plan.size.custom',
		defaultMessage: 'Custom',
	},
	pyroCpuLine: {
		id: 'settings.billing.pyro.cpu',
		defaultMessage: '{shared} Shared CPUs (Bursts up to {bursts} CPUs)',
	},
	pyroRamLine: {
		id: 'settings.billing.pyro.ram',
		defaultMessage: '{gb} GB RAM',
	},
	pyroSwapLine: {
		id: 'settings.billing.pyro.swap',
		defaultMessage: '{gb} GB Swap',
	},
	pyroStorageLine: {
		id: 'settings.billing.pyro.storage',
		defaultMessage: '{gb} GB SSD',
	},
	pyroStatusProcessing: {
		id: 'settings.billing.pyro.status.processing',
		defaultMessage:
			'Your payment is being processed. Your server will activate once payment is complete.',
	},
	pyroStatusFailed: {
		id: 'settings.billing.pyro.status.failed',
		defaultMessage:
			'Your subscription payment failed. Please update your payment method, then resubscribe.',
	},
	pyroResubscribeRequestSubmittedTitle: {
		id: 'settings.billing.pyro.resubscribe.request-submitted.title',
		defaultMessage: 'Resubscription request submitted',
	},
	pyroResubscribeRequestSubmittedText: {
		id: 'settings.billing.pyro.resubscribe.request-submitted.text',
		defaultMessage:
			'If the server is currently suspended, it may take up to 10 minutes for another charge attempt to be made.',
	},
	pyroResubscribeSuccessTitle: {
		id: 'settings.billing.pyro.resubscribe.success.title',
		defaultMessage: 'Success',
	},
	pyroResubscribeSuccessText: {
		id: 'settings.billing.pyro.resubscribe.success.text',
		defaultMessage: 'Server subscription resubscribed successfully',
	},
	pyroResubscribeErrorTitle: {
		id: 'settings.billing.pyro.resubscribe.error.title',
		defaultMessage: 'Error resubscribing',
	},
	pyroResubscribeErrorText: {
		id: 'settings.billing.pyro.resubscribe.error.text',
		defaultMessage: 'An error occurred while resubscribing to your Modrinth server.',
	},
})

function getIntervalNounLabel(interval) {
	return interval === 'yearly'
		? formatMessage(messages.intervalYear)
		: formatMessage(messages.intervalMonth)
}

function getIntervalAdjectiveLabel(interval) {
	return interval === 'yearly'
		? formatMessage(messages.intervalYearly)
		: formatMessage(messages.intervalMonthly)
}

const [
	{ data: paymentMethods, refresh: refreshPaymentMethods },
	{ data: charges, refresh: refreshCharges },
	{ data: customer, refresh: refreshCustomer },
	{ data: subscriptions, refresh: refreshSubscriptions },
	{ data: productsData, refresh: refreshProducts },
	{ data: serversData, refresh: refreshServers },
] = await Promise.all([
	useAsyncData('billing/payment_methods', () =>
		useBaseFetch('billing/payment_methods', { internal: true }),
	),
	useAsyncData('billing/payments', () => useBaseFetch('billing/payments', { internal: true })),
	useAsyncData('billing/customer', () => useBaseFetch('billing/customer', { internal: true })),
	useAsyncData('billing/subscriptions', () =>
		useBaseFetch('billing/subscriptions', { internal: true }),
	),
	useAsyncData('billing/products', () => useBaseFetch('billing/products', { internal: true })),
	useAsyncData('servers', () => useServersFetch('servers')),
])

const midasProduct = ref(products.find((x) => x.metadata?.type === 'midas'))
const midasSubscription = computed(() =>
	subscriptions.value?.find(
		(x) =>
			x.status === 'provisioned' && midasProduct.value?.prices?.find((y) => y.id === x.price_id),
	),
)
const midasSubscriptionPrice = computed(() =>
	midasSubscription.value
		? midasProduct.value?.prices?.find((x) => x.id === midasSubscription.value.price_id)
		: null,
)
const midasCharge = computed(() =>
	midasSubscription.value
		? charges.value?.find((x) => x.subscription_id === midasSubscription.value.id)
		: null,
)

const oppositePrice = computed(() =>
	midasSubscription.value
		? midasProduct.value?.prices?.find((price) => price.id === midasSubscription.value.price_id)
				?.prices?.intervals?.[oppositeInterval.value]
		: undefined,
)

const pyroSubscriptions = computed(() => {
	const pyroSubs = subscriptions.value?.filter((s) => s?.metadata?.type === 'pyro') || []
	const servers = serversData.value?.servers || []

	return pyroSubs.map((subscription) => {
		const server = servers.find((s) => s.server_id === subscription.metadata.id)
		return {
			...subscription,
			serverInfo: server,
		}
	})
})

const midasPurchaseModal = ref()
const country = useUserCountry()
const price = computed(() =>
	midasProduct.value?.prices?.find((x) => x.currency_code === getCurrency(country.value)),
)

const primaryPaymentMethodId = computed(() => {
	if (customer.value?.invoice_settings?.default_payment_method) {
		return customer.value.invoice_settings.default_payment_method
	} else if (paymentMethods.value?.[0]?.id) {
		return paymentMethods.value[0].id
	} else {
		return null
	}
})

const addPaymentMethodModal = ref()

function addPaymentMethod() {
	addPaymentMethodModal.value.show(paymentMethods.value)
}

async function createSetupIntent() {
	return await useBaseFetch('billing/payment_method', {
		internal: true,
		method: 'POST',
	})
}

const removePaymentMethodIndex = ref()

const changingInterval = ref(false)

const oppositeInterval = computed(() =>
	midasCharge.value?.subscription_interval === 'yearly' ? 'monthly' : 'yearly',
)

async function switchMidasInterval(interval) {
	changingInterval.value = true
	startLoading()
	try {
		await useBaseFetch(`billing/subscription/${midasSubscription.value.id}`, {
			internal: true,
			method: 'PATCH',
			body: {
				interval,
			},
		})
		await refresh()
	} catch (error) {
		console.error('Error switching Modrinth+ payment interval:', error)
	}
	stopLoading()
	changingInterval.value = false
}

async function editPaymentMethod(index, primary) {
	startLoading()
	try {
		await useBaseFetch(`billing/payment_method/${paymentMethods.value[index].id}`, {
			internal: true,
			method: 'PATCH',
			data: {
				primary,
			},
		})
		await refresh()
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
	stopLoading()
}

async function removePaymentMethod(index) {
	startLoading()
	try {
		await useBaseFetch(`billing/payment_method/${paymentMethods.value[index].id}`, {
			internal: true,
			method: 'DELETE',
		})
		await refresh()
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
	stopLoading()
}

const cancelSubscriptionId = ref(null)
async function cancelSubscription(id, cancelled) {
	startLoading()
	try {
		await useBaseFetch(`billing/subscription/${id}`, {
			internal: true,
			method: 'PATCH',
			body: {
				cancelled,
			},
		})
		await refresh()
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
	stopLoading()
}

const getPyroProduct = (subscription) => {
	if (!subscription || !productsData.value) return null
	return productsData.value.find((p) => p.prices?.some((x) => x.id === subscription.price_id))
}

// Get product by a price ID (useful for pending next-charge changes)
const getProductFromPriceId = (priceId) => {
	if (!priceId || !productsData.value) return null
	return productsData.value.find((p) => p.prices?.some((x) => x.id === priceId))
}

const getPyroCharge = (subscription) => {
	if (!subscription || !charges.value) return null
	return charges.value.find(
		(charge) => charge.subscription_id === subscription.id && charge.status !== 'succeeded',
	)
}

const getProductSize = (product) => {
	if (!product || !product.metadata) return formatMessage(messages.productSizeUnknown)
	const ramSize = product.metadata.ram
	if (ramSize === 4096) return formatMessage(messages.productSizeSmall)
	if (ramSize === 6144) return formatMessage(messages.productSizeMedium)
	if (ramSize === 8192) return formatMessage(messages.productSizeLarge)
	return formatMessage(messages.productSizeCustom)
}

const getProductPrice = (product, interval) => {
	if (!product || !product.prices) return null
	const countryValue = country.value
	return (
		product.prices.find(
			(p) => p.currency_code === getCurrency(countryValue) && p.prices?.intervals?.[interval],
		) ??
		product.prices.find((p) => p.currency_code === 'USD' && p.prices?.intervals?.[interval]) ??
		product.prices[0]
	)
}

const getPlanChangeVerb = (currentProduct, nextProduct) => {
	const curRam = currentProduct?.metadata?.ram ?? 0
	const nextRam = nextProduct?.metadata?.ram ?? 0

	return nextRam < curRam ? 'downgrade' : 'upgrade'
}

const modalCancel = ref(null)

const upgradeModal = ref(null)
const showPyroUpgradeModal = (subscription) => {
	upgradeModal.value?.open(subscription?.metadata?.id)
}

const resubscribePyro = async (subscriptionId, wasSuspended) => {
	try {
		await useBaseFetch(`billing/subscription/${subscriptionId}`, {
			internal: true,
			method: 'PATCH',
			body: {
				cancelled: false,
			},
		})
		await refresh()
		if (wasSuspended) {
			addNotification({
				title: formatMessage(messages.pyroResubscribeRequestSubmittedTitle),
				text: formatMessage(messages.pyroResubscribeRequestSubmittedText),
				type: 'success',
			})
		} else {
			addNotification({
				title: formatMessage(messages.pyroResubscribeSuccessTitle),
				text: formatMessage(messages.pyroResubscribeSuccessText),
				type: 'success',
			})
		}
	} catch {
		addNotification({
			title: formatMessage(messages.pyroResubscribeErrorTitle),
			text: formatMessage(messages.pyroResubscribeErrorText),
			type: 'error',
		})
	}
}

const refresh = async () => {
	await Promise.all([
		refreshPaymentMethods(),
		refreshCharges(),
		refreshCustomer(),
		refreshSubscriptions(),
		refreshProducts(),
		refreshServers(),
	])
}

function showCancellationSurvey(subscription) {
	if (!subscription) {
		console.warn('No survey notice to open')
		return
	}

	const product = getPyroProduct(subscription)
	const priceObj = product?.prices?.find((x) => x.id === subscription.price_id)
	const price = priceObj?.prices?.intervals?.[subscription.interval]
	const currency = priceObj?.currency_code

	const popupOptions = {
		layout: 'modal',
		width: 700,
		autoClose: 2000,
		hideTitle: true,
		hiddenFields: {
			username: auth.value?.user?.username,
			user_id: auth.value?.user?.id,
			user_email: auth.value?.user?.email,
			subscription_id: subscription.id,
			price_id: subscription.price_id,
			interval: subscription.interval,
			started: subscription.created,
			plan_ram: product?.metadata.ram / 1024,
			plan_cpu: product?.metadata.cpu,
			price: price ? `${price / 100}` : 'unknown',
			currency: currency ?? 'unknown',
		},
		onOpen: () => console.log(`Opened cancellation survey for: ${subscription.id}`),
		onClose: () => console.log(`Closed cancellation survey for: ${subscription.id}`),
		onSubmit: (payload) => {
			console.log('Form submitted, cancelling server.', payload)
			cancelSubscription(subscription.id, true)
		},
	}

	const formId = 'mOr7lM'

	try {
		if (window.Tally?.openPopup) {
			console.log(
				`Opening Tally popup for servers subscription ${subscription.id} (form ID: ${formId})`,
			)
			window.Tally.openPopup(formId, popupOptions)
		} else {
			console.warn('Tally script not yet loaded')
			cancelSubscription(subscription.id, true)
		}
	} catch (e) {
		console.error('Error opening Tally popup:', e)
	}
}

useHead({
	script: [
		{
			src: 'https://tally.so/widgets/embed.js',
			defer: true,
		},
	],
})

const getPendingChange = (subscription) => {
	const charge = getPyroCharge(subscription)
	if (!charge || charge.status !== 'open') return null

	const nextProduct = getProductFromPriceId(charge.price_id)
	if (!nextProduct || charge.price_id === subscription.price_id) {
		// Not a plan change, but interval could change
		if (charge.subscription_interval && charge.subscription_interval !== subscription.interval) {
			return {
				planSize: getProductSize(getPyroProduct(subscription)),
				cpu: getPyroProduct(subscription)?.metadata?.cpu / 2,
				cpuBurst: getPyroProduct(subscription)?.metadata?.cpu,
				ramGb: (getPyroProduct(subscription)?.metadata?.ram || 0) / 1024,
				swapGb: (getPyroProduct(subscription)?.metadata?.swap || 0) / 1024 || undefined,
				storageGb: (getPyroProduct(subscription)?.metadata?.storage || 0) / 1024 || undefined,
				date: charge.due,
				intervalChange: charge.subscription_interval,
				verb: 'Switches',
			}
		}
		return null
	}

	const curProduct = getPyroProduct(subscription)
	const verb = getPlanChangeVerb(curProduct, nextProduct)
	const cpu = nextProduct?.metadata?.cpu ?? 0
	const ram = nextProduct?.metadata?.ram ?? 0
	const swap = nextProduct?.metadata?.swap ?? 0
	const storage = nextProduct?.metadata?.storage ?? 0

	return {
		planSize: getProductSize(nextProduct),
		cpu: cpu / 2,
		cpuBurst: cpu,
		ramGb: ram / 1024,
		swapGb: swap ? swap / 1024 : undefined,
		storageGb: storage ? storage / 1024 : undefined,
		date: charge.due,
		intervalChange:
			charge.subscription_interval && charge.subscription_interval !== subscription.interval
				? charge.subscription_interval
				: null,
		verb,
	}
}
</script>
