<script setup lang="ts">
import { FilterIcon } from '@modrinth/assets'

import DropdownFilterBar, {
	type DropdownFilterBarCategory,
} from '#ui/components/base/DropdownFilterBar.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import type { ContentMetadataFilterValue } from '../composables/use-content-metadata-filters'

defineProps<{
	categories: DropdownFilterBarCategory[]
}>()

const selected = defineModel<ContentMetadataFilterValue>({ required: true })

const { formatMessage } = useVIntl()

const messages = defineMessages({
	filter: {
		id: 'content.page-layout.filter.add',
		defaultMessage: 'Filter',
	},
})

const triggerClass =
	'!size-10 !rounded-full !border-0 !bg-transparent !shadow-none !p-0 !text-primary hover:!bg-surface-4 focus-visible:!outline-none focus-visible:!ring-4 focus-visible:!ring-brand-shadow [&>svg]:!size-5'
</script>

<template>
	<div class="flex min-w-0 flex-wrap items-center justify-end gap-1.5 [&_[data-button]]:!h-10">
		<DropdownFilterBar
			v-model="selected"
			:categories="categories"
			:show-label="false"
			:add-label="formatMessage(messages.filter)"
			:add-button-class="triggerClass"
			:show-previews="false"
			:show-clear-button="false"
			add-button-size="lg"
			checkbox-position="right"
			menu-align="end"
			apply-immediately
		>
			<template #add-button>
				<FilterIcon />
			</template>
			<template #option="{ option, selected: optionSelected }">
				<span
					class="min-w-0 truncate font-semibold leading-tight"
					:class="optionSelected ? 'text-contrast' : 'text-primary'"
				>
					{{ option.label }}
				</span>
			</template>
		</DropdownFilterBar>
	</div>
</template>
