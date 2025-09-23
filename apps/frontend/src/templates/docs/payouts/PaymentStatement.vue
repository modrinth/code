<script setup lang="ts">
import { Text, View } from '@ceereals/vue-pdf'
import StyledDoc from '../shared/StyledDoc.vue'

interface Payer {
	name: string
	address: string
	email: string
}

interface Recipient {
	name: string
	address: string
	email: string
}

const props = withDefaults(
	defineProps<{
		payer?: Partial<Payer>
		recipient?: Partial<Recipient>
		payment?: Partial<{
			id: string
			date: string // ISO date string
			currency: string
			gross: number
			fees?: number
			net: number
		}>
	}>(),
	{
		payer: () => ({
			name: 'Rinth, Inc.',
			address: '800 N King St, Suite 304 #3133, Wilmington, DE 19801',
			email: 'support@modrinth.com',
		}),
		recipient: () => ({}),
		payment: () => ({ currency: 'USD' }),
	},
)

function formatMoney(currency: string, amount: number) {
	try {
		return new Intl.NumberFormat('en-US', { style: 'currency', currency }).format(amount)
	} catch {
		return `${currency} ${amount.toFixed(2)}`
	}
}

function displayMoney(currency?: string, amount?: number) {
	if (typeof amount === 'number') return formatMoney(currency ?? 'USD', amount)
	return '—'
}
</script>

<template>
	<StyledDoc title="Payment Statement" v-slot="{ styles }">
		<View :style="styles.section">
			<View :style="styles.row">
				<View :style="[styles.col, styles.colLeft]">
					<Text :style="styles.sectionTitle">Payer</Text>
					<Text :style="styles.text">{{ payer?.name ?? 'Rinth, Inc.' }}</Text>
					<Text :style="styles.text">{{
						payer?.address ?? '800 N King St, Suite 304 #3133, Wilmington, DE 19801'
					}}</Text>
					<Text :style="styles.text">{{ payer?.email ?? 'support@modrinth.com' }}</Text>
				</View>
				<View :style="[styles.col, styles.colRight]">
					<Text :style="styles.sectionTitle">Recipient</Text>
					<Text :style="styles.text">{{ recipient?.name ?? '—' }}</Text>
					<Text :style="styles.text">{{ recipient?.address ?? '—' }}</Text>
					<Text :style="styles.text">{{ recipient?.email ?? '—' }}</Text>
				</View>
			</View>
		</View>

		<View :style="styles.hr" />

		<View :style="styles.section">
			<Text :style="styles.sectionTitle">Payment details</Text>
			<View :style="styles.row">
				<View :style="[styles.col, styles.colLeft]">
					<Text :style="styles.text"
						><Text :style="{ fontWeight: 700 }">Payment ID:</Text> {{ payment?.id ?? '—' }}</Text
					>
					<Text :style="styles.text"
						><Text :style="{ fontWeight: 700 }">Payment date:</Text>
						{{ payment?.date ?? '—' }}</Text
					>
					<Text :style="styles.text"
						><Text :style="{ fontWeight: 700 }">Currency:</Text>
						{{ payment?.currency ?? 'USD' }}</Text
					>
				</View>
				<View :style="[styles.col, styles.colRight]">
					<Text :style="styles.text"
						><Text :style="{ fontWeight: 700 }">Gross amount:</Text>
						{{ displayMoney(payment?.currency, payment?.gross) }}</Text
					>
					<Text v-if="payment?.fees !== undefined" :style="styles.text"
						><Text :style="{ fontWeight: 700 }">Fees:</Text>
						{{ displayMoney(payment?.currency, payment?.fees) }}</Text
					>
					<Text :style="styles.text"
						><Text :style="{ fontWeight: 700 }">Net amount:</Text>
						{{ displayMoney(payment?.currency, payment?.net) }}</Text
					>
				</View>
			</View>
		</View>

		<View :style="styles.hr" />

		<View :style="styles.section">
			<Text :style="styles.sectionTitle">Purpose of payment</Text>
			<Text :style="styles.text">
				This payout reflects revenue earned by the creator through their activity on the Modrinth
				platform. Earnings are based on advertising revenue, subscriptions, and/or affiliate
				commissions tied to the creator’s published projects, in accordance with the Rewards Program
				Terms.
			</Text>
		</View>

		<View :style="styles.hr" />

		<View>
			<Text :style="styles.textMuted">
				This statement records a payout issued by Rinth, Inc. to the creator and may be retained as
				proof of payment.
			</Text>
		</View>
	</StyledDoc>
</template>
