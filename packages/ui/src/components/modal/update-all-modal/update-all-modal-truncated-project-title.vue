<script setup lang="ts">
import { useUpdateAllTruncatedTooltip } from './use-update-all-truncated-tooltip'

const props = withDefaults(
	defineProps<{
		title: string
		as?: 'span' | 'button' | 'h3'
	}>(),
	{ as: 'span' },
)

const { element, tooltip } = useUpdateAllTruncatedTooltip(() => props.title)
</script>

<template>
	<component :is="as" ref="element" v-tooltip="tooltip">{{ title }}</component>
</template>
