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

import type { PageHeaderMetadataItemProps } from '../types'
import PageHeaderMetadataItem from './page-header-metadata-item.vue'

const props = withDefaults(
	defineProps<
		PageHeaderMetadataItemProps & {
			date: string | number | Date
			label?: string
			locale?: string
			relative?: boolean
		}
	>(),
	{
		label: '',
		locale: undefined,
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

const parsedDate = computed(() => {
	const date = props.date instanceof Date ? props.date : new Date(props.date)
	return Number.isNaN(date.getTime()) ? null : date
})
const absoluteDate = computed(() => {
	if (!parsedDate.value) return ''
	return new Intl.DateTimeFormat(props.locale, {
		dateStyle: 'medium',
		timeStyle: 'short',
	}).format(parsedDate.value)
})
const formattedDate = computed(() => {
	if (!parsedDate.value) return ''
	if (!props.relative) return absoluteDate.value
	return formatRelativeDate(parsedDate.value, props.locale)
})
const resolvedTooltip = computed(() => props.tooltip ?? (absoluteDate.value || undefined))
const displayLabel = computed(() => [props.label, formattedDate.value].filter(Boolean).join(' '))

function formatRelativeDate(date: Date, locale?: string) {
	const diffSeconds = (date.getTime() - Date.now()) / 1000
	const absoluteSeconds = Math.abs(diffSeconds)
	const formatter = new Intl.RelativeTimeFormat(locale, { numeric: 'auto' })

	if (absoluteSeconds < 60) {
		return formatter.format(Math.round(diffSeconds), 'second')
	}
	if (absoluteSeconds < 3600) {
		return formatter.format(Math.round(diffSeconds / 60), 'minute')
	}
	if (absoluteSeconds < 86400) {
		return formatter.format(Math.round(diffSeconds / 3600), 'hour')
	}
	if (absoluteSeconds < 2592000) {
		return formatter.format(Math.round(diffSeconds / 86400), 'day')
	}
	if (absoluteSeconds < 31536000) {
		return formatter.format(Math.round(diffSeconds / 2592000), 'month')
	}
	return formatter.format(Math.round(diffSeconds / 31536000), 'year')
}
</script>
