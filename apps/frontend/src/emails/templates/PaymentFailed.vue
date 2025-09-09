<script setup lang="ts">
import { Button, Heading, Link as VLink, Text } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail title="Payment failed for {paymentfailed.service}">
		<Heading as="h1" class="mb-2 text-2xl font-bold">
			Payment failed for {paymentfailed.service}
		</Heading>

		<Text class="text-muted text-base">Hi {user.name},</Text>
		<Text class="text-muted text-base">
			Our attempt to collect payment for {paymentfailed.amount} from the payment card on file was
			unsuccessful. Please update your billing settings to avoid service termination.
		</Text>
		<Button
			href="{billing.url}"
			target="_blank"
			class="text-accentContrast inline-block rounded-[12px] bg-brand px-4 py-3 text-[14px] font-bold"
		>
			Update billing settings
		</Button>

		<VLink href="{billing.url}">
			<Text class="text-muted mt-2 break-words text-xs font-bold">{billing.url}</Text>
		</VLink>
	</StyledEmail>
</template>
