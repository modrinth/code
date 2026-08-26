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

<script setup lang="ts">
import { TriangleAlertIcon, XCircleIcon } from '@modrinth/assets'
import { type MessageDescriptor, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

interface ValidationCheck {
	severity: 'valid' | 'warn' | 'error'
	message?: MessageDescriptor
	values?: Record<string, unknown>
}

const props = withDefaults(defineProps<{ check?: ValidationCheck | null }>(), {
	check: null,
})

const { formatMessage } = useVIntl()

const icon = computed(() => (props.check?.severity === 'error' ? XCircleIcon : TriangleAlertIcon))

const message = computed(() => {
	if (!props.check?.message) return undefined
	return formatMessage(props.check.message, props.check.values)
})
</script>
