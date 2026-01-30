<script setup lang="ts">
import { Button, Heading, Img, Link as VLink, Section, Text } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail
		title="You've been invited to an organization"
		:manual-links="[
			{ link: 'https://modrinth.com/organization/{organization.id}', label: 'Organization page' },
			{ link: 'https://modrinth.com/dashboard/notifications', label: 'Notification dashboard' },
			{ link: 'https://support.modrinth.com', label: 'Support Portal' },
		]"
	>
		<Section class="mb-4 mt-2">
			<Img
				src="{organization.icon_url}"
				alt="organization icon"
				width="64"
				height="64"
				class="block h-auto rounded-lg"
			/>
		</Section>

		<Heading as="h1" class="mb-2 text-2xl font-bold"
			>You've been invited to an organization</Heading
		>

		<Text class="text-base">Hi {user.name},</Text>

		<Text class="text-base"
			>Modrinth user
			<b
				><VLink href="https://modrinth.com/user/{inviter.name}" class="text-green underline">
					{inviter.name}
				</VLink></b
			>
			has invited you to join the organization
			<b
				><VLink
					href="https://modrinth.com/organization/{organization.id}"
					class="text-green underline"
				>
					{organization.name}
				</VLink></b
			>
			on Modrinth.
		</Text>

		<Section class="bg-bg-super mb-4 mt-4 rounded-lg border border-divider pb-4 pl-4 pr-4 pt-4">
			<Text class="m-0 text-base">
				As an organization member you may gain access to manage projects, teams, members, settings,
				and other resources depending on the permissions assigned to you.
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
			If you were not expecting this invitation you should reject it. If you believe this was sent
			in error or is abusive, please contact support
			<VLink href="https://support.modrinth.com" class="text-green underline">
				through the Support Portal</VLink
			>.
		</Text>
	</StyledEmail>
</template>
