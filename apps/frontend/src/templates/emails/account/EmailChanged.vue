<script setup lang="ts">
import { Heading, Link as VLink, Text } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail
		title="Your email has been changed"
		:manual-links="[{ link: 'https://support.modrinth.com', label: 'Support Portal' }]"
	>
		<Heading as="h1" class="mb-2 text-2xl font-bold"> Your email has been changed </Heading>

		<Text class="text-muted text-base">Hi {user.name},</Text>
		<Text class="text-muted text-base">
			At your request, we've successfully updated your Modrinth account's email to
			{emailchanged.new_email}.
		</Text>
		<Text class="text-muted text-base">
			If you did not make this change, please contact us immediately by replying to this email or
			<VLink href="https://support.modrinth.com" class="text-green underline"
				>through our Support Portal</VLink
			>.
		</Text>
	</StyledEmail>
</template>
