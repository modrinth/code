<script setup lang="ts">
import { PowerIcon, PowerOffIcon } from '@modrinth/assets'
import { computed } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import FloatingActionBar from '#ui/components/base/FloatingActionBar.vue'
import ProgressBar from '#ui/components/base/ProgressBar.vue'
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
		defaultMessage: 'Enabling content... ({progress}/{total})',
	},
	bulkEnablingWaiting: {
		id: 'content.selection-bar.bulk.enabling-waiting',
		defaultMessage: 'Enabling content...',
	},
	bulkDisabling: {
		id: 'content.selection-bar.bulk.disabling',
		defaultMessage: 'Disabling content... ({progress}/{total})',
	},
	bulkDisablingWaiting: {
		id: 'content.selection-bar.bulk.disabling-waiting',
		defaultMessage: 'Disabling content...',
	},
	bulkUpdating: {
		id: 'content.selection-bar.bulk.updating',
		defaultMessage: 'Updating content... ({progress}/{total})',
	},
	bulkUpdatingWaiting: {
		id: 'content.selection-bar.bulk.updating-waiting',
		defaultMessage: 'Updating content...',
	},
	bulkDeleting: {
		id: 'content.selection-bar.bulk.deleting',
		defaultMessage: 'Deleting content... ({progress}/{total})',
	},
	bulkDeletingWaiting: {
		id: 'content.selection-bar.bulk.deleting-waiting',
		defaultMessage: 'Deleting content...',
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

const selectedCountText = computed(() => {
	if (props.contentTypeLabel) {
		return formatMessage(messages.selectedCount, {
			count: props.selectedItems.length,
			contentType: `${props.contentTypeLabel}${props.selectedItems.length === 1 ? '' : 's'}`,
		})
	}
	return formatMessage(messages.selectedCountSimple, {
		count: props.selectedItems.length,
	})
})

const bulkProgressMessage = computed(() => {
	if (!props.bulkOperation) return ''
	const messageMap = {
		enable: props.bulkWaiting ? messages.bulkEnablingWaiting : messages.bulkEnabling,
		disable: props.bulkWaiting ? messages.bulkDisablingWaiting : messages.bulkDisabling,
		update: props.bulkWaiting ? messages.bulkUpdatingWaiting : messages.bulkUpdating,
		delete: props.bulkWaiting ? messages.bulkDeletingWaiting : messages.bulkDeleting,
	}
	return formatMessage(messageMap[props.bulkOperation], {
		progress: props.bulkProgress,
		total: props.bulkTotal,
	})
})
</script>

<template>
	<FloatingActionBar :shown="shown" :aria-label="ariaLabel">
		<template v-if="!isBulkOperating">
			<div class="flex items-center gap-0.5">
				<span class="px-4 py-2.5 text-base font-semibold text-contrast">
					{{ selectedCountText }}
				</span>
				<div class="mx-1 h-6 w-px bg-surface-5" />
				<ButtonStyled type="transparent">
					<button class="!text-primary" @click="emit('clear')">
						{{ formatMessage(commonMessages.clearButton) }}
					</button>
				</ButtonStyled>
			</div>

			<div class="ml-auto flex items-center gap-0.5">
				<slot name="actions" />

				<ButtonStyled v-if="allDisabled" type="transparent">
					<button :disabled="isBusy" @click="emit('enable')">
						<PowerIcon />
						{{ formatMessage(messages.enable) }}
					</button>
				</ButtonStyled>
				<ButtonStyled v-else type="transparent">
					<button :disabled="isBusy" @click="emit('disable')">
						<PowerOffIcon />
						{{ formatMessage(messages.disable) }}
					</button>
				</ButtonStyled>

				<slot name="actions-end" />
			</div>
		</template>

		<template v-else>
			<div class="flex flex-1 flex-col gap-2" aria-live="polite">
				<span class="text-sm font-medium text-contrast">
					{{ bulkProgressMessage }}
				</span>
				<ProgressBar
					full-width
					:waiting="bulkWaiting"
					:progress="bulkProgress"
					:max="bulkTotal"
					color="brand"
				/>
			</div>
		</template>
	</FloatingActionBar>
</template>
