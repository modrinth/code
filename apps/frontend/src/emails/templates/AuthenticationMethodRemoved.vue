<script setup lang="ts">
import { Heading, Link as VLink, Text } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail title="Authentication method removed">
		<Heading as="h1" class="mb-2 text-2xl font-bold"> Authentication method removed </Heading>

		<Text class="text-muted text-base">Hi {user.name},</Text>
		<Text class="text-muted text-base">
			When logging into Modrinth, you can no longer log in using the {authprovider.name}
			authentication provider.
		</Text>
		<Text class="text-muted text-base">
			If you did not make this change, please contact us immediately by replying to this email or
			<VLink href="https://support.modrinth.com" class="text-green underline"
				>through our Support Portal</VLink
			>.
		</Text>
	</StyledEmail>
</template>
