<template>
	<div class="min-w-0">
		<div class="mb-2 flex items-baseline justify-between gap-2">
			<component
				:is="inputId ? 'label' : 'span'"
				:id="labelId"
				:for="inputId"
				class="font-semibold text-contrast"
			>
				<slot name="label">{{ label }}</slot>
				<span v-if="labelOptional" class="font-normal text-secondary">
					{{ ' ' }}<IntlFormatted :message-id="commonMessages.optionalFieldLabel" />
				</span>
			</component>
			<slot name="right" />
		</div>
		<slot :label-id="labelId" />
		<slot name="bottom" />
	</div>
</template>

<script setup lang="ts">
import { useId } from 'vue'

import { commonMessages } from '../../../utils/common-messages'
import IntlFormatted from '../IntlFormatted.vue'

defineProps<{
	label?: string
	labelOptional?: boolean
	inputId?: string
}>()

const labelId = useId()
</script>
