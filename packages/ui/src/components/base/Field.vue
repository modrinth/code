<template>
	<div class="flex min-w-0 flex-col gap-2">
		<div class="flex items-baseline justify-between gap-2 empty:hidden">
			<component
				:is="inputId ? 'label' : 'span'"
				:id="labelId"
				:for="inputId"
				class="field-label font-semibold text-lg text-contrast empty:hidden"
			>
				<slot name="label">{{ label }}</slot>
				<span v-if="labelOptional" class="font-normal text-secondary">
					{{ ' ' }}<IntlFormatted :message-id="commonMessages.optionalFieldLabel" />
				</span>
			</component>
			<slot name="right" />
		</div>
		<slot :label-id="labelId" />
		<div v-if="$slots.subtext" :class="subtextClass">
			<slot name="subtext" />
		</div>
		<slot name="bottom" />
	</div>
</template>

<script setup lang="ts">
import { useId } from 'vue'

import { commonMessages } from '../../utils/common-messages'
import IntlFormatted from './IntlFormatted.vue'

withDefaults(
	defineProps<{
		label?: string
		labelOptional?: boolean
		inputId?: string
		subtextClass?: string
	}>(),
	{ subtextClass: 'leading-tight text-secondary' },
)

const labelId = useId()
</script>
