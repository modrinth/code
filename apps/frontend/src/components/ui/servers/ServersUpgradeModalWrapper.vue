<template>
	<ServersUpgradeModalWrapperBase
		ref="wrapperRef"
		:stripe-publishable-key="config.public.stripePublishableKey"
		:site-url="config.public.siteUrl"
		:products="(products ?? []) as Labrinth.Billing.Internal.Product[]"
	/>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
// TODO: Remove this wrapper when we figure out how to do cross platform state + stripe
import { ServersUpgradeModalWrapper as ServersUpgradeModalWrapperBase } from '@modrinth/ui'

import { products } from '~/generated/state.json'

const config = useRuntimeConfig()

const wrapperRef = ref<InstanceType<typeof ServersUpgradeModalWrapperBase> | null>(null)

defineExpose({
	open: (id?: string) => wrapperRef.value?.open(id),
})
</script>
