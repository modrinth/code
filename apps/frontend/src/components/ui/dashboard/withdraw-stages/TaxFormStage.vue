<template>
	<div class="flex w-full flex-row justify-between">
		<span class="font-semibold text-contrast">{{ formatMessage(messages.withdrawRemaining) }}</span>
		<div>
			<span class="text-orange">{{ formatMoney(remainingLimit) }}</span> /
			{{ formatMoney(600) }}
		</div>
	</div>
	<div class="flex h-2 w-full overflow-hidden rounded-full bg-button-bg">
		<div class="bg-orange" :style="{ width: `${(usedLimit / 600) * 100}%` }"></div>
	</div>
	<template v-if="remainingLimit > 0">
		<span>
			<IntlFormatted
				:message-id="messages.nearingThreshold"
				:values="{
					amount600: formatMoney(600),
					amountRemaining: formatMoney(remainingLimit),
				}"
			>
				<template #b="{ children }">
					<b><component :is="() => normalizeChildren(children)" /></b>
				</template>
			</IntlFormatted>
		</span>
		<Admonition
			type="warning"
			show-actions-underneath
			:header="formatMessage(messages.taxFormRequiredHeader)"
		>
			<span>
				<IntlFormatted
					:message-id="messages.taxFormRequiredBody"
					:values="{ available: formatMoney(balance?.available) }"
				>
					<template #b="{ children }">
						<b><component :is="() => normalizeChildren(children)" /></b>
					</template>
				</IntlFormatted>
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
				:values="{ amount600: formatMoney(600) }"
			>
				<template #b="{ children }">
					<b><component :is="() => normalizeChildren(children)" /></b>
				</template>
			</IntlFormatted>
		</span>
	</template>
</template>

<script setup lang="ts">
import { FileTextIcon } from '@modrinth/assets'
import { Admonition, ButtonStyled } from '@modrinth/ui'
import { formatMoney } from '@modrinth/utils'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { IntlFormatted } from '@vintl/vintl/components'
import { computed } from 'vue'

import { normalizeChildren } from '@/utils/vue-children.ts'

const props = defineProps<{
	balance: any
	onShowTaxForm: () => void
}>()

const { formatMessage } = useVIntl()

const usedLimit = computed(() => props.balance?.withdrawn_ytd ?? 0)
const remainingLimit = computed(() => {
	const raw = 600 - usedLimit.value
	if (raw <= 0) return 0
	const cents = Math.floor(raw * 100)
	return cents / 100
})

function showTaxFormModal() {
	props.onShowTaxForm()
}

const messages = defineMessages({
	withdrawRemaining: {
		id: 'dashboard.creator-withdraw-modal.withdraw-remaining',
		defaultMessage: 'Withdraw remaining',
	},
	nearingThreshold: {
		id: 'dashboard.creator-withdraw-modal.nearing-threshold',
		defaultMessage:
			"You're nearing the {amount600} withdrawal threshold. You can withdraw up to <b>{amountRemaining}</b> now, but you'll need to complete a tax form to withdraw more.",
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
	completeTaxForm: {
		id: 'dashboard.creator-withdraw-modal.complete-tax-form',
		defaultMessage: 'Complete tax form',
	},
	withdrawLimitUsed: {
		id: 'dashboard.creator-withdraw-modal.withdraw-limit-used',
		defaultMessage:
			"You've used up your <b>{amount600}</b> withdrawal limit. You must complete a tax form to withdraw more.",
	},
})
</script>
