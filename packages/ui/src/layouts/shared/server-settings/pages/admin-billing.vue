<template>
	<div class="universal-card">
		<p>You can manage this server owner's billing from the admin billing page.</p>
		<ButtonStyled>
			<AutoLink :to="adminBillingHref">Go to Admin Billing</AutoLink>
		</ButtonStyled>
	</div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import AutoLink from '#ui/components/base/AutoLink.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import { injectModrinthServerContext } from '#ui/providers/server-context'

import { injectServerSettings } from '../providers'

const { server } = injectModrinthServerContext()
const serverSettings = injectServerSettings()

const ownerId = computed(() => server.value?.owner_id ?? 'unknown')
const adminBillingHref = computed(() => {
	const path = `/admin/billing/${ownerId.value}`
	return serverSettings.isApp.value ? `https://modrinth.com${path}` : path
})
</script>
