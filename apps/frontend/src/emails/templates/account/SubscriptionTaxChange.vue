<script setup lang="ts">
import { Column, Heading, Link as VLink, Row, Section, Text } from '@vue-email/components'

import StyledEmail from '@/emails/shared/StyledEmail.vue'
</script>

<template>
	<StyledEmail
		title="Price change for {taxnotification.service}"
		:manual-links="[{ link: 'https://support.modrinth.com', label: 'Support Portal' }]"
	>
		<Heading as="h1" class="mb-2 text-2xl font-bold">
			Price change for {taxnotification.service}
		</Heading>

		<Text class="text-muted text-base">Hi {user.name},</Text>
		<Text class="text-muted text-base">
			We're writing to let you know that taxes for your {taxnotification.service} subscription will
			affect your upcoming {taxnotification.billing_interval} billing cycle. This changes your price
			starting on {taxnotification.due}.
		</Text>

		<Section class="bg-bg-super rounded-lg border border-divider pb-4 pl-4 pr-4 pt-4">
			<Row>
				<Column class="w-full">
					<Row class="mb-2 mt-0">
						<Column>
							<Text class="m-0 text-sm font-semibold text-secondary">Old price</Text>
							<Text class="m-0 text-base"><b>{taxnotification.old_total_amount}</b></Text>
							<Text class="m-0 text-xs text-secondary">
								includes {taxnotification.old_amount} + {taxnotification.old_tax_amount} tax
							</Text>
						</Column>
					</Row>

					<Row>
						<Column>
							<Text class="m-0 text-sm font-semibold text-secondary">New price</Text>
							<Text class="m-0 text-base"><b>{taxnotification.new_total_amount}</b></Text>
							<Text class="m-0 text-xs text-secondary">
								includes {taxnotification.new_amount} + {taxnotification.new_tax_amount} tax
							</Text>
						</Column>
					</Row>
				</Column>
			</Row>
		</Section>

		<Text class="text-muted text-base">
			No action is required on your part. This change will automatically take effect on
			{taxnotification.due} with your next {taxnotification.billing_interval} charge.
		</Text>

		<Text class="text-muted text-base">
			If you have questions or need help with your subscription, reply to this email or visit our
			<VLink href="https://support.modrinth.com" class="text-green underline">Support Portal</VLink
			>.
		</Text>
	</StyledEmail>
</template>
