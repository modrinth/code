<script setup lang="ts">
import { Button, Heading, Img, Link as VLink, Section, Text } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail
		title="Your project, {project.name}, has been approved 🎉"
		:manual-links="[
			{ link: 'https://modrinth.com/project/{project.id}', label: 'Project page' },
			{ link: 'https://modrinth.com/legal/rules', label: 'Community Guidelines' },
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
		<Heading as="h1" class="mb-2 text-2xl font-bold"
			>Your project, {project.name}, has been approved 🎉</Heading
		>

		<Text class="text-base">Congratulations {user.name},</Text>

		<Text class="text-base">
			Your project
			<VLink href="https://modrinth.com/project/{project.id}" class="text-green underline"
				>{project.name}</VLink
			>
			has been <b>approved</b> by the moderation team!
		</Text>

		<Button
			href="https://modrinth.com/project/{project.id}"
			target="_blank"
			class="text-accentContrast inline-block rounded-[12px] bg-brand pb-3 pl-4 pr-4 pt-3 text-[14px] font-bold"
		>
			View project
		</Button>

		<Text class="text-base">
			If you have questions or believe something isn't correct, you can reach out via the
			<VLink href="https://support.modrinth.com" class="text-green underline">Support Portal</VLink
			>.
		</Text>

		<Text class="text-base">Thank you for sharing your work with the Modrinth community!</Text>
	</StyledEmail>
</template>
