<script setup lang="ts">
import { Button, Heading, Link as VLink, Section, Text } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail
		title="Revenue available to withdraw!"
		:manual-links="[
			{ link: 'https://modrinth.com/dashboard/revenue', label: 'Revenue dashboard' },
			{ link: 'https://support.modrinth.com', label: 'Support Portal' },
		]"
	>
		<Heading as="h1" class="mb-2 text-2xl font-bold">Revenue available to withdraw!</Heading>

		<Text class="text-base">Hi {user.name},</Text>

		<Text class="text-base">
			The {payout.amount} earned during {payout.period} has been processed and is now available to
			withdraw from your account.
		</Text>

		<Section class="mb-4 mt-4">
			<Button
				href="https://modrinth.com/dashboard/revenue"
				target="_blank"
				class="text-accentContrast inline-block rounded-[12px] bg-brand pb-3 pl-4 pr-4 pt-3 text-[14px] font-bold"
			>
				View revenue dashboard
			</Button>
		</Section>

		<Text class="text-base">
			If you have any questions about the creator rewards program, please contact support through
			the
			<VLink href="https://support.modrinth.com" class="text-green underline">Support Portal</VLink>
			or by replying to this email.
		</Text>

		<Text class="text-base">Thank you for being a creator on Modrinth!</Text>
	</StyledEmail>
</template>
