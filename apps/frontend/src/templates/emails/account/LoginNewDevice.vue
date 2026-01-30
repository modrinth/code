<script setup lang="ts">
import { Column, Heading, Img, Link as VLink, Row, Section, Text } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail
		title="Sign in from new device"
		:manual-links="[{ link: 'https://support.modrinth.com', label: 'Support Portal' }]"
	>
		<Heading as="h1" class="mb-2 text-2xl font-bold"> Sign in from new device </Heading>

		<Text class="text-muted text-base">Hi {user.name},</Text>
		<Text class="text-muted text-base">
			We noticed that your account was just signed into from a new device or location. If this was
			you, you can safely ignore this email.
		</Text>

		<Section class="bg-bg-super rounded-lg border border-divider pb-4 pl-4 pr-4 pt-4">
			<Row>
				<Column class="w-full">
					<Row class="mb-2 mt-0">
						<Column class="w-8">
							<Img
								width="20"
								height="20"
								alt="Time icon"
								src="https://cdn-raw.modrinth.com/email/earth.png"
							/>
						</Column>
						<Column>
							<Text class="m-0 text-sm font-semibold text-secondary">Approximate location</Text>
							<Text class="m-0 text-base">{newdevice.location}</Text>
						</Column>
					</Row>

					<Row class="mb-2">
						<Column class="w-8">
							<Img
								width="20"
								height="20"
								alt="Time icon"
								src="https://cdn-raw.modrinth.com/email/monitor-smartphone.png"
							/>
						</Column>
						<Column>
							<Text class="m-0 text-sm font-semibold text-secondary">IP Address</Text>
							<Text class="m-0 font-mono text-base">{newdevice.ip}</Text>
						</Column>
					</Row>

					<Row>
						<Column class="w-8">
							<Img
								width="20"
								height="20"
								alt="Time icon"
								src="https://cdn-raw.modrinth.com/email/clock.png"
							/>
						</Column>
						<Column>
							<Text class="m-0 text-sm font-semibold text-secondary">Time</Text>
							<Text class="m-0 text-base">{time.now}</Text>
						</Column>
					</Row>
				</Column>
			</Row>
		</Section>

		<Text class="text-muted text-base">
			If this wasn't you, please update your password and review your account security settings. If
			you cannot do this, contact us immediately through our
			<VLink href="https://support.modrinth.com" class="text-green underline">Support Portal</VLink
			>.
		</Text>
	</StyledEmail>
</template>
