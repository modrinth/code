<script setup lang="ts">
import { Heading, Text } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail title="We’ve added time to your server">
		<Heading as="h1" class="mb-2 text-2xl font-bold">We’ve added time to your server</Heading>

		<Text class="text-muted text-base">Hi {user.name},</Text>
		<Text class="text-muted text-base">{credit.header_message}</Text>

		<Text class="text-muted text-base">
			To make up for it, we've added {credit.days_formatted} to your {credit.subscription.type}
			subscription.
		</Text>
		<Text class="text-muted text-base">
			Your next charge was scheduled for {credit.previous_due} and will now be on {credit.next_due}.
		</Text>

		<Text class="text-muted text-base">Thank you for supporting us,</Text>
		<Text class="text-muted text-base">The Modrinth Team</Text>
	</StyledEmail>
</template>
