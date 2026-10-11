<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ServerStackIcon } from '@modrinth/assets'
import { injectModrinthClient, ServersManagePageIndex } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import { useRootBreadcrumb } from '@/providers/breadcrumbs'

import { config } from '../config'
import LocalServers from './hosting/LocalServers.vue'

const stripePublishableKey = (config.stripePublishableKey as string) || ''

const client = injectModrinthClient()

useRootBreadcrumb({
	slot: 'root',
	id: 'servers',
	label: 'Hosting',
	to: '/hosting/manage/',
	visual: { type: 'icon', component: ServerStackIcon },
})

const { data: products } = useQuery({
	queryKey: ['billing', 'products'],
	queryFn: () => client.labrinth.billing_internal.getProducts(),
	enabled: !client.selfHostedHosting,
})

const resolvedProducts = computed<Labrinth.Billing.Internal.Product[]>(() => products.value ?? [])
</script>

<template>
	<LocalServers v-if="client.selfHostedHosting" />
	<ServersManagePageIndex
		v-else
		:stripe-publishable-key="stripePublishableKey"
		:products="resolvedProducts"
	/>
</template>
