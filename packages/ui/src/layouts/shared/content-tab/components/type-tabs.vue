<script setup lang="ts">
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonProjectTypeCategoryMessages } from '#ui/utils/common-messages'

import type { ContentType } from '../types'

defineProps<{
	types: ContentType[]
}>()

const active = defineModel<ContentType>({ required: true })

const { formatMessage } = useVIntl()

const messages = defineMessages({
	label: {
		id: 'content.page-layout.type-tabs.label',
		defaultMessage: 'Content types',
	},
})
</script>

<template>
	<div
		role="tablist"
		:aria-label="formatMessage(messages.label)"
		class="flex min-w-0 items-center gap-1 overflow-x-auto rounded-[14px] border border-solid border-surface-5 p-[3px] shadow-[0_24px_48px_rgba(0,0,0,0.03),0_10px_18px_rgba(0,0,0,0.03),0_5px_8px_rgba(0,0,0,0.04),0_2px_4px_rgba(0,0,0,0.04)]"
	>
		<button
			v-for="type in types"
			:key="type"
			role="tab"
			type="button"
			:aria-selected="active === type"
			class="flex h-[34px] shrink-0 items-center whitespace-nowrap rounded-xl border border-solid px-3 text-base font-semibold leading-5 transition-colors duration-100 focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-brand-shadow"
			:class="
				active === type
					? 'type-tab-active border-transparent text-brand'
					: 'border-transparent text-primary hover:bg-surface-4'
			"
			@click="active = type"
		>
			{{ formatMessage(commonProjectTypeCategoryMessages[type]) }}
		</button>
	</div>
</template>

<style scoped>
.type-tab-active {
	background-color: color-mix(in srgb, var(--color-brand) 30%, var(--surface-3));
	border-color: color-mix(in srgb, var(--color-brand) 60%, transparent);
}
</style>
