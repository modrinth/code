<script setup lang="ts">
import { Button, Heading, Link as VLink, Text } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail title="Reset your Modrinth password">
		<Heading as="h1" class="mb-2 text-2xl font-bold"> Verify your email </Heading>

		<Text class="text-muted text-base">Hi {user.name},</Text>
		<Text class="text-muted text-base">
			Please visit the link below to reset your password. If you did not request for your password
			to be reset, you can safely ignore this email.
		</Text>

		<Button
			href="{resetpassword.url}"
			target="_blank"
			class="text-accentContrast inline-block rounded-[12px] bg-brand px-4 py-3 text-[14px] font-bold"
		>
			Reset Password
		</Button>

		<VLink href="{resetpassword.url}">
			<Text class="text-muted mt-2 break-words text-xs font-bold">{resetpassword.url}</Text>
		</VLink>
	</StyledEmail>
</template>
