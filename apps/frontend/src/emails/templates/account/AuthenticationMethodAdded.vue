<script setup lang="ts">
import { Heading, Text, Link as VLink } from '@vue-email/components'

import StyledEmail from '@/emails/shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail
		title="Authentication method added"
		:manual-links="[{ link: 'https://support.modrinth.com', label: 'Support Portal' }]"
	>
		<Heading as="h1" class="mb-2 text-2xl font-bold"> Authentication method added </Heading>

		<Text class="text-muted text-base">Hi {user.name},</Text>
		<Text class="text-muted text-base">
			When logging into Modrinth, you can now log in using the {authprovider.name} authentication
			provider.
		</Text>
		<Text class="text-muted text-base">
			If you did not make this change, please contact us immediately by replying to this email or
			<VLink href="https://support.modrinth.com" class="text-green underline"
				>through our Support Portal</VLink
			>.
		</Text>
	</StyledEmail>
</template>
