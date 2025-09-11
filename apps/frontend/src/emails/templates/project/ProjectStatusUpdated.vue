<script setup lang="ts">
import { Button, Heading, Img, Link as VLink, Section, Text } from '@vue-email/components'

import StyledEmail from '@/emails/shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail
		title="Project status updated"
		:manual-links="[
			{ link: 'https://modrinth.com/legal/rules', label: 'Community Guidelines' },
			{
				link: 'https://modrinth.com/project/{project.id}/moderation',
				label: 'Your project\'s moderation thread',
			},
			{ link: 'https://support.modrinth.com', label: 'Support Portal' },
		]"
	>
		<Section class="mb-4 mt-2">
			<Img
				src="{project.icon_url}"
				alt="project icon"
				width="64"
				height="64"
				class="block h-auto rounded-lg"
			/>
		</Section>
		<Heading as="h1" class="mb-2 text-2xl font-bold">Project status updated</Heading>

		<Text class="text-base">Hi {user.name},</Text>

		<Text class="text-base">
			Your project <b>{project.name}</b> has been <b>{project.newstatus}</b> by the moderation team.
		</Text>

		<Section class="bg-bg-super mb-4 mt-4 rounded-lg border border-divider pb-4 pl-4 pr-4 pt-4">
			<Text class="m-0 text-base">
				If your project status is <b>approved</b>, it should now appear publicly in search and
				listings. If it is <b>withheld</b> or <b>rejected</b>, please
				<VLink
					href="https://modrinth.com/project/{project.id}/moderation"
					class="text-green underline"
					>check the moderation thread</VLink
				>
				for any requested changes or information on why it was rejected or withheld.
			</Text>
		</Section>

		<Button
			href="https://modrinth.com/project/{project.id}"
			target="_blank"
			class="text-accentContrast inline-block rounded-[12px] bg-brand pb-3 pl-4 pr-4 pt-3 text-[14px] font-bold"
		>
			View project
		</Button>

		<Text class="text-base">
			If you believe this status was applied in error, you can reply in the moderation thread or
			contact support through our
			<VLink href="https://support.modrinth.com" class="text-green underline">Support Portal</VLink>
			or by replying to this email.
		</Text>

		<Text class="text-base">Thank you for publishing on Modrinth!</Text>
	</StyledEmail>
</template>
