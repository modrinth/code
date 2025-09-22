<script setup lang="ts">
import { Body, Head, Html, Img, Style } from '@vue-email/components'
import { computed } from 'vue'

type PayoutMethod = 'paypal' | 'venmo' | 'ach' | 'gift_card' | 'other'
type PayoutStatus = 'pending' | 'paid' | 'failed'

interface CompanyInfo {
	name: string
	addressLines?: string[]
	logoUrl?: string
	website?: string
	supportEmail?: string
}

interface RecipientInfo {
	name: string
	email?: string
	addressLines?: string[]
	id?: string
}

interface StatementMeta {
	id: string
	periodStart?: string
	periodEnd?: string
	periodLabel?: string
	issuedAt?: string
	currency?: string
	notes?: string[]
}

interface LineItem {
	label: string
	amount: number
	note?: string
	group?: string
	type?: 'earning' | 'deduction'
}

interface SimpleItem {
	label: string
	amount: number
	note?: string
}

interface PayoutInfo {
	method: PayoutMethod
	destination?: string
	reference?: string
	date?: string
	status?: PayoutStatus
	fees?: number
	amount?: number
}

const props = defineProps<{
	title?: string
	company: CompanyInfo
	recipient: RecipientInfo
	statement: StatementMeta
	payout?: PayoutInfo
	breakdown: LineItem[]
	taxes?: SimpleItem[]
	adjustments?: SimpleItem[]
	otherDeductions?: SimpleItem[]
	previousBalance?: number
	locale?: string
	brand?: { primary?: string; accentContrast?: string }
}>()

const locale = props.locale ?? 'en-US'
const currency = props.statement.currency ?? 'USD'

function fmtCurrency(amount: number | undefined | null) {
	const a = amount ?? 0
	try {
		return new Intl.NumberFormat(locale, { style: 'currency', currency }).format(a)
	} catch {
		return `${a.toFixed(2)} ${currency}`
	}
}

function fmtDate(iso?: string) {
	if (!iso) return ''
	try {
		return new Date(iso).toLocaleDateString(locale, {
			year: 'numeric',
			month: 'short',
			day: '2-digit',
		})
	} catch {
		return iso
	}
}

const earningsTotal = computed(() =>
	(props.breakdown || [])
		.filter((x) => x.type !== 'deduction')
		.reduce((sum, x) => sum + (Number.isFinite(x.amount) ? x.amount : 0), 0),
)

const deductionsTotal = computed(() => {
	const core = (props.breakdown || [])
		.filter((x) => x.type === 'deduction')
		.reduce((sum, x) => sum + (Number.isFinite(x.amount) ? x.amount : 0), 0)
	const taxes = (props.taxes || []).reduce(
		(sum, x) => sum + (Number.isFinite(x.amount) ? x.amount : 0),
		0,
	)
	const other = (props.otherDeductions || []).reduce(
		(sum, x) => sum + (Number.isFinite(x.amount) ? x.amount : 0),
		0,
	)
	const fees = props.payout?.fees ?? 0
	return core + taxes + other + fees
})

const adjustmentsTotal = computed(() =>
	(props.adjustments || []).reduce((sum, x) => sum + (Number.isFinite(x.amount) ? x.amount : 0), 0),
)

const startingBalance = computed(() => props.previousBalance ?? 0)
const netBeforePayout = computed(
	() =>
		startingBalance.value + earningsTotal.value + adjustmentsTotal.value - deductionsTotal.value,
)
const payoutAmount = computed(() => props.payout?.amount ?? 0)
const endingBalance = computed(() => netBeforePayout.value - payoutAmount.value)

function payoutMethodLabel(method?: PayoutMethod) {
	switch (method) {
		case 'paypal':
			return 'PayPal'
		case 'venmo':
			return 'Venmo'
		case 'ach':
			return 'ACH Transfer'
		case 'gift_card':
			return 'Gift card'
		case 'other':
			return 'Other'
		default:
			return '—'
	}
}

