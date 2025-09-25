<script setup lang="ts">
import { Column, Row, Section, Text, Link as VLink } from '@vue-email/components'
import StyledDoc from '../shared/StyledDoc.vue'
</script>

<template>
	<StyledDoc title="Payment Statement">
		<Section class="mb-8">
			<div class="flex justify-between items-start">
				<div class="flex-1">
					<Text class="m-0 mb-2 text-2xl font-bold text-base">Payment Statement</Text>
					<Text class="m-0 text-sm text-secondary">Payment ID: { statement.payment_id }</Text>
					<Text class="m-0 text-sm text-secondary">Date: { statement.payment_date }</Text>
				</div>
				<div class="flex-1 text-right">
					<Text class="m-0 text-3xl font-bold text-brand">USD$ 223.40</Text>
					<Text class="m-0 mt-1 text-sm text-secondary">Net Amount Paid</Text>
				</div>
			</div>
		</Section>

		<Section class="mb-8">
			<div class="flex gap-8 items-start">
				<div class="flex-1">
					<Text class="m-0 mb-3 border-b border-divider pb-2 text-sm font-semibold text-base">
						From
					</Text>
					<Text class="m-0 mb-1 text-sm font-semibold text-base">Rinth, Inc.</Text>
					<Text class="m-0 text-sm leading-tight text-secondary">800 N King St</Text>
					<Text class="m-0 text-sm leading-tight text-secondary">Suite 304 #3133</Text>
					<Text class="m-0 text-sm leading-tight text-secondary">Wilmington, DE 19801</Text>
					<Text class="m-0 mt-2 text-sm text-secondary">
						<VLink href="mailto:support@modrinth.com" class="text-blue no-underline">
							support@modrinth.com
						</VLink>
					</Text>
				</div>

				<div class="flex-1">
					<Text class="m-0 mb-3 border-b border-divider pb-2 text-sm font-semibold text-base">
						To
					</Text>
					<Text class="m-0 mb-1 text-sm font-semibold text-base">
						{ statement.recipient_name }
					</Text>
					<Text class="m-0 text-sm leading-tight text-secondary">
						{ statement.recipient_address_line_1 }
					</Text>
					<Text class="m-0 text-sm leading-tight text-secondary">
						{ statement.recipient_address_line_2 }
					</Text>
					<Text class="m-0 text-sm leading-tight text-secondary">
						{ statement.recipient_address_line_3 }
					</Text>
					<Text class="m-0 mt-2 text-sm text-secondary">
						<VLink href="mailto:{ statement.recipient_email }" class="text-blue no-underline">
							{ statement.recipient_email }
						</VLink>
					</Text>
				</div>
			</div>
		</Section>

		<Section class="mb-8">
			<Text class="m-0 mb-4 border-b border-divider pb-2 text-lg font-semibold text-base">
				Payment Details
			</Text>

			<div class="flex justify-between items-center mb-3">
				<Text class="m-0 text-sm text-secondary">Gross Amount</Text>
				<Text class="m-0 text-sm text-base">{ statement.gross_amount }</Text>
			</div>

			<div class="flex justify-between items-center mb-3">
				<Text class="m-0 text-sm text-secondary">Processing Fees</Text>
				<Text class="m-0 text-sm text-base">{ statement.fees }</Text>
			</div>

			<hr class="my-4 border-divider" />

			<div class="flex justify-between items-center mb-6">
				<Text class="m-0 text-base font-semibold text-base">Net Amount</Text>
				<Text class="m-0 text-base font-semibold text-base">{ statement.net_amount }</Text>
			</div>
		</Section>

		<Section class="mb-8">
			<Text class="m-0 mb-4 border-b border-divider pb-2 text-lg font-semibold text-base">
				Purpose of Payment
			</Text>
			<Text class="m-0 text-sm leading-relaxed text-secondary">
				This payout reflects revenue earned by the creator through their activity on the Modrinth
				platform. Earnings are based on advertising revenue, subscriptions, and/or affiliate
				commissions tied to the creator's published projects, in accordance with the Rewards Program
				Terms.
			</Text>
		</Section>

		<Section class="mt-12 border-t border-divider pt-6">
			<Text class="text-muted m-0 text-center text-xs">
				This statement records a payout issued by Rinth, Inc. to the creator and may be retained as
				proof of payment.
			</Text>
		</Section>
	</StyledDoc>
</template>
