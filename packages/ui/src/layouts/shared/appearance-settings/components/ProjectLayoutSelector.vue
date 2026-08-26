<script setup lang="ts">
import { RadioButtonCheckedIcon, RadioButtonIcon } from '@modrinth/assets'

import { defineMessages, useVIntl } from '#ui/composables'

import type { ProjectLayout } from '../types'

defineProps<{
	title: string
	modelValue: ProjectLayout
}>()

const emit = defineEmits<{
	'update:modelValue': [layout: ProjectLayout]
}>()

const { formatMessage } = useVIntl()
const layoutOptions = ['rows', 'grid'] as const
const messages = defineMessages({
	rows: {
		id: 'settings.display.project-list-layouts.mode.rows',
		defaultMessage: 'Rows',
	},
	grid: {
		id: 'settings.display.project-list-layouts.mode.grid',
		defaultMessage: 'Grid',
	},
})
</script>

<template>
	<fieldset class="m-0 w-full min-w-0 border-0 p-0">
		<legend class="mb-4 p-0 text-lg font-semibold text-contrast">
			{{ title }}
		</legend>
		<div class="grid grid-cols-[repeat(auto-fit,minmax(9.5rem,1fr))] gap-4">
			<button
				v-for="layout in layoutOptions"
				:key="layout"
				type="button"
				class="flex !w-full cursor-pointer flex-col overflow-hidden rounded-[var(--radius-md)] border border-solid border-divider bg-button-bg p-0 text-left text-primary outline-2 outline-transparent transition-[filter,transform] hover:brightness-[0.85] focus-visible:ring-4 focus-visible:ring-brand-shadow active:scale-[0.97] active:brightness-[0.8]"
				:class="{ '!text-contrast': modelValue === layout }"
				:aria-pressed="modelValue === layout"
				@click="emit('update:modelValue', layout)"
			>
				<div class="flex w-full items-center justify-center bg-surface-2 p-6">
					<div
						class="grid h-[4.5rem] w-28 gap-1"
						:class="layout === 'rows' ? 'grid-cols-1 grid-rows-4' : 'grid-cols-2 grid-rows-2'"
						aria-hidden="true"
					>
						<div
							v-for="previewCard in 4"
							:key="previewCard"
							class="rounded-lg border-2 border-solid"
							:class="
								modelValue === layout
									? 'border-brand bg-brand-highlight'
									: 'border-transparent bg-surface-4'
							"
						/>
					</div>
				</div>
				<div class="flex grow items-center px-4 py-3 text-left">
					<RadioButtonCheckedIcon
						v-if="modelValue === layout"
						class="mr-2 shrink-0 text-brand"
						aria-hidden="true"
					/>
					<RadioButtonIcon v-else class="mr-2 shrink-0" aria-hidden="true" />
					{{ formatMessage(messages[layout]) }}
				</div>
			</button>
		</div>
	</fieldset>
</template>
