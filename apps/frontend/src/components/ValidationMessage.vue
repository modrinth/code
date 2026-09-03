<template>
	<div v-if="validations.length > 0" class="flex w-full flex-col gap-1.5">
		<div
			v-for="(validation, index) in validations"
			:key="validation.code ?? validation.message?.id ?? index"
			class="flex w-full items-start gap-1.5"
			:class="{
				'text-red': validation.severity === 'error',
				'text-orange': validation.severity === 'warning',
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
				class="mt-0.5"
			/>
			{{ validation.message ? formatMessage(validation.message, validation.values) : undefined }}
		</div>
	</div>
</template>

<script setup lang="ts">
import { LightBulbIcon, TriangleAlertIcon, XCircleIcon } from '@modrinth/assets'
import type { FieldValidationMessage } from '@modrinth/moderation'
import { injectProjectPageContext, useVIntl } from '@modrinth/ui'
import { computed, onScopeDispose, ref, shallowRef, watch } from 'vue'

type ValidationCheck = Omit<FieldValidationMessage, 'code'> & { code?: string }

type ValidationCheckInput = ValidationCheck | ValidationCheck[] | null

const props = withDefaults(
	defineProps<{
		check?: ValidationCheckInput
		debounce?: number
		projectField?: unknown
		currentField?: unknown
	}>(),
	{
		check: null,
		debounce: 300,
	},
)

const { formatMessage } = useVIntl()
const { projectValidationLoading } = injectProjectPageContext()
const displayedCheck = shallowRef<ValidationCheckInput>(props.check)
const validationIsStale = ref(props.projectField !== props.currentField)
let debounceTimer: ReturnType<typeof setTimeout> | undefined
let validationRefreshCompleted = false

function updateDisplayedCheck(check: ValidationCheckInput) {
	clearTimeout(debounceTimer)
	const update = () => {
		displayedCheck.value = check
		if (
			validationRefreshCompleted &&
			!projectValidationLoading.value &&
			props.projectField === props.currentField
		) {
			validationIsStale.value = false
			validationRefreshCompleted = false
		}
	}

	if (props.debounce <= 0) {
		update()
		return
	}

	debounceTimer = setTimeout(update, props.debounce)
}

watch(
	() => [props.projectField, props.currentField] as const,
	([projectField, currentField], [previousProjectField]) => {
		if (projectField !== currentField) {
			validationIsStale.value = true
			validationRefreshCompleted = false
		} else if (projectField === previousProjectField) {
			validationIsStale.value = false
			validationRefreshCompleted = false
		}
	},
)

watch(projectValidationLoading, (loading, wasLoading) => {
	if (!loading && wasLoading) {
		validationRefreshCompleted = true
		updateDisplayedCheck(props.check)
	}
})

watch(() => props.check, updateDisplayedCheck)

onScopeDispose(() => clearTimeout(debounceTimer))

const validations = computed(() => {
	if (validationIsStale.value || projectValidationLoading.value) return []

	return Array.isArray(displayedCheck.value)
		? displayedCheck.value
		: displayedCheck.value
			? [displayedCheck.value]
			: []
})
</script>
