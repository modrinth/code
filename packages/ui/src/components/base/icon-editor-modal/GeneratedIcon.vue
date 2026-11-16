<template>
	<div class="overflow-hidden" :style="iconBackgroundStyle(config.background)">
		<img
			v-if="symbolAsset"
			:src="symbolAsset"
			:alt="alt"
			class="size-full select-none object-cover"
			draggable="false"
		/>
	</div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { getIconSymbolAsset } from './editor-catalog'
import { iconBackgroundStyle } from './generated-icon'
import type { IconConfig } from './types'

const props = withDefaults(
	defineProps<{
		config: IconConfig
		alt?: string
	}>(),
	{ alt: '' },
)

const symbolAsset = computed(() => getIconSymbolAsset(props.config.symbol))
</script>
