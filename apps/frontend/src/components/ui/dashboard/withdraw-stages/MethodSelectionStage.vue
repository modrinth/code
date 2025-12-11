<template>
	<div class="flex flex-col gap-4">
		<Admonition v-if="shouldShowTaxLimitWarning" type="warning">
			<IntlFormatted
				:message-id="messages.taxLimitWarning"
				:values="{
					amount: formatMoney(maxWithdrawAmount),
				}"
			>
				<template #b="{ children }">
					<span class="font-semibold">
						<component :is="() => normalizeChildren(children)" />
					</span>
				</template>
				<template #tax-link="{ children }">
					<span class="cursor-pointer text-link" @click="onShowTaxForm">
						<component :is="() => normalizeChildren(children)" />
					</span>
				</template>
			</IntlFormatted>
		</Admonition>
		<div class="flex flex-col">
			<div class="flex flex-col gap-2.5">
				<div class="flex flex-row gap-1 align-middle">
					<span class="align-middle font-semibold text-contrast">{{
						formatMessage(messages.region)
					}}</span>
					<UnknownIcon
						v-tooltip="formatMessage(messages.regionTooltip)"
						class="mt-auto size-5 text-secondary"
					/>
				</div>
				<Combobox
					:model-value="selectedCountryCode"
					:options="countries"
					:placeholder="formatMessage(messages.countryPlaceholder)"
					searchable
					:search-placeholder="formatMessage(messages.countrySearchPlaceholder)"
					:max-height="240"
					class="h-12"
					@update:model-value="handleCountryChange"
				/>
			</div>
			<div class="flex flex-col gap-2.5">
				<div class="flex flex-row gap-1 align-middle">
					<span class="align-middle font-semibold text-contrast">{{
						formatMessage(messages.selectMethod)
					}}</span>
				</div>
				<div v-if="loading" class="flex min-h-[120px] items-center justify-center">
					<SpinnerIcon class="size-8 animate-spin text-contrast" />
				</div>
				<template v-else>
					<ButtonStyled
						v-for="method in paymentOptions"
						:key="method.value"
						:color="withdrawData.selection.method === method.value ? 'green' : 'standard'"
						:highlighted="withdrawData.selection.method === method.value"
						type="chip"
					>
						<button
							class="!justify-start !gap-2 !text-left sm:!h-10"
							@click="handleMethodSelection(method)"
						>
							<component :is="method.icon" class="shrink-0" />
							<span class="flex-1 truncate text-sm sm:text-[1rem]">
								{{ typeof method.label === 'string' ? method.label : formatMessage(method.label) }}
							</span>
							<span class="ml-auto shrink-0 text-xs font-normal text-secondary sm:text-sm">{{
								method.fee
							}}</span>
						</button>
					</ButtonStyled>
				</template>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { SpinnerIcon, UnknownIcon } from '@modrinth/assets'
import {
	Admonition,
	ButtonStyled,
	Combobox,
	injectNotificationManager,
	normalizeChildren,
	useDebugLogger,
} from '@modrinth/ui'
import { formatMoney } from '@modrinth/utils'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { IntlFormatted } from '@vintl/vintl/components'
import { useGeolocation } from '@vueuse/core'

import { useCountries, useFormattedCountries, useUserCountry } from '@/composables/country.ts'
import { type PayoutMethod, useWithdrawContext } from '@/providers/creator-withdraw.ts'

const debug = useDebugLogger('MethodSelectionStage')
const {
	withdrawData,
	availableMethods,
	paymentOptions,
	balance,
	maxWithdrawAmount,
	paymentMethodsCache,
} = useWithdrawContext()
const userCountry = useUserCountry()
const allCountries = useCountries()
const { coords } = useGeolocation()
const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()
const auth = await useAuth()

const messages = defineMessages({
	taxLimitWarning: {
		id: 'dashboard.creator-withdraw-modal.method-selection.tax-limit-warning',
		defaultMessage:
			'Your withdraw limit is <b>{amount}</b>, <tax-link>complete a tax form</tax-link> to withdraw more.',
	},
	region: {
		id: 'dashboard.creator-withdraw-modal.method-selection.region',
		defaultMessage: 'Region',
	},
	regionTooltip: {
		id: 'dashboard.creator-withdraw-modal.method-selection.region-tooltip',
		defaultMessage: 'Some payout methods are not available in certain regions.',
	},
	countryPlaceholder: {
		id: 'dashboard.creator-withdraw-modal.method-selection.country-placeholder',
		defaultMessage: 'Select your country',
	},
	countrySearchPlaceholder: {
		id: 'dashboard.creator-withdraw-modal.method-selection.country-search-placeholder',
		defaultMessage: 'Search countries...',
	},
	selectMethod: {
		id: 'dashboard.creator-withdraw-modal.method-selection.select-method',
		defaultMessage: 'Select withdraw method',
	},
	errorTitle: {
		id: 'dashboard.creator-withdraw-modal.method-selection.error-title',
		defaultMessage: 'Failed to load payment methods',
	},
	errorText: {
		id: 'dashboard.creator-withdraw-modal.method-selection.error-text',
		defaultMessage: 'Unable to fetch available payment methods. Please try again later.',
	},
})

