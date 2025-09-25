<script setup lang="ts">
import { Heading, Link as VLink, Text } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail
		title="Price change for {taxnotification.service}"
		:manual-links="[{ link: 'https://support.modrinth.com', label: 'Support Portal' }]"
		:support-info="['{subscription.id}']"
	>
		<Heading as="h1" class="mb-2 text-2xl font-bold">
			Price change for {taxnotification.service}
		</Heading>

		<Text class="text-muted text-base">Hi {user.name},</Text>
		<Text class="text-muted text-base">
			We're writing to let you know about an update to your {taxnotification.service} subscription.
		</Text>

		<Text class="text-muted text-base">
			Your plan and rate are staying the same. The only change is that your region is now subject to
			tax. Starting {taxnotification.due}, your next charge will include
			{taxnotification.new_tax_amount} in tax.
		</Text>

		<Text class="text-muted text-base">
			You do not need to take any action. This change will apply automatically on
			{taxnotification.due}.
		</Text>

		<Text class="text-muted text-base">
			Thank you for choosing Modrinth! If you have any questions or need help with your
			subscription, reply to this email or visit our
			<VLink href="https://support.modrinth.com" class="text-green underline">Support Portal</VLink
			>.
		</Text>

		<Text class="text-muted text-base">
			Best,<br />
			The Modrinth Team
		</Text>
	</StyledEmail>
</template>
