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
import { computed, onScopeDispose, shallowRef, watch } from 'vue'

interface ValidationCheck {
	code?: string
	severity: 'valid' | 'warn' | 'warning' | 'suggestion' | 'error'
	message?: MessageDescriptor
	values?: Record<string, unknown>
}

type ValidationCheckInput = ValidationCheck | ValidationCheck[] | null

const props = withDefaults(
	defineProps<{
		check?: ValidationCheckInput
		debounce?: number
	}>(),
	{
		check: null,
		debounce: 300,
	},
)

const { formatMessage } = useVIntl()
const displayedCheck = shallowRef<ValidationCheckInput>(props.check)
let debounceTimer: ReturnType<typeof setTimeout> | undefined

watch(
	() => props.check,
	(check) => {
		clearTimeout(debounceTimer)
		if (props.debounce <= 0) {
			displayedCheck.value = check
			return
		}

		debounceTimer = setTimeout(() => {
			displayedCheck.value = check
		}, props.debounce)
	},
)

onScopeDispose(() => clearTimeout(debounceTimer))

const validations = computed(() =>
	(Array.isArray(displayedCheck.value)
		? displayedCheck.value
		: displayedCheck.value
			? [displayedCheck.value]
			: []
	).filter((validation) => validation.severity !== 'valid'),
)
</script>
