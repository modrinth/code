<script setup lang="ts">
import { Button, Column, Heading, Row, Section, Text, Link as VLink } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail
		title="You've been invited to a server"
		:manual-links="[
			{ link: 'https://modrinth.com/dashboard/notifications', label: 'Notification dashboard' },
			{ link: 'https://support.modrinth.com', label: 'Support Portal' },
		]"
	>
		<Heading as="h1" class="mb-2 text-2xl font-bold">You've been invited to a server</Heading>

		<Text class="text-base">Hi <span class="no-auto-link">{user.name}</span>,</Text>

		<Text class="text-base">
			Modrinth user
			<b
				><VLink href="https://modrinth.com/user/{inviter.name}" class="text-green underline">
					{inviter.name}
				</VLink></b
			>
			has invited you to help manage
			<b>{server.name}</b>
			on Modrinth Hosting.
		</Text>

		<Section class="bg-bg-super mb-4 mt-4 rounded-lg border border-divider pb-4 pl-4 pr-4 pt-4">
			<Text class="m-0 text-base">
				You have been invited with the <b>{server.role}</b> role permission.
			</Text>
		</Section>

		<Button
			href="https://modrinth.com/dashboard/notifications"
			target="_blank"
			class="text-accentContrast inline-block rounded-[12px] bg-brand pb-3 pl-4 pr-4 pt-3 text-[14px] font-bold"
		>
			Review invitation
		</Button>

		<VLink href="https://modrinth.com/dashboard/notifications">
			<Text class="text-muted mt-2 break-words text-xs font-bold"
				>https://modrinth.com/dashboard/notifications</Text
			>
		</VLink>

		<Text class="text-base">
			To accept or reject this invitation, open your Modrinth notifications and review the invite.
			If you were not expecting this invitation, contact the server owner or reach out to Modrinth
			Support
			<VLink href="https://support.modrinth.com" class="text-green underline">
				through the Support Portal</VLink
			>.
		</Text>

		<Section class="border-0 border-t border-solid border-divider pt-4">
			<Text class="mb-3 mt-0 text-base font-bold">What does my role mean?</Text>
			<Row>
				<Column class="pr-2 align-top">
					<Section class="bg-bg-super rounded-lg border border-divider pb-3 pl-3 pr-3 pt-3">
						<Text class="m-0 text-xs font-bold uppercase">Editor</Text>
						<Text class="mb-0 mt-2 text-base">
							Manage instance content, files, backups, and other settings.
						</Text>
					</Section>
				</Column>
				<Column class="pl-2 align-top">
					<Section class="bg-bg-super rounded-lg border border-divider pb-3 pl-3 pr-3 pt-3">
						<Text class="m-0 text-xs font-bold uppercase">Viewer</Text>
						<Text class="mb-0 mt-2 text-base">
							Start, stop, and view the server without making changes.
						</Text>
					</Section>
				</Column>
			</Row>
		</Section>
	</StyledEmail>
</template>
