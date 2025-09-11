<script setup lang="ts">
import { Heading, Text, Link as VLink } from '@vue-email/components'

import StyledEmail from '@/emails/shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail
		title="Your Modrinth password was removed"
		:manual-links="[{ link: 'https://support.modrinth.com', label: 'Support Portal' }]"
	>
		<Heading as="h1" class="mb-2 text-2xl font-bold"> Your Modrinth password was removed </Heading>

		<Text class="text-muted text-base">Hi {user.name},</Text>
		<Text class="text-muted text-base">
			Your password has been removed from your account. You must now use an authentication provider
			to log into your Modrinth account.</Text
		>
		<Text class="text-muted text-base">
			If you did not make this change, please contact us immediately by replying to this email or
			<VLink href="https://support.modrinth.com" class="text-green underline"
				>through our Support Portal</VLink
			>.
		</Text>
	</StyledEmail>
</template>
