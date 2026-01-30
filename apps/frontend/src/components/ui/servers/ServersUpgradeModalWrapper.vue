<template>
	<ServersUpgradeModalWrapperBase
		ref="wrapperRef"
		:stripe-publishable-key="config.public.stripePublishableKey"
		:site-url="config.public.siteUrl"
		:products="generatedState.products || []"
	/>
</template>

<script setup lang="ts">
// TODO: Remove this wrapper when we figure out how to do cross platform state + stripe
import { ServersUpgradeModalWrapper as ServersUpgradeModalWrapperBase } from '@modrinth/ui'

import { useGeneratedState } from '~/composables/generated'

const config = useRuntimeConfig()
const generatedState = useGeneratedState()

const wrapperRef = ref<InstanceType<typeof ServersUpgradeModalWrapperBase> | null>(null)

defineExpose({
	open: (id?: string) => wrapperRef.value?.open(id),
})
</script>
