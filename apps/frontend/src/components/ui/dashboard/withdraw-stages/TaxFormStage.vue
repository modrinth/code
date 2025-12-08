<template>
	<div class="flex flex-col gap-2.5 sm:gap-3">
		<div class="flex flex-col gap-3">
			<div class="flex w-full flex-col gap-1 sm:flex-row sm:justify-between sm:gap-0">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.withdrawLimit) }}</span>
				<div>
					<span class="text-orange">{{ formatMoney(usedLimit) }}</span> /
					<span class="text-contrast">{{ formatMoney(600) }}</span>
				</div>
			</div>
			<div class="flex h-2.5 w-full overflow-hidden rounded-full bg-surface-2">
				<div
					v-if="usedLimit > 0"
					class="gradient-border bg-orange"
					:style="{ width: `${(usedLimit / 600) * 100}%` }"
				></div>
			</div>
		</div>
		<template v-if="remainingLimit > 0">
			<span>
				<IntlFormatted
					:message-id="messages.nearingThreshold"
					:values="{
						amountRemaining: formatMoney(remainingLimit),
					}"
				>
					<template #b="{ children }">
						<span class="font-medium">
							<component :is="() => normalizeChildren(children)" />
						</span>
					</template>
				</IntlFormatted>
			</span>
			<Admonition
				type="warning"
				show-actions-underneath
				:header="formatMessage(messages.taxFormRequiredHeader)"
			>
				<span class="text-sm font-normal md:text-base">
					{{
						formatMessage(messages.taxFormRequiredBodyWithLimit, {
							limit: formatMoney(remainingLimit),
						})
					}}
				</span>
				<template #icon="{ iconClass }">
					<FileTextIcon :class="iconClass" />
				</template>
				<template #actions>
					<ButtonStyled color="orange">
						<button @click="showTaxFormModal">
							{{ formatMessage(messages.completeTaxForm) }}
						</button>
					</ButtonStyled>
				</template>
			</Admonition>
		</template>
		<template v-else>
			<span>
				<IntlFormatted
					:message-id="messages.withdrawLimitUsed"
					:values="{ withdrawLimit: formatMoney(600) }"
				>
					<template #b="{ children }">
						<b>
							<component :is="() => normalizeChildren(children)" />
						</b>
					</template>
				</IntlFormatted>
			</span>
		</template>
	</div>
</template>

<script setup lang="ts">
import { FileTextIcon } from '@modrinth/assets'
import { Admonition, ButtonStyled, normalizeChildren } from '@modrinth/ui'
import { formatMoney } from '@modrinth/utils'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { IntlFormatted } from '@vintl/vintl/components'
import { computed } from 'vue'

import { TAX_THRESHOLD_ACTUAL } from '@/providers/creator-withdraw.ts'

const props = defineProps<{
	balance: any
	onShowTaxForm: () => void
}>()

const { formatMessage } = useVIntl()

const usedLimit = computed(() => props.balance?.withdrawn_ytd ?? 0)
const remainingLimit = computed(() => {
	const raw = TAX_THRESHOLD_ACTUAL - usedLimit.value
	if (raw <= 0) return 0
	const cents = Math.floor(raw * 100)
	return cents / 100
})

function showTaxFormModal() {
	props.onShowTaxForm()
}

const messages = defineMessages({
	withdrawLimit: {
		id: 'dashboard.creator-withdraw-modal.withdraw-limit',
		defaultMessage: 'Withdraw limit',
	},
	nearingThreshold: {
		id: 'dashboard.creator-withdraw-modal.nearing-threshold',
		defaultMessage:
			"You're nearing the withdraw threshold. You can withdraw <b>{amountRemaining}</b> now, but a tax form is required for more.",
	},
	taxFormRequiredHeader: {
		id: 'dashboard.creator-withdraw-modal.tax-form-required.header',
		defaultMessage: 'Tax form required',
	},
	taxFormRequiredBody: {
		id: 'dashboard.creator-withdraw-modal.tax-form-required.body',
		defaultMessage:
			'To withdraw your full <b>{available}</b> available balance please complete the form below. It is required for tax reporting and only needs to be done once.',
	},
	taxFormRequiredBodyWithLimit: {
		id: 'dashboard.creator-withdraw-modal.tax-form-required.body-with-limit',
		defaultMessage:
			"You must complete a W-9 or W-8 form for Modrinth's tax records so we remain compliant with tax regulations.",
	},
	completeTaxForm: {
		id: 'dashboard.creator-withdraw-modal.complete-tax-form',
		defaultMessage: 'Complete tax form',
	},
	withdrawLimitUsed: {
		id: 'dashboard.creator-withdraw-modal.withdraw-limit-used',
		defaultMessage:
			"You've used up your <b>{withdrawLimit}</b> withdrawal limit. You must complete a tax form to withdraw more.",
	},
})
</script>

<style lang="css" scoped>
.gradient-border {
	position: relative;

	&::after {
		content: '';
		position: absolute;
		inset: 0;
		background: linear-gradient(to bottom, rgba(255, 255, 255, 0.3), transparent);
		border-radius: inherit;
		mask:
			linear-gradient(#fff 0 0) content-box,
			linear-gradient(#fff 0 0);
		mask-composite: xor;
		padding: 2px;
		pointer-events: none;
	}
}
</style>
