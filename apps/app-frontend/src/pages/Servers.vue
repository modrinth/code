<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { injectModrinthClient, ServersManagePageIndex } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'
import { config } from '../config'

const stripePublishableKey = (config.stripePublishableKey as string) || ''

const client = injectModrinthClient()

const { data: products } = useQuery({
	queryKey: ['billing', 'products'],
	queryFn: () => client.labrinth.billing_internal.getProducts(),
})

const resolvedProducts = computed<Labrinth.Billing.Internal.Product[]>(() => products.value ?? [])
</script>

<template>
	<ServersManagePageIndex
		:stripe-publishable-key="stripePublishableKey"
		:site-url="'https://modrinth.com'"
		:products="resolvedProducts"
	/>
</template>
