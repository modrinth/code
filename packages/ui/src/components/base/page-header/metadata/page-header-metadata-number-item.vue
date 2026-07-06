<template>
	<PageHeaderMetadataItem
		:icon="icon"
		:icon-props="iconProps"
		:icon-class="iconClass"
		:tooltip="resolvedTooltip"
		:aria-label="ariaLabel"
		:to="to"
		:action="action"
		:disabled="disabled"
	>
		<span>{{ formattedValue }}</span>
		<span v-if="label">{{ label }}</span>
	</PageHeaderMetadataItem>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import type { PageHeaderMetadataItemProps } from '../types'
import PageHeaderMetadataItem from './page-header-metadata-item.vue'

const props = withDefaults(
	defineProps<
		PageHeaderMetadataItemProps & {
			value: number
			label?: string
			locale?: string
			compact?: boolean
		}
	>(),
	{
		label: '',
		locale: undefined,
		compact: true,
		icon: undefined,
		iconProps: undefined,
		iconClass: undefined,
		tooltip: undefined,
		ariaLabel: undefined,
		to: undefined,
		action: undefined,
		disabled: false,
	},
)

const formattedValue = computed(() =>
	new Intl.NumberFormat(props.locale, {
		notation: props.compact ? 'compact' : 'standard',
		maximumFractionDigits: props.compact ? 1 : 0,
	}).format(props.value),
)
const fullValue = computed(() => new Intl.NumberFormat(props.locale).format(props.value))
const resolvedTooltip = computed(() => {
	if (props.tooltip) return props.tooltip
	if (!props.compact || formattedValue.value === fullValue.value) return undefined
	return [fullValue.value, props.label].filter(Boolean).join(' ')
})
</script>
