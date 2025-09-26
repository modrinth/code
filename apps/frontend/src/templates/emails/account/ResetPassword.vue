<script setup lang="ts">
import { Button, Heading, Link as VLink, Text } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail
		title="Reset your password"
		:manual-links="[{ link: '{resetpassword.url}', label: 'Password reset link' }]"
	>
		<Heading as="h1" class="mb-2 text-2xl font-bold"> Reset your password </Heading>

		<Text class="text-muted text-base">Hi {user.name},</Text>
		<Text class="text-muted text-base">
			Please visit the link below to reset your password. If you did not request for your password
			to be reset, you can safely ignore this email.
		</Text>

		<Button
			href="{resetpassword.url}"
			target="_blank"
			class="text-accentContrast inline-block rounded-[12px] bg-brand pb-3 pl-4 pr-4 pt-3 text-[14px] font-bold"
		>
			Reset password
		</Button>

		<VLink href="{resetpassword.url}">
			<Text class="text-muted mt-2 break-words text-xs font-bold">{resetpassword.url}</Text>
		</VLink>
	</StyledEmail>
</template>
