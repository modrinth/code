<script setup lang="ts">
import { InfoIcon } from '@modrinth/assets'
import { formatPrice } from '@modrinth/utils'
import { type MessageDescriptor, useVIntl } from '@vintl/vintl'
import { Menu } from 'floating-vue'
import { computed, inject, type Ref } from 'vue'

import { monthsInInterval, type ServerBillingInterval, type ServerPlan } from '../../utils/billing'
import ServersSpecs from './ServersSpecs.vue'

const props = withDefaults(
	defineProps<{
		plan: ServerPlan
		title: MessageDescriptor
		description: MessageDescriptor
		buttonColor?: 'standard' | 'brand' | 'red' | 'orange' | 'green' | 'blue' | 'purple'
		mostPopular?: boolean
		selected?: boolean
	}>(),
	{
		buttonColor: 'standard',
		mostPopular: false,
		selected: false,
	},
)

const emit = defineEmits<{
	(e: 'select', plan: ServerPlan): void
}>()

const { formatMessage, locale } = useVIntl()

// TODO: Use DI framework when merged.
const selectedInterval = inject<Ref<ServerBillingInterval>>('selectedInterval')
const currency = inject<string>('currency')

const perMonth = computed(() => {
	if (!props.plan || !currency || !selectedInterval?.value) return undefined
	const total = props.plan.prices?.find((x) => x.currency_code === currency)?.prices?.intervals?.[
		selectedInterval.value
	]
	if (!total) return undefined
	return total / monthsInInterval[selectedInterval.value]
})

const mostPopularStyle = computed(() => {
	if (!props.mostPopular) return undefined
	const style: Record<string, string> = {
		backgroundImage:
			'radial-gradient(86.12% 101.64% at 95.97% 94.07%, rgba(27, 217, 106, 0.23) 0%, rgba(14, 115, 56, 0.2) 100%)',
		boxShadow: '0px 12px 38.1px rgba(27, 217, 106, 0.13)',
	}

	if (!props.selected) {
		style.borderColor = 'rgba(12, 107, 52, 0.55)'
	}

	return style
})
</script>

<template>
	<div
		class="rounded-2xl p-4 font-semibold transition-all duration-300 experimental-styles-within h-full border-2 border-solid cursor-pointer select-none"
		:class="{
			'bg-brand-highlight border-brand': selected,
			'bg-button-bg border-transparent': !selected,
			'!bg-bg': mostPopular,
		}"
		:style="mostPopularStyle"
		role="button"
		tabindex="0"
		:aria-pressed="selected"
		@click="emit('select', plan)"
		@keydown.enter.prevent="emit('select', plan)"
		@keydown.space.prevent="emit('select', plan)"
	>
		<div class="flex h-full flex-col justify-between gap-2">
			<div class="flex flex-col">
				<div class="flex items-center justify-between">
					<span class="text-2xl font-semibold text-contrast">
						{{ formatMessage(title) }}
					</span>
					<div
						v-if="mostPopular"
						class="relative w-fit rounded-full bg-highlight-green px-3 py-1 text-sm font-bold text-brand backdrop-blur-lg"
					>
						Most Popular
					</div>
				</div>
				<span class="m-0 text-lg font-bold text-contrast">
					{{ formatPrice(locale, perMonth, currency, true) }}
					<span class="text-sm font-semibold text-secondary">
						/ month{{ selectedInterval !== 'monthly' ? `, billed ${selectedInterval}` : '' }}
					</span>
				</span>
			</div>

			<span class="text-sm">{{ formatMessage(description) }}</span>

			<div class="w-fit">
				<Menu
					placement="bottom-start"
					:triggers="['hover', 'focus']"
					:auto-hide="true"
					:delay="{ show: 100, hide: 120 }"
					:distance="6"
				>
					<template #default="{ shown }">
						<div
							class="flex w-fit items-center gap-2 cursor-help text-sm font-medium cursor-default select-none outline-none"
							:class="shown ? 'text-primary' : 'text-secondary'"
							role="button"
							tabindex="0"
							aria-haspopup="true"
							:aria-expanded="shown"
						>
							<InfoIcon />
							View plan details
						</div>
					</template>

					<template #popper>
						<div class="w-fit rounded-md border border-contrast/10 p-3 shadow-lg">
							<ServersSpecs
								:ram="plan.metadata.ram!"
								:storage="plan.metadata.storage!"
								:cpus="plan.metadata.cpu!"
							/>
						</div>
					</template>
				</Menu>
			</div>
		</div>
	</div>
</template>
