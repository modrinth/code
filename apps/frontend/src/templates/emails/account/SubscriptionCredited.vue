<script setup lang="ts">
import { Heading, Text } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail
		title="We added {credit.days} days to your subscription"
		:manual-links="[{ link: 'https://support.modrinth.com', label: 'Support Portal' }]"
		:support-info="['{subscription.id}']"
	>
		<Heading as="h1" class="mb-2 text-2xl font-bold">
			We added {credit.days} days to your subscription
		</Heading>

		<Text class="text-muted text-base">Hi {user.name},</Text>
		<Text class="text-muted text-base">
			We've credited your subscription by {credit.days} days.
		</Text>
		<Text class="text-muted text-base"> Previous due date: {credit.previous_due} </Text>
		<Text class="text-muted text-base"> New due date: {credit.next_due} </Text>

		<Text class="text-muted text-base"> Thank you for being a Modrinth customer! </Text>

		<Text class="text-muted text-base">
			Best,<br />
			The Modrinth Team
		</Text>
	</StyledEmail>
</template>
