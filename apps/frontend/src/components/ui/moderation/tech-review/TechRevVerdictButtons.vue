<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { BanIcon, CheckCheckIcon, CheckIcon, ShieldAlertIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { canUpdateGlobalDetail } from './helpers'
import { injectTechReviewDecisions } from './use-tech-review-decisions'

const REMAINING_LABELS = {
	globalSafe: 'All remaining globally safe',
	localSafe: 'All remaining safe',
	localUnsafe: 'All remaining malware',
	globalUnsafe: 'All remaining globally unsafe',
} as const

const TRACE_ARIA_LABELS = {
	globalSafe: 'Global pass',
	localSafe: 'Local pass',
	localUnsafe: 'Local fail',
	globalUnsafe: 'Global fail',
} as const

const props = defineProps<{
	variant: 'remaining' | 'trace'
	remainingCount?: number
	jar?: boolean
	detail?: Labrinth.TechReview.Internal.ReportIssueDetail
	globalDisabled?: boolean
	localDisabled?: boolean
}>()

const emit = defineEmits<{
	globalSafe: []
	localSafe: []
	localUnsafe: []
	globalUnsafe: []
}>()

const {
	isDetailActionSelected,
	getDetailActionTooltip,
	updatingDetails,
	updatingGlobalDetailKeys,
} = injectTechReviewDecisions()

const groupAriaLabel = computed(() => {
	if (props.variant === 'trace') return 'Trace verdict actions'
	return props.jar ? 'Remaining JAR issue actions' : 'Remaining issue actions'
})

const remainingLabel = computed(() =>
	props.variant === 'remaining' && props.remainingCount != null
		? `${props.remainingCount} issue${props.remainingCount === 1 ? '' : 's'} remaining`
		: undefined,
)

const ariaLabels = computed(() =>
	props.variant === 'remaining' ? REMAINING_LABELS : TRACE_ARIA_LABELS,
)

function tooltip(decision: 'safe' | 'malware', scope: 'local' | 'global'): string {
	if (props.variant === 'remaining') {
		if (decision === 'safe' && scope === 'global') return REMAINING_LABELS.globalSafe
		if (decision === 'safe') return REMAINING_LABELS.localSafe
		if (scope === 'local') return REMAINING_LABELS.localUnsafe
		return REMAINING_LABELS.globalUnsafe
	}

	return getDetailActionTooltip(props.detail!, decision, scope)
}

function selected(decision: 'safe' | 'malware', scope: 'local' | 'global'): boolean {
	if (props.variant !== 'trace' || !props.detail) return false
	return isDetailActionSelected(props.detail, decision, scope)
}

const isGlobalDisabled = computed(() => {
	if (props.variant === 'remaining') return props.globalDisabled
	if (!props.detail) return true
	return (
		!canUpdateGlobalDetail(props.detail) ||
		updatingGlobalDetailKeys.has(props.detail.key) ||
		updatingDetails.has(props.detail.id)
	)
})

const isLocalDisabled = computed(() => {
	if (props.variant === 'remaining') return props.localDisabled
	if (!props.detail) return true
	return updatingDetails.has(props.detail.id) || updatingGlobalDetailKeys.has(props.detail.key)
})

const BUTTON_BASE_CLASS =
	'custom-focus-indicator flex size-8 cursor-pointer items-center justify-center border-0 border-l border-solid border-l-surface-5 bg-transparent p-0 transition-[background-color,filter] duration-150 ease-in-out first:rounded-s-[calc(var(--radius-md)-1px)] first:border-l-0 last:rounded-e-[calc(var(--radius-md)-1px)] disabled:cursor-not-allowed disabled:opacity-50 [&>svg]:size-4'

function buttonClass(decision: 'safe' | 'malware', scope: 'local' | 'global') {
	return [
		BUTTON_BASE_CLASS,
		decision === 'safe' ? 'text-green' : 'text-red',
		selected(decision, scope)
			? 'bg-bg-green shadow-[inset_0_0_0_1px_var(--color-green)] hover:bg-bg-green focus-visible:bg-bg-green focus-visible:shadow-[inset_0_0_0_2px_var(--color-green)]'
			: 'hover:bg-surface-4 focus-visible:bg-surface-4 focus-visible:shadow-[inset_0_0_0_2px_var(--color-brand)]',
	]
}
</script>

<template>
	<div
		class="flex items-center overflow-hidden rounded-xl border border-solid border-surface-5 bg-surface-3"
		role="group"
		:aria-label="groupAriaLabel"
	>
		<span
			v-if="remainingLabel"
			class="whitespace-nowrap px-3 text-sm font-semibold text-secondary"
			>{{ remainingLabel }}</span
		>
		<button
			v-tooltip="tooltip('safe', 'global')"
			:class="buttonClass('safe', 'global')"
			:aria-label="ariaLabels.globalSafe"
			:disabled="isGlobalDisabled"
			@click="emit('globalSafe')"
		>
			<CheckCheckIcon aria-hidden="true" />
		</button>
		<button
			v-tooltip="tooltip('safe', 'local')"
			:class="buttonClass('safe', 'local')"
			:aria-label="ariaLabels.localSafe"
			:disabled="isLocalDisabled"
			@click="emit('localSafe')"
		>
			<CheckIcon aria-hidden="true" />
		</button>
		<button
			v-tooltip="tooltip('malware', 'local')"
			:class="buttonClass('malware', 'local')"
			:aria-label="ariaLabels.localUnsafe"
			:disabled="isLocalDisabled"
			@click="emit('localUnsafe')"
		>
			<BanIcon aria-hidden="true" />
		</button>
		<button
			v-tooltip="tooltip('malware', 'global')"
			:class="buttonClass('malware', 'global')"
			:aria-label="ariaLabels.globalUnsafe"
			:disabled="isGlobalDisabled"
			@click="emit('globalUnsafe')"
		>
			<ShieldAlertIcon aria-hidden="true" />
		</button>
	</div>
</template>
