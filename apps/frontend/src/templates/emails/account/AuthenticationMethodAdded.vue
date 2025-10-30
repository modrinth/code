<script setup lang="ts">
import { Heading, Link as VLink, Text } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail
		title="New sign-in method added"
		:manual-links="[{ link: 'https://support.modrinth.com', label: 'Support Portal' }]"
	>
		<Heading as="h1" class="mb-2 text-2xl font-bold"> New sign-in method added </Heading>

		<Text class="text-muted text-base">Hi {user.name},</Text>
		<Text class="text-muted text-base">
			Your {authprovider.name} account has been connected and you can now use it to sign in to your
			Modrinth account.
		</Text>
		<Text class="text-muted text-base">
			If you did not make this change, please contact us immediately through our
			<VLink href="https://support.modrinth.com" class="text-green underline">Support Portal</VLink
			>.
		</Text>
	</StyledEmail>
</template>
