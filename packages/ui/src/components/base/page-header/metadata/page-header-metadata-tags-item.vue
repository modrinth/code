<template>
	<PageHeaderMetadataItem
		class="!text-wrap"
		:icon="icon"
		:icon-props="iconProps"
		:icon-class="iconClass"
		:tooltip="tooltip"
		:aria-label="ariaLabel"
		:to="to"
		:action="action"
		:disabled="disabled"
	>
		<div class="flex min-w-0 flex-wrap gap-2">
			<slot />
		</div>
	</PageHeaderMetadataItem>
</template>

<script setup lang="ts">
import type { PageHeaderMetadataItemProps } from '../types'
import PageHeaderMetadataItem from './page-header-metadata-item.vue'

withDefaults(defineProps<PageHeaderMetadataItemProps>(), {
	icon: undefined,
	iconProps: undefined,
	iconClass: undefined,
	tooltip: undefined,
	ariaLabel: undefined,
	to: undefined,
	action: undefined,
	disabled: false,
})
</script>
