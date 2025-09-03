<template>
	<div
		:class="[
			'flex rounded-2xl border-2 border-solid p-4 gap-4 font-semibold text-contrast',
			typeClasses[type],
		]"
	>
		<slot name="icon" :iconClass="['hidden h-8 w-8 flex-none sm:block', iconClasses[type]]">
			<component
				:is="icons[type]"
				:class="['hidden h-8 w-8 flex-none sm:block', iconClasses[type]]"
			/>
		</slot>
		<div class="flex flex-col gap-2">
			<div class="font-semibold flex justify-between gap-4">
				<slot name="header">{{ header }}</slot>
			</div>
			<div class="font-normal text-sm sm:text-base">
				<slot>{{ body }}</slot>
			</div>
			<div v-if="showActionsUnderneath">
				<slot name="actions" />
			</div>
		</div>
		<div v-if="!showActionsUnderneath" class="ml-auto w-fit">
			<slot name="actions" />
		</div>
	</div>
</template>

<script setup lang="ts">
import { InfoIcon, IssuesIcon, XCircleIcon } from '@modrinth/assets'

withDefaults(
	defineProps<{
		type?: 'info' | 'warning' | 'critical'
		header?: string
		body?: string
		showActionsUnderneath?: boolean
	}>(),
	{
		type: 'info',
		header: '',
		body: '',
		showActionsUnderneath: false,
	},
)

const typeClasses = {
	info: 'border-brand-blue bg-bg-blue',
	warning: 'border-brand-orange bg-bg-orange',
	critical: 'border-brand-red bg-bg-red',
}

const iconClasses = {
	info: 'text-brand-blue',
	warning: 'text-brand-orange',
	critical: 'text-brand-red',
}

const icons = {
	info: InfoIcon,
	warning: IssuesIcon,
	critical: XCircleIcon,
}
</script>
