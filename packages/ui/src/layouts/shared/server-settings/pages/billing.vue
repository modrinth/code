<template>
	<div class="universal-card">
		<p>You can manage your server's billing from Settings > Billing and subscriptions.</p>
		<ButtonStyled>
			<AutoLink :to="billingHref">Go to Billing</AutoLink>
		</ButtonStyled>
	</div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { AutoLink, ButtonStyled } from '#ui/components'
import { injectModrinthServerContext } from '#ui/providers/server-context'

import { injectServerSettings } from '../providers'

const { serverId } = injectModrinthServerContext()
const serverSettings = injectServerSettings()

const billingHref = computed(() => {
	const path = `/settings/billing#server-${serverId}`
	return serverSettings.isApp.value ? `https://modrinth.com${path}` : path
})
</script>
