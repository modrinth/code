<template>
	<component
		:is="check.severity === 'error' ? XCircleIcon : TriangleAlertIcon"
		v-if="check && check.severity !== 'valid'"
		v-tooltip="formatMessage(check.message, check.values)"
		:class="['size-6 animate-pulse', check.severity === 'error' ? 'text-red' : 'text-orange']"
	/>
</template>

<script setup>
import { TriangleAlertIcon, XCircleIcon } from '@modrinth/assets'
import { useVIntl } from '@modrinth/ui'

defineProps({
	check: { type: Object, default: null },
})

const { formatMessage } = useVIntl()
</script>
