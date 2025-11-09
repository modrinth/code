<script setup lang="ts">
import { ServersManagePageIndex } from '@modrinth/ui'

import { useGeneratedState } from '~/composables/generated'

definePageMeta({
	middleware: 'auth',
})

useHead({
	title: 'Servers - Modrinth',
})

const config = useRuntimeConfig()
const generatedState = useGeneratedState()
</script>

<template>
	<ServersManagePageIndex
		:stripe-publishable-key="config.public.stripePublishableKey"
		:site-url="config.public.siteUrl"
		:products="generatedState.products || []"
	/>
</template>
