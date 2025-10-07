<script setup lang="ts">
import { Button, Heading, Link as VLink, Text } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail
		title="Verify your Modrinth email"
		:manual-links="[{ link: '{verifyemail.url}', label: 'Verification link' }]"
	>
		<Heading as="h1" class="mb-2 text-2xl font-bold"> Verify your email </Heading>

		<Text class="text-muted text-base">Hi {user.name},</Text>
		<Text class="text-muted text-base">
			Please visit the link below to verify your email. If the button does not work, you can copy
			the link and paste it into your browser. This link expires in 24 hours.
		</Text>

		<Button
			href="{verifyemail.url}"
			target="_blank"
			class="text-accentContrast inline-block rounded-[12px] bg-brand pb-3 pl-4 pr-4 pt-3 text-[14px] font-bold"
		>
			Verify email
		</Button>

		<VLink href="{verifyemail.url}">
			<Text class="text-muted mt-2 break-words text-xs font-bold">{verifyemail.url}</Text>
		</VLink>
	</StyledEmail>
</template>
