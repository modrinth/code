<script setup lang="ts">
import { Heading, Text, Link as VLink } from '@vue-email/components'

import StyledEmail from '@/emails/shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail
		title="Report submitted successfully"
		:manual-links="[
			{ link: 'https://modrinth.com/legal/rules', label: 'Community Guidelines' },
			{
				link: 'https://modrinth.com/dashboard/report/{newreport.id}',
				label: 'View report in dashboard',
			},
			{ link: 'https://support.modrinth.com', label: 'Support Portal' },
		]"
	>
		<Heading as="h1" class="mb-2 text-2xl font-bold">Report submitted successfully</Heading>

		<Text class="text-base">Hi {user.name},</Text>

		<Text class="text-base">
			Thank you for reporting content on Modrinth. We've received your report and our moderation
			team will review it shortly.
		</Text>

		<Text class="text-base">
			Our team takes all reports seriously and will investigate according to our
			<VLink href="https://modrinth.com/legal/rules" class="text-green underline"
				>Content Rules</VLink
			>. You'll receive an update once we've completed our review.
		</Text>

		<Text class="text-base">
			If you have any additional information about this report, you can
			<VLink
				href="https://modrinth.com/dashboard/report/{newreport.id}"
				class="text-green underline"
				>view it in your dashboard</VLink
			>
			or
			<VLink href="https://support.modrinth.com" class="text-green underline"
				>contact our support team</VLink
			>.
		</Text>

		<Text class="text-base">
			Thank you for helping keep Modrinth safe and welcoming for everyone.
		</Text>
	</StyledEmail>
</template>
