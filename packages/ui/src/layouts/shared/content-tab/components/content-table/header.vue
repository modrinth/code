<script setup lang="ts">
import { InfoIcon } from '@modrinth/assets'
import { ref } from 'vue'

import Checkbox from '#ui/components/base/Checkbox.vue'
import { useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import type { ContentTableLayout } from './types'

defineProps<{
	layout: ContentTableLayout
	stacked: boolean
	flat: boolean
	stuck: boolean
	selection: boolean
	allSelected: boolean
	someSelected: boolean
	selectionDisabled: boolean
	showEnabledFor: boolean
	showVersion: boolean
	showActions: boolean
	enabledForLabel: string
	enabledForDescription: string
}>()

const emit = defineEmits<{
	toggleAll: []
}>()

const { formatMessage } = useVIntl()

const element = ref<HTMLElement | null>(null)
defineExpose({ element })
</script>

<template>
	<div
		ref="element"
		role="rowgroup"
		class="sticky top-0 z-10 grid grid-cols-[var(--content-columns)] items-center gap-x-4 bg-surface-3 px-3"
		:class="[
			stacked ? 'gap-y-2 py-2' : 'h-12',
			flat || stuck ? 'rounded-none' : 'rounded-t-[20px]',
			stuck
				? 'transition-[border-radius] duration-100 border-0 border-y border-solid border-surface-4 shadow-md before:pointer-events-none before:absolute before:inset-x-0 before:-top-4 before:h-5 before:bg-surface-3'
				: '',
		]"
	>
		<div role="row" class="flex min-w-0 items-center gap-4" :class="{ 'col-span-full': stacked }">
			<Checkbox
				v-if="selection"
				:model-value="allSelected"
				:indeterminate="someSelected"
				:aria-label="formatMessage(commonMessages.selectAllLabel)"
				:disabled="selectionDisabled"
				class="shrink-0"
				@update:model-value="emit('toggleAll')"
			/>
			<span role="columnheader" class="font-semibold text-secondary">{{
				formatMessage(commonMessages.projectLabel)
			}}</span>
		</div>

		<div
			v-if="showEnabledFor"
			role="columnheader"
			class="flex min-w-0 items-center gap-1.5 whitespace-nowrap font-semibold text-secondary"
		>
			<span>{{ enabledForLabel }}</span>
			<span
				v-tooltip="enabledForDescription"
				role="img"
				:aria-label="enabledForDescription"
				class="inline-flex size-4 cursor-help items-center justify-center"
				tabindex="0"
			>
				<InfoIcon class="size-4" aria-hidden="true" />
			</span>
		</div>

		<div v-if="showVersion" role="columnheader" class="min-w-0">
			<span class="font-semibold text-secondary">{{
				formatMessage(commonMessages.versionLabel)
			}}</span>
		</div>

		<div
			v-if="showActions"
			role="columnheader"
			class="shrink-0 text-right"
			:class="layout === 'narrow' ? 'justify-self-start' : 'justify-self-end'"
		>
			<span class="font-semibold text-secondary">{{
				formatMessage(commonMessages.actionsLabel)
			}}</span>
		</div>
	</div>
</template>
