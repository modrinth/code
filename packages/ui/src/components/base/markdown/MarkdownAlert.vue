<template>
	<Admonition
		v-if="noBody"
		:type="admonitionTypeByAlertType[type] ?? 'neutral'"
		:header="title ?? capitalize(type)"
		:foldable="foldable"
		:default-open="!!open"
	/>
	<Admonition
		v-else
		:type="admonitionTypeByAlertType[type] ?? 'neutral'"
		:header="title ?? capitalize(type)"
		:foldable="foldable"
		:default-open="!!open"
	>
		<slot />
	</Admonition>
</template>

<script setup lang="ts">
import Admonition from '../Admonition.vue'

type AdmonitionType = NonNullable<InstanceType<typeof Admonition>['$props']['type']>

const admonitionTypeByAlertType: Record<string, AdmonitionType> = {
	note: 'neutral',
	tip: 'success',
	important: 'info',
	warning: 'warning',
	caution: 'critical',
}

function capitalize(value: string): string {
	return value.charAt(0).toUpperCase() + value.slice(1)
}

defineProps<{
	type: string
	title?: string
	foldable?: boolean
	open?: boolean
	noBody?: boolean
}>()
</script>

<style scoped>
:deep(p) {
	margin: 0;
	line-height: inherit;
}
</style>
