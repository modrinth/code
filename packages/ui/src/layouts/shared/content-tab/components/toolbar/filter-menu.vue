<script setup lang="ts">
import { FilterIcon } from '@modrinth/assets'

import DropdownFilterBar, {
	type DropdownFilterBarCategory,
} from '#ui/components/base/DropdownFilterBar.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import type { ContentMetadataFilterValue } from '../../composables/use-content-metadata-filters'

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
const previewTriggerClass =
	'!h-10 !rounded-[14px] !border !border-solid !border-brand !bg-brand-highlight !px-3 !text-sm !font-medium !text-brand !shadow-[0_1px_1.5px_rgba(0,0,0,0.15)] transition-all duration-100 active:scale-[0.97] hover:!bg-brand-highlight focus-visible:!outline-none focus-visible:!ring-4 focus-visible:!ring-brand-shadow [&>svg]:!size-5 [&>svg]:!text-brand'
</script>

<template>
	<div class="flex min-w-0 flex-wrap items-center justify-end gap-1.5 [&_[data-button]]:!h-10">
		<DropdownFilterBar
			v-model="selected"
			:categories="categories"
			:show-label="false"
			:add-label="formatMessage(messages.filter)"
			:add-button-class="triggerClass"
			:preview-trigger-class="previewTriggerClass"
			add-button-size="lg"
			checkbox-position="right"
			menu-align="end"
			apply-immediately
		>
			<template #add-button>
				<FilterIcon />
			</template>
			<template #preview-content="{ label, summary }">
				<span class="min-w-0 flex-1 truncate">
					<span class="font-medium">{{ label }}:</span>
					<span class="ml-1 font-semibold text-contrast">{{ summary }}</span>
				</span>
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
