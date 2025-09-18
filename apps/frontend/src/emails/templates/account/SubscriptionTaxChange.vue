<script setup lang="ts">
import { Heading, Text, Link as VLink } from '@vue-email/components'

import StyledEmail from '@/emails/shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail
		title="Price change for {taxnotification.service}"
		:manual-links="[{ link: 'https://support.modrinth.com', label: 'Support Portal' }]"
	>
		<Heading as="h1" class="mb-2 text-2xl font-bold">
			Price change for {taxnotification.service}
		</Heading>

		<Text class="text-muted text-base">Hi {user.name},</Text>
		<Text class="text-muted text-base">
			We're writing to let you know that taxes for your {taxnotification.service} subscription will
			affect your upcoming {taxnotification.billing_interval} billing cycle.
		</Text>

		<Text class="text-muted text-base">
			Previously, you were paying {taxnotification.old_tax_amount} in tax. Your next charge will
			include {taxnotification.new_tax_amount} in tax starting on {taxnotification.due}.
		</Text>

		<Text class="text-muted text-base">
			No action is required on your part. This change will automatically take effect on
			{taxnotification.due} with your next {taxnotification.billing_interval} charge.
		</Text>

		<Text class="text-muted text-base">
			If you have questions or need help with your subscription, reply to this email or visit our
			<VLink href="https://support.modrinth.com" class="text-green underline">Support Portal</VLink
			>.
		</Text>
	</StyledEmail>
</template>