const brandPrimary = props.brand?.primary ?? '#00af5c'
const brandContrast = props.brand?.accentContrast ?? '#ffffff'
</script>

<template>
	<Html lang="en">
		<Head>
			<title>{{ title || 'Payment Statement' }}</title>
			<meta http-equiv="X-UA-Compatible" content="IE=edge" />
			<meta http-equiv="Content-Type" content="text/html; charset=UTF-8" />
			<meta name="viewport" content="width=device-width, initial-scale=1" />
			<link
				href="https://fonts.googleapis.com/css?family=Inter:700,400"
				rel="stylesheet"
				type="text/css"
			/>
			<Style>
				body { margin: 0; padding: 0; background: #f7f7f7; color: #2c2e31; font-family: Inter,
				Arial, sans-serif; } .container { width: 100%; max-width: 800px; margin: 0 auto; padding:
				24px; } .card { background: #ffffff; border: 1px solid #e5e7eb; border-radius: 8px; } .p-16
				{ padding: 16px; } .p-24 { padding: 24px; } .mt-8 { margin-top: 8px; } .mt-12 { margin-top:
				12px; } .mt-16 { margin-top: 16px; } .mt-24 { margin-top: 24px; } .muted { color: #6b7280;
				font-size: 12px; text-transform: uppercase; margin: 0; } .small { font-size: 12px; } .title
				{ font-weight: 700; font-size: 20px; margin: 0; } .subtitle { font-weight: 700; font-size:
				16px; margin: 0; } .text { margin: 0; font-size: 14px; } .right { text-align: right; }
				.header { display: table; width: 100%; } .header-col { display: table-cell; vertical-align:
				middle; } .logo { display: inline-block; vertical-align: middle; } .spacer { height: 16px; }
				.table { width: 100%; border-collapse: collapse; } .table th, .table td { padding: 8px;
				border-top: 1px solid #e5e7eb; vertical-align: top; } .table thead th { background: #f3f4f6;
				border-top: none; font-size: 12px; text-transform: uppercase; color: #6b7280; text-align:
				left; } .summary-head { background: {{ brandPrimary }}; color: {{ brandContrast }}; padding:
				12px 16px; font-weight: 700; font-size: 14px; } .row { display: table; width: 100%;
				table-layout: fixed; } .col-6 { display: table-cell; width: 50%; vertical-align: top; }
				.divider { height: 1px; background: #e5e7eb; border: 0; margin: 24px 0; } .list { margin:
				8px 0 0; padding-left: 18px; }
			</Style>
		</Head>

		<Body>
			<div class="container">
				<!-- Header -->
				<div class="card p-24">
					<div class="header">
						<div class="header-col" style="width: 60%">
							<div>
								<span v-if="company.logoUrl" class="logo" style="margin-right: 10px">
									<Img
										:src="company.logoUrl"
										width="36"
										alt=""
										style="display: block; height: auto"
									/>
								</span>
								<span style="display: inline-block; vertical-align: middle">
									<p class="title">{{ company.name }}</p>
									<p v-if="company.website" class="text muted" style="text-transform: none">
										{{ company.website }}
									</p>
								</span>
							</div>
						</div>
						<div class="header-col right" style="width: 40%">
							<p class="title">Payment Statement</p>
							<p class="text muted" style="text-transform: none">
								Statement ID: {{ statement.id }}
							</p>
						</div>
					</div>

					<!-- Meta band -->
					<div class="card mt-16 p-16" style="border-radius: 6px">
						<div class="row">
							<div class="col-6">
								<p class="muted">Period</p>
								<p class="text">
									<template v-if="statement.periodLabel">{{ statement.periodLabel }}</template>
									<template v-else
										>{{ fmtDate(statement.periodStart) }} –
										{{ fmtDate(statement.periodEnd) }}</template
									>
								</p>
							</div>
							<div class="col-6">
								<p class="muted">Issued</p>
								<p class="text">{{ fmtDate(statement.issuedAt) }}</p>
							</div>
						</div>
						<div class="mt-12">
							<p class="muted">Currency</p>
							<p class="text">{{ statement.currency || 'USD' }}</p>
						</div>
					</div>

					<!-- Parties -->
					<div class="row mt-16">
						<div class="col-6">
							<div class="card p-16" style="border-radius: 6px; margin-right: 8px">
								<p class="muted">From</p>
								<p class="subtitle" style="margin-top: 6px">{{ company.name }}</p>
								<p v-for="(line, idx) in company.addressLines" :key="'c-' + idx" class="text">
									{{ line }}
								</p>
								<p v-if="company.supportEmail" class="text">{{ company.supportEmail }}</p>
							</div>
						</div>
						<div class="col-6">
							<div class="card p-16" style="border-radius: 6px; margin-left: 8px">
								<p class="muted">To</p>
								<p class="subtitle" style="margin-top: 6px">{{ recipient.name }}</p>
								<p v-for="(line, idx) in recipient.addressLines" :key="'r-' + idx" class="text">
									{{ line }}
								</p>
								<p v-if="recipient.email" class="text">{{ recipient.email }}</p>
								<p v-if="recipient.id" class="text muted" style="text-transform: none">
									ID: {{ recipient.id }}
								</p>
							</div>
						</div>
					</div>

					<!-- Summary totals -->
					<div class="card mt-16" style="overflow: hidden; border-radius: 6px">
						<div class="summary-head">Summary</div>
						<div class="p-16">
							<table class="table">
								<tbody>
									<tr>
										<td style="width: 60%; color: #484d54">Previous balance</td>
										<td class="right" style="width: 40%; font-weight: 600">
											{{ fmtCurrency(startingBalance) }}
										</td>
									</tr>
									<tr>
										<td style="color: #484d54">Earnings</td>
										<td class="right" style="font-weight: 600">{{ fmtCurrency(earningsTotal) }}</td>
									</tr>
									<tr v-if="adjustments && adjustments.length">
										<td style="color: #484d54">Adjustments</td>
										<td class="right" style="font-weight: 600">
											{{ fmtCurrency(adjustmentsTotal) }}
										</td>
									</tr>
									<tr>
										<td style="color: #484d54">Deductions (incl. taxes &amp; fees)</td>
										<td class="right" style="font-weight: 600">
											− {{ fmtCurrency(deductionsTotal) }}
										</td>
									</tr>
									<tr>
										<td style="color: #484d54">Net before payout</td>
										<td class="right" style="font-weight: 700">
											{{ fmtCurrency(netBeforePayout) }}
										</td>
									</tr>
									<tr v-if="payout && payout.amount">
										<td style="color: #484d54">
											Payout sent — {{ payoutMethodLabel(payout.method) }}
										</td>
										<td class="right" style="font-weight: 700">
											− {{ fmtCurrency(payoutAmount) }}
										</td>
									</tr>
									<tr>
										<td style="color: #484d54">Ending balance</td>
										<td class="right" style="font-weight: 700">{{ fmtCurrency(endingBalance) }}</td>
									</tr>
								</tbody>
							</table>
						</div>
					</div>

					<!-- Payout details -->
					<div v-if="payout" class="card mt-16 p-16" style="border-radius: 6px">
						<p class="subtitle">Payout details</p>
						<table class="table" style="margin-top: 8px">
							<tbody>
								<tr>
									<td style="width: 25%"><span class="muted">Method</span></td>
									<td style="width: 75%">{{ payoutMethodLabel(payout.method) }}</td>
								</tr>
								<tr>
									<td><span class="muted">Destination</span></td>
									<td>{{ payout.destination || '—' }}</td>
								</tr>
								<tr>
									<td><span class="muted">Date</span></td>
									<td>{{ fmtDate(payout.date) || '—' }}</td>
								</tr>
								<tr>
									<td><span class="muted">Status</span></td>
									<td>{{ payout.status || '—' }}</td>
								</tr>
								<tr>
									<td><span class="muted">Reference</span></td>
									<td>{{ payout.reference || '—' }}</td>
								</tr>
								<tr>
									<td><span class="muted">Fees</span></td>
									<td>{{ fmtCurrency(payout.fees || 0) }}</td>
								</tr>
								<tr>
									<td><span class="muted">Amount</span></td>
									<td>{{ fmtCurrency(payout.amount || 0) }}</td>
								</tr>
							</tbody>
						</table>
					</div>

					<!-- Breakdown -->
					<div class="card mt-16" style="overflow: hidden; border-radius: 6px">
						<div style="background: #f3f4f6; padding: 12px 16px; font-weight: 600; font-size: 14px">
							Breakdown
						</div>
						<div class="p-16">
							<table class="table">
								<thead>
									<tr>
										<th style="width: 66%">Description</th>
										<th class="right" style="width: 17%">Type</th>
										<th class="right" style="width: 17%">Amount</th>
									</tr>
								</thead>
								<tbody>
									<tr v-for="(item, idx) in breakdown" :key="'b-' + idx">
										<td>
											<div style="font-weight: 600">{{ item.label }}</div>
											<div v-if="item.note" class="small muted" style="text-transform: none">
												{{ item.note }}
											</div>
										</td>
										<td class="right">
											<span v-if="item.type === 'deduction'" style="color: #cb2245">Deduction</span>
											<span v-else style="color: #00af5c">Earning</span>
										</td>
										<td class="right" style="font-weight: 600">
											<span v-if="item.type === 'deduction'">− </span>{{ fmtCurrency(item.amount) }}
										</td>
									</tr>

									<tr v-for="(adj, idx) in adjustments" :key="'adj-' + idx">
										<td>
											<div style="font-weight: 600">{{ adj.label }}</div>
											<div v-if="adj.note" class="small muted" style="text-transform: none">
												{{ adj.note }}
											</div>
										</td>
										<td class="right">Adjustment</td>
										<td class="right" style="font-weight: 600">{{ fmtCurrency(adj.amount) }}</td>
									</tr>

									<tr v-for="(tax, idx) in taxes" :key="'tax-' + idx">
										<td>
											<div style="font-weight: 600">{{ tax.label }}</div>
											<div v-if="tax.note" class="small muted" style="text-transform: none">
												{{ tax.note }}
											</div>
										</td>
										<td class="right">Tax</td>
										<td class="right" style="font-weight: 600">− {{ fmtCurrency(tax.amount) }}</td>
									</tr>

									<tr v-for="(od, idx) in otherDeductions" :key="'od-' + idx">
										<td>
											<div style="font-weight: 600">{{ od.label }}</div>
											<div v-if="od.note" class="small muted" style="text-transform: none">
												{{ od.note }}
											</div>
										</td>
										<td class="right">Deduction</td>
										<td class="right" style="font-weight: 600">− {{ fmtCurrency(od.amount) }}</td>
									</tr>
								</tbody>
							</table>
						</div>
					</div>

					<div class="card mt-16 p-16" style="border-radius: 6px">
						<p class="small muted" style="text-transform: none">
							This statement is provided for your records and is not a tax document. Keep this for
							your accounting. If you have questions, contact
							{{ company.supportEmail || 'support' }}.
						</p>
						<ul v-if="statement.notes && statement.notes.length" class="list">
							<li
								v-for="(note, idx) in statement.notes"
								:key="'n-' + idx"
								class="small"
								style="color: #484d54"
							>
								{{ note }}
							</li>
						</ul>
					</div>
				</div>
			</div>
		</Body>
	</Html>
</template>
