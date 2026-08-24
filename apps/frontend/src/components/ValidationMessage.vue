<template>
	<div
		v-if="check && check.severity !== 'valid'"
		class="flex w-full items-center gap-1.5"
		:class="check.severity === 'error' ? 'text-red' : 'text-orange'"
	>
		<component :is="icon" class="my-auto" />
		{{ message }}
	</div>
</template>

<script setup>
import { TriangleAlertIcon, XCircleIcon } from '@modrinth/assets'
import { useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

const props = defineProps({
	check: { type: Object, default: null },
})

const { formatMessage } = useVIntl()

const icon = computed(() => (props.check?.severity === 'error' ? XCircleIcon : TriangleAlertIcon))

const message = computed(() => {
	if (!props.check?.message) return undefined
	return formatMessage(props.check.message, props.check.values)
})
</script>