defineProps<{
	onShowTaxForm: () => void
}>()

const emit = defineEmits<{
	(e: 'close-modal'): void
}>()

const countries = useFormattedCountries()

const selectedCountryCode = computed(() => withdrawData.value.selection.country?.id)

const shouldShowTaxLimitWarning = computed(() => {
	const balanceValue = balance.value
	if (!balanceValue) return false

	const formIncomplete = balanceValue.form_completion_status !== 'complete'
	const wouldHitLimit = (balanceValue.withdrawn_ytd ?? 0) + (balanceValue.available ?? 0) >= 600

	return formIncomplete && wouldHitLimit
})

const loading = ref(false)

watch(
	() => withdrawData.value.selection.country,
	async (country) => {
		debug('Watch triggered, country:', country)
		if (!country) {
			availableMethods.value = []
			return
		}

		if (paymentMethodsCache.value[country.id]) {
			debug('Using cached methods for', country.id)
			availableMethods.value = paymentMethodsCache.value[country.id]
			return
		}

		loading.value = true
		debug('Fetching payout methods for country:', country.id)

		try {
			const methods = (await useBaseFetch('payout/methods', {
				apiVersion: 3,
				query: { country: country.id },
			})) as PayoutMethod[]
			debug('Received payout methods:', methods)

			paymentMethodsCache.value[country.id] = methods
			availableMethods.value = methods
		} catch (e) {
			console.error('[MethodSelectionStage] Failed to fetch payout methods:', e)
			addNotification({
				title: formatMessage(messages.errorTitle),
				text: formatMessage(messages.errorText),
				type: 'error',
			})
			emit('close-modal')
		} finally {
			loading.value = false
		}
	},
	{ immediate: true },
)

function handleMethodSelection(option: {
	value: string
	methodId: string | undefined
	type: string
}) {
	withdrawData.value.selection.method = option.value
	withdrawData.value.selection.methodId = option.methodId ?? null

	if (option.type === 'tremendous') {
		withdrawData.value.selection.provider = 'tremendous'
	} else if (option.type === 'fiat' || option.type === 'crypto') {
		withdrawData.value.selection.provider = 'muralpay'
	} else if (option.type === 'paypal') {
		withdrawData.value.selection.provider = 'paypal'
	} else if (option.type === 'venmo') {
		withdrawData.value.selection.provider = 'venmo'
	} else {
		withdrawData.value.selection.provider = 'muralpay'
	}
}

watch(paymentOptions, (newOptions) => {
	withdrawData.value.selection.method = null
	withdrawData.value.selection.methodId = null
	withdrawData.value.selection.provider = null

	if (newOptions.length === 1) {
		const option = newOptions[0]
		handleMethodSelection(option)
	}
})

watch(
	() => withdrawData.value.selection.provider,
	(newProvider) => {
		if (newProvider === 'tremendous') {
			const userEmail = (auth.value.user as any)?.email || ''
			withdrawData.value.providerData = {
				type: 'tremendous',
				deliveryEmail: userEmail,
				giftCardDetails: null,
				currency: undefined,
			}
		} else if (newProvider === 'muralpay') {
			withdrawData.value.providerData = {
				type: 'muralpay',
				kycData: {} as any,
				accountDetails: {},
			}
		} else if (newProvider === 'paypal' || newProvider === 'venmo') {
			withdrawData.value.providerData = {
				type: newProvider,
			}
		}
	},
)

function handleCountryChange(countryCode: string | null) {
	debug('handleCountryChange called with:', countryCode)
	if (countryCode) {
		const normalizedCode = countryCode.toUpperCase()
		const country = allCountries.value.find((c) => c.alpha2 === normalizedCode)
		debug('Found country:', country)
		if (country) {
			withdrawData.value.selection.country = {
				id: country.alpha2,
				name: country.alpha2 === 'TW' ? 'Taiwan' : country.nameShort,
			}
			debug('Set selectedCountry to:', withdrawData.value.selection.country)
		}
	} else {
		withdrawData.value.selection.country = null
	}
}

debug('Setup: userCountry.value =', userCountry.value)
debug('Setup: current selectedCountry =', withdrawData.value.selection.country)

if (!withdrawData.value.selection.country) {
	const defaultCountryCode = userCountry.value || 'US'
	debug('Setup: calling handleCountryChange with', defaultCountryCode)
	handleCountryChange(defaultCountryCode)
	debug('Setup: selectedCountryCode computed =', selectedCountryCode.value)
}

async function getCountryFromGeoIP(lat: number, lon: number): Promise<string | null> {
	try {
		const response = await fetch(
			`https://api.bigdatacloud.net/data/reverse-geocode-client?latitude=${lat}&longitude=${lon}&localityLanguage=en`,
		)
		const data = await response.json()
		return data.countryCode || null
	} catch {
		return null
	}
}

onMounted(async () => {
	if (withdrawData.value.selection.country?.id === 'US' && !userCountry.value) {
		if (coords.value.latitude && coords.value.longitude) {
			const geoCountry = await getCountryFromGeoIP(coords.value.latitude, coords.value.longitude)
			if (geoCountry) {
				handleCountryChange(geoCountry)
			}
		}
	}
})
</script>
