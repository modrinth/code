<script setup lang="ts">
import { PowerIcon, PowerOffIcon } from '@modrinth/assets'
import { computed } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import FloatingActionBar from '#ui/components/base/FloatingActionBar.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import type { BulkOperationType } from '../composables/bulk-operations'
import type { ContentItem } from '../types'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	selectedCount: {
		id: 'content.selection-bar.selected-count',
		defaultMessage: '{count} {contentType} selected',
	},
	selectedCountSimple: {
		id: 'content.selection-bar.selected-count-simple',
		defaultMessage: '{count, number} selected',
	},
	enable: {
		id: 'content.selection-bar.enable',
		defaultMessage: 'Enable',
	},
	disable: {
		id: 'content.selection-bar.disable',
		defaultMessage: 'Disable',
	},
	bulkEnabling: {
		id: 'content.selection-bar.bulk.enabling',
		defaultMessage: 'Enabling {progress}/{total} {contentType}...',
	},
	bulkEnablingWaiting: {
		id: 'content.selection-bar.bulk.enabling-waiting',
		defaultMessage: 'Enabling {contentType}...',
	},
	bulkDisabling: {
		id: 'content.selection-bar.bulk.disabling',
		defaultMessage: 'Disabling {progress}/{total} {contentType}...',
	},
	bulkDisablingWaiting: {
		id: 'content.selection-bar.bulk.disabling-waiting',
		defaultMessage: 'Disabling {contentType}...',
	},
	bulkUpdating: {
		id: 'content.selection-bar.bulk.updating',
		defaultMessage: 'Updating {progress}/{total} {contentType}...',
	},
	bulkUpdatingWaiting: {
		id: 'content.selection-bar.bulk.updating-waiting',
		defaultMessage: 'Updating {contentType}...',
	},
	bulkDeleting: {
		id: 'content.selection-bar.bulk.deleting',
		defaultMessage: 'Deleting {progress}/{total} {contentType}...',
	},
	bulkDeletingWaiting: {
		id: 'content.selection-bar.bulk.deleting-waiting',
		defaultMessage: 'Deleting {contentType}...',
	},
	allAlreadyEnabled: {
		id: 'content.selection-bar.all-already-enabled',
		defaultMessage: 'All selected content is already enabled',
	},
	allAlreadyDisabled: {
		id: 'content.selection-bar.all-already-disabled',
		defaultMessage: 'All selected content is already disabled',
	},
})

interface Props {
	selectedItems: ContentItem[]
	contentTypeLabel?: string
	isBusy?: boolean
	isBulkOperating?: boolean
	bulkOperation?: BulkOperationType | null
	bulkProgress?: number
	bulkTotal?: number
	bulkWaiting?: boolean
	ariaLabel?: string
}

const props = withDefaults(defineProps<Props>(), {
	contentTypeLabel: undefined,
	isBusy: false,
	isBulkOperating: false,
	bulkOperation: null,
	bulkProgress: 0,
	bulkTotal: 0,
	bulkWaiting: false,
	ariaLabel: undefined,
})

const emit = defineEmits<{
	clear: []
	enable: []
	disable: []
}>()

const shown = computed(() => props.selectedItems.length > 0 || props.isBulkOperating)

const allDisabled = computed(() => props.selectedItems.every((m) => !m.enabled))
const allEnabled = computed(() => props.selectedItems.every((m) => m.enabled))

const selectedCountText = computed(() => {
	const count = props.selectedItems.length || props.bulkTotal
	if (props.contentTypeLabel) {
		return formatMessage(messages.selectedCount, {
			count,
			contentType: `${props.contentTypeLabel}${count === 1 ? '' : 's'}`,
		})
	}
	return formatMessage(messages.selectedCountSimple, { count })
})

const bulkProgressMessage = computed(() => {
	if (!props.bulkOperation) return ''
	const contentType = props.contentTypeLabel
		? `${props.contentTypeLabel}${props.bulkTotal === 1 ? '' : 's'}`
		: 'items'
	const messageMap = {
		enable: props.bulkWaiting ? messages.bulkEnablingWaiting : messages.bulkEnabling,
		disable: props.bulkWaiting ? messages.bulkDisablingWaiting : messages.bulkDisabling,
		update: props.bulkWaiting ? messages.bulkUpdatingWaiting : messages.bulkUpdating,
		delete: props.bulkWaiting ? messages.bulkDeletingWaiting : messages.bulkDeleting,
	}
	return formatMessage(messageMap[props.bulkOperation], {
		progress: props.bulkProgress,
		total: props.bulkTotal,
		contentType,
	})
})
</script>

<template>
	<FloatingActionBar :shown="shown" :aria-label="ariaLabel">
		<div class="flex items-center gap-0.5">
			<span class="px-4 py-2.5 text-base font-semibold text-contrast tabular-nums">
				{{ selectedCountText }}
			</span>
			<div class="mx-1 h-6 w-px bg-surface-5" />
			<ButtonStyled type="transparent">
				<button
					class="!text-primary"
					:disabled="isBulkOperating"
					:class="{ 'opacity-60 pointer-events-none': isBulkOperating }"
					@click="emit('clear')"
				>
					{{ formatMessage(commonMessages.clearButton) }}
				</button>
			</ButtonStyled>
		</div>

		<div v-if="!isBulkOperating" class="ml-auto flex items-center gap-0.5">
			<slot name="actions" />

			<ButtonStyled type="transparent">
				<button
					v-tooltip="allEnabled ? formatMessage(messages.allAlreadyEnabled) : undefined"
					:disabled="isBusy || allEnabled"
					@click="emit('enable')"
				>
					<PowerIcon />
					{{ formatMessage(messages.enable) }}
				</button>
			</ButtonStyled>
			<ButtonStyled type="transparent">
				<button
					v-tooltip="allDisabled ? formatMessage(messages.allAlreadyDisabled) : undefined"
					:disabled="isBusy || allDisabled"
					@click="emit('disable')"
				>
					<PowerOffIcon />
					{{ formatMessage(messages.disable) }}
				</button>
			</ButtonStyled>

			<slot name="actions-end" />
		</div>

		<div v-else class="ml-auto flex items-center" aria-live="polite">
			<span class="px-4 py-2.5 text-base font-semibold text-secondary tabular-nums">
				{{ bulkProgressMessage }}
			</span>
		</div>

		<div v-if="isBulkOperating" class="absolute bottom-0 left-0 right-0 h-1">
			<div
				class="h-full rounded-l-full bg-brand transition-[width] duration-200 ease-in-out"
				:class="{ 'animate-indeterminate': bulkWaiting }"
				:style="
					!bulkWaiting
						? { width: `${bulkTotal > 0 ? (bulkProgress / bulkTotal) * 100 : 0}%` }
						: undefined
				"
				role="progressbar"
				:aria-valuenow="bulkWaiting ? undefined : bulkProgress"
				:aria-valuemin="0"
				:aria-valuemax="bulkTotal"
				style="box-shadow: 0px -2px 4px 0px rgba(27, 217, 106, 0.1)"
			/>
		</div>
	</FloatingActionBar>
</template>

<style scoped>
@keyframes indeterminate {
	0% {
		width: 20%;
		margin-left: -20%;
	}
	100% {
		width: 60%;
		margin-left: 100%;
	}
}

.animate-indeterminate {
	animation: indeterminate 1.5s ease-in-out infinite;
}
</style>
