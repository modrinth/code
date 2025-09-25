<script setup lang="ts">
import { Heading, Link as VLink, Text } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail
		title="Your password has been removed"
		:manual-links="[{ link: 'https://support.modrinth.com', label: 'Support Portal' }]"
	>
		<Heading as="h1" class="mb-2 text-2xl font-bold"> Your password has been removed </Heading>

		<Text class="text-muted text-base">Hi {user.name},</Text>
		<Text class="text-muted text-base">
			At your request, your password has been removed from your account. You must now use a linked
			authentication provider (such as your {passremoved.provider} account) to log into your
			Modrinth account.</Text
		>
		<Text class="text-muted text-base">
			If you did not make this change, please contact us immediately by replying to this email or
			<VLink href="https://support.modrinth.com" class="text-green underline"
				>through our Support Portal</VLink
			>.
		</Text>
	</StyledEmail>
</template>
