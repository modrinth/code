<template>
	<div class="flex items-center gap-1">
		<ButtonStyled v-if="showClear && hasLogs" type="transparent">
			<button @click="emit('clear')">
				<XIcon />
				{{ formatMessage(commonMessages.clearButton) }}
			</button>
		</ButtonStyled>
		<ButtonStyled v-if="showDelete" type="transparent" hover-color-fill="background" color="red">
			<button
				v-tooltip="deleteDisabled ? deleteDisabledTooltip : undefined"
				:disabled="deleteDisabled"
				@click="emit('delete')"
			>
				<TrashIcon />
				{{ formatMessage(commonMessages.deleteLabel) }}
			</button>
		</ButtonStyled>
		<ButtonStyled v-if="hasLogs" type="transparent">
			<button
				v-tooltip="shareDisabled ? shareDisabledTooltip : undefined"
				:disabled="shareDisabled || sharing"
				@click="emit('share')"
			>
				<SpinnerIcon v-if="sharing" class="animate-spin" />
				<ShareIcon v-else />
				{{ formatMessage(messages.share) }}
			</button>
		</ButtonStyled>
		<ButtonStyled type="transparent">
			<button @click="emit('toggle-fullscreen')">
				<ContractIcon v-if="fullscreen" />
				<ExpandIcon v-else />
				{{ fullscreen ? formatMessage(messages.collapse) : formatMessage(messages.expand) }}
			</button>
		</ButtonStyled>
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

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	share: {
		id: 'servers.console.action.share',
		defaultMessage: 'Share',
	},
	expand: {
		id: 'servers.console.action.expand',
		defaultMessage: 'Expand',
	},
	collapse: {
		id: 'servers.console.action.collapse',
		defaultMessage: 'Collapse',
	},
})

defineProps<{
	showClear?: boolean
	hasLogs?: boolean
	shareDisabled?: boolean
	shareDisabledTooltip?: string
	sharing?: boolean
	fullscreen?: boolean
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
