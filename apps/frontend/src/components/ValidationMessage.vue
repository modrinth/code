<template>
	<div v-if="validations.length > 0" class="flex w-full flex-col gap-1.5">
		<div
			v-for="(validation, index) in validations"
			:key="validation.code ?? validation.message?.id ?? index"
			class="flex w-full items-center gap-1.5"
			:class="{
				'text-red': validation.severity === 'error',
				'text-orange': validation.severity === 'warn' || validation.severity === 'warning',
				'text-purple': validation.severity === 'suggestion',
			}"
		>
			<component
				:is="
					validation.severity === 'error'
						? XCircleIcon
						: validation.severity === 'suggestion'
							? LightBulbIcon
							: TriangleAlertIcon
				"
				class="my-auto"
			/>
			{{ validation.message ? formatMessage(validation.message, validation.values) : undefined }}
		</div>
	</div>
</template>

<script setup lang="ts">
import { LightBulbIcon, TriangleAlertIcon, XCircleIcon } from '@modrinth/assets'
import { type MessageDescriptor, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

interface ValidationCheck {
	code?: string
	severity: 'valid' | 'warn' | 'warning' | 'suggestion' | 'error'
	message?: MessageDescriptor
	values?: Record<string, unknown>
}

const props = withDefaults(defineProps<{ check?: ValidationCheck | ValidationCheck[] | null }>(), {
	check: null,
})

const { formatMessage } = useVIntl()

const validations = computed(() =>
	(Array.isArray(props.check) ? props.check : props.check ? [props.check] : []).filter(
		(validation) => validation.severity !== 'valid',
	),
)
</script>
