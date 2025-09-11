<script setup lang="ts">
import { Heading, Link as VLink, Section, Text } from '@vue-email/components'

import StyledEmail from '@/emails/shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail
		title="Report status updated"
		:manual-links="[
			{ link: 'https://modrinth.com/legal/rules', label: 'Community Guidelines' },
			{
				link: 'https://modrinth.com/dashboard/report/{report.id}',
				label: 'View report in dashboard',
			},
			{ link: 'https://support.modrinth.com', label: 'Support Portal' },
		]"
	>
		<Heading as="h1" class="mb-2 text-2xl font-bold">Report status updated</Heading>

		<Text class="text-base">Hi {user.name},</Text>

		<Text class="text-base">Your report, {report.id}, from {report.date} has been updated.</Text>

		<Section class="bg-bg-super mb-4 mt-4 rounded-lg border border-divider pb-4 pl-4 pr-4 pt-4">
			<Text class="m-0 text-base">{report.update}</Text>
		</Section>

		<Text class="text-base">
			You can
			<VLink href="https://modrinth.com/dashboard/report/{report.id}" class="text-green underline"
				>view the full report thread</VLink
			>
			to see all messages. If you have more information to add, reply in the dashboard thread so our
			team can review it in context. For other questions you can contact support through our
			<VLink href="https://support.modrinth.com" class="text-green underline">Support Portal</VLink>
			or by replying to this email.
		</Text>

		<Text class="text-base"
			>Thank you for helping keep Modrinth safe and welcoming for everyone.</Text
		>
	</StyledEmail>
</template>
