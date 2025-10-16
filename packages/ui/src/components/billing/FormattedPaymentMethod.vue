<script setup lang="ts">
import { CardIcon, CurrencyIcon, PayPalIcon, UnknownIcon } from '@modrinth/assets'
import { useVIntl } from '@vintl/vintl'
import type Stripe from 'stripe'

import { commonMessages, paymentMethodMessages } from '../../utils'

const { formatMessage } = useVIntl()
defineProps<{
	method: Stripe.PaymentMethod
}>()
</script>

<template>
	<template v-if="'type' in method">
		<CardIcon v-if="method.type === 'card'" class="size-[1.5em]" />
		<CurrencyIcon v-else-if="method.type === 'cashapp'" class="size-[1.5em]" />
		<PayPalIcon v-else-if="method.type === 'paypal'" class="size-[1.5em]" />
		<UnknownIcon v-else class="size-[1.5em]" />
		<span v-if="method.type === 'card' && 'card' in method && method.card">
			{{
				formatMessage(commonMessages.paymentMethodCardDisplay, {
					card_brand:
						formatMessage(paymentMethodMessages[method.card.brand]) ??
						formatMessage(paymentMethodMessages.unknown),
					last_four: method.card.last4,
				})
			}}
		</span>
		<template v-else>
			{{
				formatMessage(paymentMethodMessages[method.type]) ??
				formatMessage(paymentMethodMessages.unknown)
			}}
		</template>

		<span v-if="method.type === 'cashapp' && 'cashapp' in method && method.cashapp">
			({{ method.cashapp.cashtag }})
		</span>
		<span v-else-if="method.type === 'paypal' && 'paypal' in method && method.paypal">
			({{ method.paypal.payer_email }})
		</span>
	</template>
</template>
