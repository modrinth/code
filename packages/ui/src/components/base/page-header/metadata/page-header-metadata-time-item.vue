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
		<span>{{ displayLabel }}</span>
	</PageHeaderMetadataItem>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { useFormatDateTime, useRelativeTime } from '../../../../composables'
import type { PageHeaderMetadataItemProps } from '../types'
import PageHeaderMetadataItem from './page-header-metadata-item.vue'

const props = withDefaults(
	defineProps<
		PageHeaderMetadataItemProps & {
			date: string | number | Date
			label?: string
			relative?: boolean
		}
	>(),
	{
		label: '',
		relative: true,
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

const formatDateTime = useFormatDateTime({
	dateStyle: 'medium',
	timeStyle: 'short',
})
const formatRelativeTime = useRelativeTime()

const parsedDate = computed(() => {
	const date = props.date instanceof Date ? props.date : new Date(props.date)
	return Number.isNaN(date.getTime()) ? null : date
})
const absoluteDate = computed(() => {
	if (!parsedDate.value) return ''
	return formatDateTime(parsedDate.value)
})
const formattedDate = computed(() => {
	if (!parsedDate.value) return ''
	if (!props.relative) return absoluteDate.value
	return formatRelativeTime(parsedDate.value)
})
const resolvedTooltip = computed(() => props.tooltip ?? (absoluteDate.value || undefined))
const displayLabel = computed(() => [props.label, formattedDate.value].filter(Boolean).join(' '))
</script>
