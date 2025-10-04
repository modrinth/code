<script setup lang="ts">
import { Heading, Img, Link as VLink, Section, Text } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
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
		<Heading as="h1" class="mb-2 text-2xl font-bold"
			>New message from moderators on {project.name}</Heading
		>

		<Text class="text-base">Hi {user.name},</Text>

		<Text class="text-base">
			Modrinth's moderation team has left a message on your project,
			<VLink
				href="https://modrinth.com/project/{project.id}/moderation"
				class="text-green underline"
				>{project.name}</VLink
			>.
		</Text>

		<Section class="bg-bg-super mb-4 mt-4 rounded-lg border border-divider pb-4 pl-4 pr-4 pt-4">
			<Text class="m-0 text-base">
				Please
				<VLink
					href="https://modrinth.com/project/{project.id}/moderation"
					class="text-green underline"
					>sign in to view the message</VLink
				>
				and reply if requested. It's important to address feedback from the moderation team
				promptly.
			</Text>
		</Section>

		<Text class="text-base">Thank you for publishing on Modrinth!</Text>
	</StyledEmail>
</template>
