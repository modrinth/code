<script setup lang="ts">
import { Heading, Text, Link as VLink } from '@vue-email/components'

import StyledEmail from '@/emails/shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail
		title="A new PAT has been added to your account"
		:manual-links="[{ link: 'https://support.modrinth.com', label: 'Support Portal' }]"
	>
		<Heading as="h1" class="mb-2 text-2xl font-bold">
			A new PAT has been added to your account
		</Heading>

		<Text class="text-muted text-base">Hi {user.name},</Text>
		<Text class="text-muted text-base">
			A new personal access token, <b>{newpat.token_name}</b>, has been created for your account.
		</Text>
		<Text class="text-muted text-base">
			If you did not create this token, please contact us immediately by replying to this email or
			<VLink href="https://support.modrinth.com" class="text-green underline"
				>through our Support Portal</VLink
			>.
		</Text>
	</StyledEmail>
</template>
