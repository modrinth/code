<template>
	<component
		:is="action.icon"
		v-if="action.icon"
		:class="action.iconClass"
		aria-hidden="true"
		v-bind="action.iconProps"
	/>
	<span v-if="!action.labelHidden && !action.circular">{{ action.label }}</span>
</template>

<script setup lang="ts">
import type { PageHeaderAction } from './types'

defineProps<{
	action: PageHeaderAction
}>()
</script>
