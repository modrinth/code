<template>
	<div class="flex items-center gap-1">
		<Button
			v-if="showClear && hasLogs"
			v-tooltip="clearDisabled ? clearDisabledTooltip : undefined"
			type="quiet"
			size="sm"
			:disabled="clearDisabled"
			@click="emit('clear')"
		>
			<XIcon aria-hidden="true" />
			{{ formatMessage(commonMessages.clearButton) }}
		</Button>
		<Button
			v-if="showDelete"
			v-tooltip="deleteDisabled ? deleteDisabledTooltip : undefined"
			type="quiet"
			size="sm"
			color="red"
			interaction="filled"
			:disabled="deleteDisabled"
			@click="emit('delete')"
		>
			<TrashIcon aria-hidden="true" />
			{{ formatMessage(commonMessages.deleteLabel) }}
		</Button>
		<Button
			v-if="hasLogs"
			v-tooltip="shareDisabled ? shareDisabledTooltip : undefined"
			type="quiet"
			size="sm"
			:disabled="shareDisabled"
			:loading="sharing"
			@click="emit('share')"
		>
			<SpinnerIcon v-if="sharing" class="animate-spin" aria-hidden="true" />
			<ShareIcon v-else aria-hidden="true" />
			{{ formatMessage(messages.share) }}
		</Button>
		<Button type="quiet" size="sm" @click="emit('toggle-fullscreen')">
			<ContractIcon v-if="fullscreen" aria-hidden="true" />
			<ExpandIcon v-else aria-hidden="true" />
			{{ formatMessage(fullscreen ? messages.collapse : messages.expand) }}
		</Button>
	</div>
</template>

<script setup lang="ts">
import {
	ContractIcon,
	ExpandIcon,
	ShareIcon,
	SpinnerIcon,
	TrashIcon,
	XIcon,
} from '@modrinth/assets'

import { Button } from '#ui/components/base/buttons'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

const { formatMessage } = useVIntl()
const messages = defineMessages({
	share: {
		id: 'console.actions.share',
		defaultMessage: 'Share',
	},
	collapse: {
		id: 'console.actions.collapse',
		defaultMessage: 'Collapse',
	},
	expand: {
		id: 'console.actions.expand',
		defaultMessage: 'Expand',
	},
})

defineProps<{
	showClear?: boolean
	hasLogs?: boolean
	shareDisabled?: boolean
	shareDisabledTooltip?: string
	sharing?: boolean
	fullscreen?: boolean
	clearDisabled?: boolean
	clearDisabledTooltip?: string
	showDelete?: boolean
	deleteDisabled?: boolean
	deleteDisabledTooltip?: string
}>()

const emit = defineEmits<{
	clear: []
	share: []
	'toggle-fullscreen': []
	delete: []
}>()
</script>
