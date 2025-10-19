<script setup lang="ts">
import { Link as VLink, Section, Text } from '@vue-email/components'

import StyledDoc from '../shared/StyledDoc.vue'
</script>

<template>
	<StyledDoc title="Payment Statement">
		<Section class="mb-8">
			<div class="flex items-start justify-between">
				<div class="flex-1">
					<Text class="m-0 mb-2 text-2xl text-base font-bold">Payment Statement</Text>
					<Text class="m-0 text-sm text-secondary">Payment ID: { statement.payment_id }</Text>
					<Text class="m-0 text-sm text-secondary">Date: { statement.payment_date }</Text>
				</div>
				<div class="flex-1 text-right">
					<Text class="m-0 text-3xl font-bold text-brand">{ statement.net_amount } </Text>
					<Text class="m-0 mt-1 text-sm text-secondary">Net Amount Paid</Text>
				</div>
			</div>
		</Section>

		<Section class="mb-8">
			<div class="flex items-start gap-8">
				<div class="flex-1">
					<Text class="m-0 mb-3 border-b border-divider pb-2 text-base text-sm font-semibold">
						From
					</Text>
					<Text class="m-0 mb-1 text-base text-sm font-semibold">Rinth, Inc.</Text>
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
					<Text class="m-0 mb-3 border-b border-divider pb-2 text-base text-sm font-semibold">
						To
					</Text>
					<Text class="m-0 mb-1 text-base text-sm font-semibold">
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
			<Text class="m-0 mb-4 border-b border-divider pb-2 text-base text-lg font-semibold">
				Payment Details
			</Text>

			<div class="mb-3 flex items-center justify-between">
				<Text class="m-0 text-sm text-secondary">Gross Amount</Text>
				<Text class="m-0 text-base text-sm">{ statement.gross_amount }</Text>
			</div>

			<div class="mb-3 flex items-center justify-between">
				<Text class="m-0 text-sm text-secondary">Processing Fees</Text>
				<Text class="m-0 text-base text-sm">{ statement.fees }</Text>
			</div>

			<hr class="my-4 border-divider" />

			<div class="mb-6 flex items-center justify-between">
				<Text class="m-0 text-base font-semibold">Net Amount</Text>
				<Text class="m-0 text-base font-semibold">{ statement.net_amount }</Text>
			</div>
		</Section>

		<Section class="mb-8">
			<Text class="m-0 mb-4 border-b border-divider pb-2 text-base text-lg font-semibold">
				Purpose of Payment
			</Text>
			<Text class="m-0 text-sm leading-relaxed text-secondary">
				This payout reflects the creator's earnings from their activity on the Modrinth platform.
				Such earnings are based on advertising revenue derived from user engagement with the
				creator's published projects and/or affiliate commissions in accordance with the Rewards
				Program Terms.
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
