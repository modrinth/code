<script setup lang="ts">
import { Button, Heading, Section, Text } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail
		title="You're invited to the Creator Club"
		:manual-links="[{ link: '{discord.link_url}', label: 'Link Discord' }]"
	>
		<Heading as="h1" class="mb-2 text-2xl font-bold">You're invited to the Creator Club</Heading>

		<Text class="text-base">Hey <span class="no-auto-link">{user.name}</span>!</Text>

		<Text class="text-base">
			Your projects just passed 20,000 total downloads, nice!
		</Text>

		<Text class="text-base">
			We want to invite you to Modrinth's Creator Club, a space in our discord where you can chat
			with other creators, share feedback with us, and stay plugged in.
		</Text>

		<Text class="text-base">
			To join just link your Discord account through Modrinth and we'll grant access automatically!
		</Text>

		<Section class="mb-4 mt-4">
			<Button
				href="{discord.link_url}"
				target="_blank"
				class="text-accentContrast inline-block rounded-[12px] bg-brand pb-3 pl-4 pr-4 pt-3 text-[14px] font-bold"
			>
				Join the Creator Club
			</Button>
		</Section>
	</StyledEmail>
</template>
