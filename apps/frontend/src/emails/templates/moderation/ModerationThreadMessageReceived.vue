<script setup lang="ts">
import StyledEmail from '@/emails/shared/StyledEmail.vue'
import { Heading, Img, Section, Text, Link as VLink } from '@vue-email/components'
</script>

<template>
	<StyledEmail
		title="New moderation message"
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
				class="block h-auto"
			/>
		</Section>
		<Heading as="h1" class="mb-2 text-2xl font-bold">New moderation message</Heading>

		<Text class="text-base">Hi {user.name},</Text>

		<Text class="text-base">
			A new message has been posted in the
			<VLink
				href="https://modrinth.com/project/{project.id}/moderation"
				class="text-green underline"
				>moderation thread</VLink
			>
			for your project <b>{project.name}</b>.
		</Text>

		<Section class="bg-bg-super mb-4 mt-4 rounded-lg border border-divider pb-4 pl-4 pr-4 pt-4">
			<Text class="m-0 text-base">
				Log in to review the message and reply if needed. Addressing moderator feedback promptly
				helps move your project forward.
			</Text>
		</Section>

		<Text class="text-base">
			If you have questions after reviewing the
			<VLink
				href="https://modrinth.com/project/{project.id}/moderation"
				class="text-green underline"
				>moderation thread</VLink
			>, you can reply there for clarification or
			<VLink href="https://support.modrinth.com" class="text-green underline"
				>contact our support team</VLink
			>.
		</Text>

		<Text class="text-base">Thank you for publishing on Modrinth!</Text>
	</StyledEmail>
</template>
