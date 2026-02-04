<template>
	<div>
		<section class="card">
			<Breadcrumbs
				:current-title="formatMessage(messages.title)"
				:link-stack="[
					{ href: '/settings/billing', label: formatMessage(commonSettingsMessages.billing) },
				]"
			/>
			<h2>{{ formatMessage(messages.title) }}</h2>
			<p>{{ formatMessage(messages.description) }}</p>
			<div
				v-for="charge in charges"
				:key="charge.id"
				class="universal-card recessed flex items-center justify-between gap-4"
			>
				<div class="flex flex-col gap-1">
					<div class="flex items-center gap-1">
						<span class="font-bold text-primary">
							<template v-if="charge.product?.metadata?.type === 'midas'">
								{{ formatMessage(messages.productMidas) }}
							</template>
							<template v-else-if="charge.product?.metadata?.type === 'pyro'">
								{{ formatMessage(messages.productPyro) }}
							</template>
							<template v-else> {{ formatMessage(messages.productMedalTrial) }} </template>
							<template v-if="charge.subscription_interval">
								{{ charge.subscription_interval }}
							</template>
						</span>
						⋅
						<span>{{ formatPrice(vintl.locale, charge.amount, charge.currency_code) }}</span>
					</div>
					<div class="flex items-center gap-1">
						<Badge :color="charge.status === 'succeeded' ? 'green' : 'red'" :type="charge.status" />
						⋅
						{{ $dayjs(charge.due).format('YYYY-MM-DD') }}
					</div>
				</div>
			</div>
		</section>
	</div>
</template>
<script setup>
import { Badge, Breadcrumbs, commonSettingsMessages, defineMessages, useVIntl } from '@modrinth/ui'
import { formatPrice } from '@modrinth/utils'

import { products } from '~/generated/state.json'

definePageMeta({
	middleware: 'auth',
})

const { formatMessage } = useVIntl()

const messages = defineMessages({
	description: {
		id: 'settings.billing.charges.description',
		defaultMessage: 'All of your past charges to your Modrinth account will be listed here:',
	},
	productMidas: {
		id: 'settings.billing.charges.product.midas',
		defaultMessage: 'Modrinth Plus',
	},
	productPyro: {
		id: 'settings.billing.charges.product.pyro',
		defaultMessage: 'Modrinth Hosting',
	},
	productMedalTrial: {
		id: 'settings.billing.charges.product.medal-trial',
		defaultMessage: 'Medal Server Trial',
	},
})

const { data: charges } = await useAsyncData(
	'billing/payments',
	() => useBaseFetch('billing/payments', { internal: true }),
	{
		transform: (charges) => {
			return charges
				.filter((charge) => charge.status !== 'open' && charge.status !== 'cancelled')
				.map((charge) => {
					const product = products.find((product) =>
						product.prices.some((price) => price.id === charge.price_id),
					)

					charge.product = product

					return charge
				})
		},
	},
)
</script>
