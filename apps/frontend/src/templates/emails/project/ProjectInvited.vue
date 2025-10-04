<script setup lang="ts">
import { Button, Heading, Img, Link as VLink, Section, Text } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail
		title="You've been invited to a project"
		:manual-links="[
			{ link: 'https://modrinth.com/project/{project.id}', label: 'Project page' },
			{ link: 'https://modrinth.com/dashboard/notifications', label: 'Notification dashboard' },
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
		<Heading as="h1" class="mb-2 text-2xl font-bold">You've been invited to a project</Heading>

		<Text class="text-base">Hi {user.name},</Text>

		<Text class="text-base">
			Modrinth user
			<b
				><VLink href="https://modrinth.com/user/{inviter.name}" class="text-green underline">
					{inviter.name}
				</VLink></b
			>
			has invited you to join the project
			<b>
				<VLink href="https://modrinth.com/project/{project.id}" class="text-green underline">
					{project.name}
				</VLink>
			</b>
			on Modrinth.
		</Text>

		<Section class="bg-bg-super mb-4 mt-4 rounded-lg border border-divider pb-4 pl-4 pr-4 pt-4">
			<Text class="m-0 text-base">
				As a project member you may be able to manage versions, edit project details, configure
				settings, and collaborate with other maintainers depending on the permissions assigned to
				you.
			</Text>
		</Section>

		<Button
			href="https://modrinth.com/dashboard/notifications"
			target="_blank"
			class="text-accentContrast inline-block rounded-[12px] bg-brand pb-3 pl-4 pr-4 pt-3 text-[14px] font-bold"
		>
			Review invitation
		</Button>

		<Text class="text-base">
			If you were not expecting this invitation you should reject the invitation. If you believe
			this was sent in error or is abusive, please contact support
			<VLink href="https://support.modrinth.com" class="text-green underline">
				through the Support Portal</VLink
			>
			or by replying to this email.
		</Text>
	</StyledEmail>
</template>
