<script setup lang="ts">
import BaseEmail from '../shared/BaseEmail.vue'
</script>

<template>
	<BaseEmail
		link="https://modrinth.com/auth/verify-email?flow=E33LrUXHB..."
		buttonText="Verify email"
	>
		<template #title>Verify your email</template>

		<template #subtitle> We need to verify your email address. </template>

		Please visit the following link below to verify your email. If the button does not work, copy
		the link and paste it into your browser. This link expires in 24 hours.
	</BaseEmail>
</template>
