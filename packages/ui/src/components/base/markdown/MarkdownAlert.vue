<template>
	<Admonition :type="admonitionTypeByMarker[as]" :header="capitalize(as)">
		<slot />
	</Admonition>
</template>

<script lang="ts">
export const alertMarkerTypes = ['note', 'tip', 'important', 'warning', 'caution'] as const
</script>

<script setup lang="ts">
import Admonition from '../Admonition.vue'

type AdmonitionType = NonNullable<InstanceType<typeof Admonition>['$props']['type']>

const admonitionTypeByMarker: Record<(typeof alertMarkerTypes)[number], AdmonitionType> = {
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
	as: (typeof alertMarkerTypes)[number]
}>()
</script>
