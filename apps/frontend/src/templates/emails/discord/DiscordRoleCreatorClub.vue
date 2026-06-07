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

		<Text class="text-base">Hi <span class="no-auto-link">{user.name}</span>,</Text>

		<Text class="text-base">
			Your projects have passed 20,000 total downloads, congratulations!
		</Text>

		<Text class="text-base">
			The Creator Club role in the Modrinth Discord is for creators like you. Link your Discord
			account through Modrinth and we'll grant it automatically.
		</Text>

		<Text class="text-base">
			Creator Club gives you access to dedicated Discord spaces: a feedback channel where you can
			share input on creator features directly with the team, and a creator chat for talking with
			other creators and Modrinth staff.
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
