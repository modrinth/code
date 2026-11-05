<script setup lang="ts">
import { Heading, Link as VLink, Section, Text } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail
		title="Your shared instance has been locked"
		:manual-links="[
			{ link: 'https://modrinth.com/legal/terms', label: 'Terms of Service' },
			{ link: 'https://support.modrinth.com', label: 'Support Portal' },
		]"
	>
		<Heading as="h1" class="mb-2 text-2xl font-bold">
			Your shared instance has been locked
		</Heading>

		<Text class="text-base">Hi <span class="no-auto-link">{user.name}</span>,</Text>

		<Text class="text-base">
			We're emailing you to let you know that your shared instance, "<b>{sharedinstance.name}</b>",
			has been locked by the Modrinth moderation team as it was found to break our
			<VLink href="https://modrinth.com/legal/terms" class="text-green underline"
				>Terms of Service</VLink
			>.
		</Text>

		<Text class="text-base">
			While it remains locked, the instance cannot be played or modified, and new members cannot be
			invited to it.
		</Text>

		<Section
			class="bg-bg-super mb-4 mt-4 rounded-lg border border-divider pb-4 pl-4 pr-4 pt-4"
			style="display: {sharedinstance.reason_display}"
		>
			<Text class="m-0 text-base font-bold">Reason provided:</Text>
			<Text class="mb-0 mt-2 whitespace-pre-wrap text-base">{sharedinstance.reason}</Text>
		</Section>

		<Text class="text-base">
			If you believe this action was taken in error, please contact
			<VLink href="https://support.modrinth.com" class="text-green underline"
				>Modrinth Support</VLink
			>
			and reference the name of the affected instance.
		</Text>
	</StyledEmail>
</template>
