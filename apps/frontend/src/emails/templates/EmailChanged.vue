<script setup lang="ts">
import { Heading, Text, Link as VLink } from '@vue-email/components'
import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail title="Your Modrinth email was changed">
		<Heading as="h1" class="mb-2 text-2xl font-bold"> Your Modrinth email was changed </Heading>

		<Text class="text-muted text-base">Hi {user.name},</Text>
		<Text class="text-muted text-base">
			Your Modrinth account email has been updated to {emailchanged.new_email}.
		</Text>
		<Text class="text-muted text-base">
			If you did not make this change, please contact us immediately by replying to this email or
			<VLink href="https://support.modrinth.com" class="text-green underline"
				>through our Support Portal</VLink
			>.
		</Text>
	</StyledEmail>
</template>
