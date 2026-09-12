<script setup lang="ts">
import { activeTooltip } from '../../providers/tooltip'
import Tooltip from './Tooltip.vue'
</script>
<template>
	<Tooltip
		:reference="activeTooltip.reference"
		:text="activeTooltip.text"
		:content="activeTooltip.content"
		:placement="activeTooltip.placement"
		:theme="activeTooltip.theme"
		:panel-class="activeTooltip.panelClass"
	/>
</template>
