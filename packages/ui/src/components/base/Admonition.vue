<template>
	<div
		:class="[
			'relative flex flex-col rounded-2xl border-[1px] border-solid p-4 gap-3 text-contrast',
			typeClasses[type],
		]"
	>
		<div class="flex items-start gap-2">
			<div
				:class="[
					'flex flex-1 gap-2',
					header || $slots.header ? 'flex-col items-start' : 'items-center',
					(dismissible || $slots['top-right-actions']) && 'pr-8',
				]"
			>
				<div
					class="flex gap-2 items-start"
					:class="header || $slots.header ? 'w-full' : 'contents'"
				>
					<slot name="icon" :icon-class="['h-6 w-6 flex-none', iconClasses[type]]">
						<component
							:is="getSeverityIcon(type)"
							:class="['h-6 w-6 flex-none', iconClasses[type]]"
						/>
					</slot>
					<div v-if="header || $slots.header" class="font-semibold text-base">
						<slot name="header">{{ header }}</slot>
					</div>
				</div>
				<div class="font-normal text-contrast/80" :class="!(header || $slots.header) && 'flex-1'">
					<slot>{{ body }}</slot>
				</div>
			</div>
			<div v-if="$slots['top-right-actions']" class="flex shrink-0 items-center gap-2">
				<slot name="top-right-actions" />
			</div>
			<ButtonStyled
				v-else-if="dismissible"
				circular
				type="highlight-colored-text"
				:color="buttonColors[type]"
			>
				<button aria-label="Dismiss" class="absolute top-3 right-3" @click="$emit('dismiss')">
					<XIcon class="h-4 w-4" />
				</button>
			</ButtonStyled>
		</div>
		<div v-if="$slots.progress">
			<slot name="progress" />
		</div>
		<div v-if="showActionsUnderneath || $slots.actions">
			<slot name="actions" />
		</div>
	</div>
</template>

<script setup lang="ts">
import { XIcon } from '@modrinth/assets'

import { getSeverityIcon } from '../../utils'
import ButtonStyled from './ButtonStyled.vue'

withDefaults(
	defineProps<{
		type?: 'info' | 'warning' | 'critical' | 'success'
		header?: string
		body?: string
		showActionsUnderneath?: boolean
		dismissible?: boolean
	}>(),
	{
		type: 'info',
		header: '',
		body: '',
		showActionsUnderneath: false,
		dismissible: false,
	},
)

defineEmits<{
	dismiss: []
}>()

const typeClasses = {
	info: 'border-brand-blue bg-bg-blue',
	warning: 'border-brand-orange bg-bg-orange',
	critical: 'border-brand-red bg-bg-red',
	success: 'border-brand-green bg-bg-green',
}

const iconClasses = {
	info: 'text-brand-blue',
	warning: 'text-brand-orange',
	critical: 'text-brand-red',
	success: 'text-brand-green',
}

const buttonColors: Record<string, 'blue' | 'orange' | 'red' | 'green'> = {
	info: 'blue',
	warning: 'orange',
	critical: 'red',
	success: 'green',
}
</script>
