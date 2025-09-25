<script setup lang="ts">
import { Heading, Link as VLink, Text } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail
		title="Report of {report.title} has been submitted"
		:manual-links="[
			{ link: 'https://modrinth.com/legal/rules', label: 'Community Guidelines' },
			{
				link: 'https://modrinth.com/dashboard/report/{newreport.id}',
				label: 'View report in dashboard',
			},
			{ link: 'https://support.modrinth.com', label: 'Support Portal' },
		]"
	>
		<Heading as="h1" class="mb-2 text-2xl font-bold"
			>Report of {report.title} has been submitted</Heading
		>

		<Text class="text-base">Hi {user.name},</Text>

		<Text class="text-base">
			We've received your report of {report.title} and our moderation team will review it shortly.
		</Text>

		<Text class="text-base">
			Our team takes all reports seriously and will investigate according to our
			<VLink href="https://modrinth.com/legal/rules" class="text-green underline"
				>Content Rules</VLink
			>, <VLink href="https://modrinth.com/legal/terms">Terms of Service</VLink> and
			<VLink href="https://modrinth.com/legal/copyright">Copyright Policy</VLink>. You'll receive an
			email update once we've completed our review.
		</Text>

		<Text class="text-base">
			If you have any additional information about this report, you can
			<VLink
				href="https://modrinth.com/dashboard/report/{newreport.id}"
				class="text-green underline"
				>view it here</VLink
			>.
		</Text>

		<Text class="text-base">
			Thank you for helping keep Modrinth safe and welcoming for everyone.
		</Text>
	</StyledEmail>
</template>
